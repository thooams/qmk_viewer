#!/bin/bash

# Build script for Linux AppImage
set -e

# Change to the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🐧 Building QMK Keyboard Viewer for Linux..."
echo "📂 Working directory: $(pwd)"

# Clean previous builds
rm -rf dist/linux
mkdir -p dist/linux

# Build the application with rawhid feature
echo "🦀 Building Rust application with rawhid feature..."
cargo build --release --features rawhid

# Create AppDir structure
mkdir -p dist/linux/QMK_Keyboard_Viewer.AppDir/{usr/bin,usr/share/applications,usr/share/icons/hicolor/256x256/apps}

# Copy executable
cp target/release/qmk_viewer dist/linux/QMK_Keyboard_Viewer.AppDir/usr/bin/

# Create desktop file
cat > dist/linux/QMK_Keyboard_Viewer.AppDir/usr/share/applications/qmk-keyboard-viewer.desktop << 'EOF'
[Desktop Entry]
Name=QMK Keyboard Viewer
Comment=View and visualize QMK keyboard layouts
Exec=qmk_viewer
Icon=qmk-keyboard-viewer
Type=Application
Categories=Utility;
StartupNotify=true
EOF

# Copy and resize icon
cp src/assets/images/qmk-viewer.png dist/linux/QMK_Keyboard_Viewer.AppDir/usr/share/icons/hicolor/256x256/apps/qmk-keyboard-viewer.png

# Create AppRun script
cat > dist/linux/QMK_Keyboard_Viewer.AppDir/AppRun << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/qmk_viewer" "$@"
EOF

chmod +x dist/linux/QMK_Keyboard_Viewer.AppDir/AppRun

# Create AppImage metadata
cat > dist/linux/QMK_Keyboard_Viewer.AppDir/AppImage << 'EOF'
{
  "type": 2,
  "version": "1.0.0",
  "name": "QMK Keyboard Viewer",
  "exec": "qmk_viewer",
  "icon": "qmk-keyboard-viewer"
}
EOF

echo "Linux AppDir created successfully!"
echo "Location: dist/linux/QMK_Keyboard_Viewer.AppDir"
echo ""
echo "To create an AppImage, you need to:"
echo "1. Download linuxdeploy from https://github.com/linuxdeploy/linuxdeploy"
echo "2. Run: linuxdeploy --appdir dist/linux/QMK_Keyboard_Viewer.AppDir --output appimage"
echo ""
echo "Or use the AppDir directly by running:"
echo "cd dist/linux/QMK_Keyboard_Viewer.AppDir && ./AppRun"
