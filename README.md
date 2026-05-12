# DustyControl

A system resources monitor with sound alarms function.

## Features

- **GUI Application** (iced) — visual application
- **Dashboard** - system resources monitor
- **Processes** - system processes control
- **Alarms** - sound alarms history
- **Settings** - General settings and Alarm Rules

## Installation

```bash
# Clone and build
cargo build --release

# Install binaries
cargo install --path .
```

## GUI Usage

```bash
# Launch GUI
./target/release/DustyControl
```

### Sound Alarms

- **beep**
- **alert**
- **alarm**
- **notification**
- **critical**
- **warnsiren**
- **chime**

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run GUI
cargo run
```

## Requirements

- Rust 1.70+
- Linux with Wayland (not tested on any other system)
- `wl-clipboard` for CLI clipboard (optional, arboard used by default)

## License

MIT
