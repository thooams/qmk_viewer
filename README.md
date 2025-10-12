# QMK Keyboard Viewer

A cross-platform application for viewing and visualizing QMK keyboard layouts with support for any QMK-compatible keyboard.

## Features

- **Universal QMK Support**: Works with any QMK keyboard by auto-detecting dimensions
- **Comprehensive Keycode Mapping**: Supports all basic QMK keycodes with proper glyph conversion
- **Dynamic UI**: Automatically adapts to any keyboard size
- **Cross-Platform**: Native builds for macOS, Windows, and Linux
- **Layer Visualization**: View and switch between keyboard layers
- **Real-time Updates**: Live keyboard state monitoring (when connected)
- **File Loading**: Load keymap files directly (.json, .c, .h formats)
- **Interactive Interface**: Browse and load keymap files with visual feedback

## Quick Start

### Building

```bash
# Build for current platform
./scripts/build-all.sh

# Or build specifically:
./scripts/build-macos-complete.sh    # macOS
./scripts/build-linux.sh             # Linux
./scripts/build-windows.ps1          # Windows (PowerShell)
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

### Loading Keymap Files

1. **Start the application**
2. **Enter file path** in the "File:" input field, or click "📂 Browse"
3. **Click "✅ Load"** to load the keymap
4. **View the status** - success or error messages will appear below the buttons

**Supported formats:**
- `.json` - QMK JSON keymap files
- `.c` / `.h` - QMK C keymap files

**Example files:**
- `tests/files/test_keymap.json` - Sample JSON keymap
- `tests/files/test_keymap.c` - Sample C keymap

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

scripts/
├── build-all.sh         # Build for all platforms
├── build-macos.sh       # macOS build
├── build-macos-complete.sh # Complete macOS build with icons
├── build-linux.sh       # Linux build
├── build-windows.bat    # Windows build (batch)
├── build-windows.ps1    # Windows build (PowerShell)
└── fix-macos-app.sh     # macOS app fix script

tests/
├── files/               # Test keymap files
│   ├── test_keymap.json
│   ├── test_keymap.c
│   └── thooams.json
└── *.rs                 # Test modules
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