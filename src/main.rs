use anyhow::{Context, Result};
use ashpd::desktop::PersistMode;
use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging only if PW_SCREENCAST_LOG is set
    if std::env::var("PW_SCREENCAST_LOG").is_ok() {
        fmt()
            .with_env_filter(EnvFilter::from_env("PW_SCREENCAST_LOG"))
            .init();
    }

    info!("Starting pw-screencast");

    // Parse CLI arguments (skip program name)
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        error!("No command provided");
        anyhow::bail!("No command provided. Usage: sc <command> [args...]");
    }

    info!("Command: {} with {} args", args[0], args.len() - 1);

    // Initialize screencast portal
    info!("Initializing screencast portal...");
    let proxy = Screencast::new().await?;
    info!("Screencast portal initialized");

    // Create a screencast session
    info!("Creating screencast session...");
    let session = proxy
        .create_session()
        .await
        .context("Failed to create screencast session")?;
    info!("Screencast session created");

    // Select sources (all types: monitor, window, virtual)
    info!("Selecting screencast sources (this may show a dialog)...");
    proxy
        .select_sources(
            &session,
            CursorMode::Embedded, // Show cursor in the stream
            SourceType::Monitor | SourceType::Window, // Accept all source types
            false,
            None,
            PersistMode::DoNot,
        )
        .await
        .context("Failed to select screencast sources")?;
    info!("Sources selected");

    // Start the screencast
    info!("Starting screencast...");
    let _response = proxy
        .start(&session, None)
        .await
        .context("Failed to start screencast")?;
    info!("Screencast started");

    // Open PipeWire remote connection and get the fd
    info!("Opening PipeWire remote connection...");
    let pipewire_fd = proxy
        .open_pipe_wire_remote(&session)
        .await
        .context("Failed to open PipeWire remote")?;

    let raw_fd = pipewire_fd.as_raw_fd();
    info!("Got PipeWire fd: {}", raw_fd);

    // Build the command, replacing {} with the fd number
    let program = &args[0];
    let fd_string = raw_fd.to_string();
    let cmd_args: Vec<String> = args[1..]
        .iter()
        .map(|arg| arg.replace("{}", &fd_string))
        .collect();

    info!("Spawning subprocess: {} {:?}", program, cmd_args);

    // Spawn subprocess with fd preserved
    // Clear FD_CLOEXEC flag so the fd survives exec() into the child process
    let status = unsafe {
        Command::new(program)
            .args(&cmd_args)
            .pre_exec(move || {
                // Clear the close-on-exec flag so fd is inherited by child
                let flags = libc::fcntl(raw_fd, libc::F_GETFD);
                if flags >= 0 {
                    libc::fcntl(raw_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC);
                }
                Ok(())
            })
            .status()
            .context("Failed to spawn subprocess")?
    };

    info!("Subprocess exited with status: {}", status);

    if !status.success() {
        error!("Subprocess failed with status: {}", status);
        anyhow::bail!("Subprocess exited with status: {}", status);
    }

    info!("Done!");
    Ok(())
}
