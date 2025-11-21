# pw-screencast

A command-line tool that launches programs with a PipeWire file descriptor from the XDG Desktop Portal screencast API.

## Usage

```bash
pw-sc <command> [args...]
```

Use `{}` in arguments to substitute the PipeWire file descriptor number:

```bash
pw-sc gst-launch-1.0 pipewiresrc fd={} target-object=gnome-shell ! gtk4paintablesink
```

This will prompt you to select a screen/window to capture on GNOME shell, then display that with GStreamer.

## Requirements

- Linux with PipeWire
- XDG Desktop Portal implementation (e.g., xdg-desktop-portal-gtk or xdg-desktop-portal-kde)

## Build

```bash
cargo build --release
```

The binary will be at `target/release/pw-sc`.

## How It Works

1. Creates a screencast session via XDG Desktop Portal
2. Obtains a PipeWire file descriptor for the screen capture stream
3. Clears the close-on-exec flag on the file descriptor
4. Spawns your command with `{}` replaced by the fd number

This allows PipeWire-capable applications to record the screen without implementing portal integration themselves.
