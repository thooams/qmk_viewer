#!/bin/bash

# Complete macOS build script that handles all issues
set -e

echo "Building QMK Keyboard Viewer for macOS (Complete)..."

# Clean previous builds
rm -rf dist/macos
mkdir -p dist/macos

# Create app bundle structure
mkdir -p "dist/macos/QMK Keyboard Viewer.app/Contents/{MacOS,Resources}"

# Create Info.plist
cat > "dist/macos/QMK Keyboard Viewer.app/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>qmk_viewer</string>
    <key>CFBundleIdentifier</key>
    <string>com.qmk.keyboard-viewer</string>
    <key>CFBundleName</key>
    <string>QMK Keyboard Viewer</string>
    <key>CFBundleDisplayName</key>
    <string>QMK Keyboard Viewer</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleIconFile</key>
    <string>qmk-viewer</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
</dict>
</plist>
EOF

# Create iconset
mkdir -p dist/macos/qmk-viewer.iconset

# Generate different icon sizes
echo "Creating icon sizes..."
sips -z 16 16 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_16x16.png
sips -z 32 32 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_16x16@2x.png
sips -z 32 32 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_32x32.png
sips -z 64 64 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_32x32@2x.png
sips -z 128 128 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_128x128.png
sips -z 256 256 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_128x128@2x.png
sips -z 256 256 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_256x256.png
sips -z 512 512 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_256x256@2x.png
sips -z 512 512 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_512x512.png
sips -z 1024 1024 src/assets/images/qmk-viewer.png --out dist/macos/qmk-viewer.iconset/icon_512x512@2x.png

# Create .icns file
echo "Creating .icns file..."
iconutil -c icns dist/macos/qmk-viewer.iconset -o dist/macos/qmk-viewer.icns
cp dist/macos/qmk-viewer.icns "dist/macos/QMK Keyboard Viewer.app/Contents/Resources/"

# Build the application
echo "Building Rust application..."
cargo build --release

# Copy executable to bundle
echo "Copying executable..."
cp target/release/qmk_viewer "dist/macos/QMK Keyboard Viewer.app/Contents/MacOS/"

# Set proper permissions
echo "Setting permissions..."
chmod +x "dist/macos/QMK Keyboard Viewer.app/Contents/MacOS/qmk_viewer"
chmod -R 755 "dist/macos/QMK Keyboard Viewer.app"

# Remove quarantine attribute
echo "Removing quarantine attribute..."
xattr -d com.apple.quarantine "dist/macos/QMK Keyboard Viewer.app" 2>/dev/null || true
xattr -d com.apple.metadata:kMDItemWhereFroms "dist/macos/QMK Keyboard Viewer.app" 2>/dev/null || true

echo ""
echo "✅ macOS bundle created successfully!"
echo "📍 Location: dist/macos/QMK Keyboard Viewer.app"
echo "🎯 You can now double-click the app to run it."
echo ""
echo "If you still get an error, try:"
echo "1. Right-click the app and select 'Open'"
echo "2. Or run: open 'dist/macos/QMK Keyboard Viewer.app'"
