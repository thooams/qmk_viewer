# QMK Keyboard Viewer

A cross-platform application for viewing and visualizing QMK keyboard layouts with support for any QMK-compatible keyboard.

## Features

- **Universal QMK Support**: Works with any QMK keyboard by auto-detecting dimensions
- **Comprehensive Keycode Mapping**: Supports all basic QMK keycodes with proper glyph conversion
- **Dynamic UI**: Automatically adapts to any keyboard size
- **Cross-Platform**: Native builds for macOS, Windows, and Linux
- **Layer Visualization**: View and switch between keyboard layers
- **Real-time Updates**: Live keyboard state monitoring (when connected)

## Quick Start

### Building

```bash
# Build for current platform
./build-all.sh

# Or build specifically:
./build-macos-complete.sh    # macOS
./build-linux.sh             # Linux
./build-windows.ps1          # Windows (PowerShell)
```

### Running

```bash
# Development
cargo run

# Or run the built application
# macOS: Double-click the .app bundle
# Linux: ./dist/linux/QMK_Keyboard_Viewer.AppDir/AppRun
# Windows: Run the .exe file
```

## Project Structure

```
src/
├── keycodes.rs          # QMK keycode mappings
├── keyboard.rs          # Generic keyboard structures
├── keyboards/
│   ├── mod.rs          # Keyboards module
│   └── planck.rs       # Planck-specific defaults
├── assets/
│   └── images/         # Application icons
├── ui.rs               # Dynamic UI
├── main.rs             # Entry point
├── config.rs           # Configuration loading
└── lib.rs              # Module exports
```

## Build Artifacts

The following files and directories are automatically ignored by git:

- `/dist/` - Build distributions for all platforms
- `/target/` - Rust build artifacts
- `*.icns`, `*.ico` - Platform-specific icon files
- `*.app/`, `*.AppDir/`, `*.exe` - Built applications
- `*.DS_Store` - macOS system files
- Temporary icon files (`qmk-viewer-*.png`)

## Development

### Prerequisites

- Rust (latest stable)
- Platform-specific tools (see BUILD.md)

### Testing

```bash
cargo test
```

### Adding New Keyboards

1. Create a new file in `src/keyboards/`
2. Implement the `Default` trait for your keyboard layout
3. Add the module to `src/keyboards/mod.rs`
4. Update the main application to use your keyboard

## Documentation

- [BUILD.md](BUILD.md) - Detailed build instructions
- [QMK Documentation](https://docs.qmk.fm/) - QMK firmware reference

## License

MIT License - see LICENSE file for details.