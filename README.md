# GraphTerm

**Terminal-Native Graphical File Manager**

A mouse-first file manager that renders pixel graphics inside the terminal using Kitty/Sixel protocols, with full SSH compatibility.

## Quick Start

```bash
# Build
cargo build --release

# Run
cargo run

# Or run the release binary
./target/release/graphterm
```

## Running in Kitty Terminal

For best experience with graphics support, run inside Kitty:

```bash
# Open Kitty terminal
kitty

# Then run GraphTerm
cd /path/to/graphterm
cargo run
```

## Controls

| Action | Control |
|--------|---------|
| Navigate | ↑ ↓ Arrow keys |
| Open folder | Enter |
| Go back | Backspace |
| Quit | q or Esc |
| Select | Mouse click |
| Context menu | Right-click |
| Scroll | Mouse wheel |

## Features (v0.1)

- [x] File grid with emoji icons
- [x] Mouse click to select
- [x] Keyboard navigation
- [x] Directory navigation (Enter/Backspace)
- [x] Terminal graphics protocol detection
- [ ] Right-click context menu (WIP)
- [ ] Image thumbnails (Phase 2)
- [ ] File operations (Phase 3)

## Development

```bash
# Check for errors
cargo check

# Run with debug output
RUST_BACKTRACE=1 cargo run

# Build release
cargo build --release
```

## License

MIT
