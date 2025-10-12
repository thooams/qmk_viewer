# Building QMK Keyboard Viewer

This document explains how to build QMK Keyboard Viewer for different platforms.

## Prerequisites

- Rust (latest stable version)
- Cargo
- Platform-specific tools (see below)

## Build Scripts

### macOS
```bash
# Production build (optimized, larger file)
./scripts/build-macos.sh

# Development build (faster compilation, debug symbols)
./scripts/build-macos-dev.sh
```
Creates a `.app` bundle with proper icon support and fixes common macOS issues. **Automatically enables the `rawhid` feature for production builds.**

**Note**: The first production build may take several minutes as it compiles all dependencies. Subsequent builds will be faster.

**Requirements:**
- macOS with `sips` and `iconutil` (built-in)

**Troubleshooting:**
If you get "damaged or incomplete" error:
```bash
./scripts/fix-macos-app.sh
```

### Linux
```bash
./scripts/build-linux.sh
```
Creates an AppDir structure that can be converted to AppImage. **Automatically enables the `rawhid` feature for production builds.**

**Requirements:**
- Linux with standard build tools
- For AppImage: `linuxdeploy` (optional)

### Windows
```bash
# PowerShell (recommended)
pwsh -File scripts/build-windows.ps1 -WithIcon

# Or Batch
scripts/build-windows.bat
```
Creates a Windows executable. **Automatically enables the `rawhid` feature for production builds.**

**Requirements:**
- Windows with PowerShell (for icon support)
- ImageMagick (optional, for icon conversion)

### All Platforms
```bash
./scripts/build-all.sh
```
Automatically detects the current platform and builds accordingly.

## Output Locations

- **macOS**: `dist/macos/QMK Keyboard Viewer.app`
- **Linux**: `dist/linux/QMK_Keyboard_Viewer.AppDir`
- **Windows**: `dist/windows/QMK_Keyboard_Viewer.exe`

## Manual Build

If you prefer to build manually:

```bash
# Build the application
cargo build --release

# The executable will be in target/release/qmk_viewer
```

## Icon Support

The application includes icon support for all platforms:

- **macOS**: Uses `.icns` format with multiple resolutions
- **Linux**: Uses PNG format in AppDir structure
- **Windows**: Can use `.ico` format (requires ImageMagick)

## Troubleshooting

### macOS
- **"Damaged or incomplete" error**: Run `./fix-macos-app.sh` to fix permissions and quarantine attributes
- **Icon doesn't appear in dock**: Try rebuilding the icon cache:
  ```bash
  sudo rm -rfv /Library/Caches/com.apple.iconservices.store
  sudo find /private/var/folders/ -name com.apple.dock.iconcache -exec rm {} \;
  killall Dock
  ```
- **App won't open**: Right-click the app and select "Open" to bypass Gatekeeper

### Linux
- For AppImage creation, download `linuxdeploy` from GitHub
- Make sure the AppDir is executable: `chmod +x dist/linux/QMK_Keyboard_Viewer.AppDir/AppRun`

### Windows
- Install ImageMagick for icon conversion
- Use PowerShell for better error handling

## Development

For development builds:
```bash
cargo build
cargo run
```

For testing:
```bash
cargo test
```
