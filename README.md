# Contextify

A context builder tool for LLM code understanding. Generates structured context from selected source files in multiple formats (TOON, XML, TOML, JSON).

## Features

- **GUI Application** (eframe/egui) — visual file selection
- **CLI Tool** (`ctx`) — command-line interface for automation
- **Wayland Support** — clipboard works on Linux/Wayland
- **Multiple Formats** — TOON, XML, TOML, JSON output

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
./target/release/contextify
./target/release/contextify /path/to/project
```

1. Select files from the tree view
2. Choose output format (TOON, XML, TOML, JSON)
3. Enter prompt/instructions
4. Click "Copy to Clipboard"

### Keyboard Shortcuts

- Select/deselect files with checkboxes
- Expand/collapse directories

## CLI Usage

```bash
# Show help
ctx --help

# List files in directory
ctx --root src/ list

# Search files by name
ctx --root . search main

# Generate context (stdout)
ctx generate src/main.rs src/lib.rs
ctx generate src/*.rs -f xml

# Copy to clipboard
ctx copy src/main.rs
ctx copy src/main.rs src/lib.rs -f toml

# Show available formats
ctx formats
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-r, --root` | Root directory | `.` |
| `-f, --format` | Output format | `toon` |

### Formats

- **TOON** — Human-readable format with code blocks
- **XML** — Standard XML structure
- **TOML** — TOML table format
- **JSON** — JSON with escaped content

## Examples

### Generate context for a Rust project

```bash
ctx generate src/*.rs -f toon
```

### Copy specific files to clipboard

```bash
ctx copy src/main.rs src/lib.rs -f json
```

### Search and generate

```bash
ctx --root . search handler | head -5
ctx generate $(ctx --root . search model) -f xml
```

## Configuration

Instructions file: `~/.contextify`

Save default instructions that will be prepended to every context:

```bash
echo "Analyze this code and suggest improvements" > ~/.contextify
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run GUI
cargo run

# Run CLI
cargo run --bin ctx -- generate src/main.rs
```

## Requirements

- Rust 1.70+
- Linux with Wayland (or X11)
- `wl-clipboard` for CLI clipboard (optional, arboard used by default)

## License

MIT