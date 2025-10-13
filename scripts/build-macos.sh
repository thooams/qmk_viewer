#!/bin/bash

# Unified macOS build script
set -e

# Change to the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🍎 Building QMK Viewer for macOS..."
echo "📂 Working directory: $(pwd)"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist/macos
mkdir -p dist/macos

# Create app bundle structure
echo "📁 Creating app bundle structure..."
mkdir -p "dist/macos/QMK Viewer.app/Contents/MacOS"
mkdir -p "dist/macos/QMK Viewer.app/Contents/Resources"

# Create Info.plist
echo "📋 Creating Info.plist..."
cat > "dist/macos/QMK Viewer.app/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>qmk_viewer</string>
    <key>CFBundleIdentifier</key>
    <string>com.qmk.keyboard-viewer</string>
    <key>CFBundleName</key>
    <string>QMK Viewer</string>
    <key>CFBundleDisplayName</key>
    <string>QMK Viewer</string>
    <key>CFBundleVersion</key>
    <string>1.0.1</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.1</string>
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

# Create iconset directory
echo "🎨 Creating icon set..."
mkdir -p dist/macos/qmk-viewer.iconset

# Check if source icon exists
if [ ! -f "src/assets/images/qmk-viewer.png" ]; then
    echo "❌ Error: Source icon not found at src/assets/images/qmk-viewer.png"
    echo "   Please ensure the icon file exists before building."
    exit 1
fi

# Generate different icon sizes
echo "🖼️  Generating icon sizes..."
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
echo "📦 Creating .icns file..."
iconutil -c icns dist/macos/qmk-viewer.iconset -o dist/macos/qmk-viewer.icns

# Copy icon to app bundle
echo "📋 Copying icon to app bundle..."
cp dist/macos/qmk-viewer.icns "dist/macos/QMK Viewer.app/Contents/Resources/"

# Build the Rust application with rawhid feature
echo "🦀 Building Rust application with rawhid feature..."
echo "⏳ This may take several minutes for the first build..."
cargo build --release --features rawhid --verbose

# Check if build was successful
if [ ! -f "target/release/qmk_viewer" ]; then
    echo "❌ Error: Rust build failed - executable not found"
    exit 1
fi

# Copy executable to bundle
echo "📦 Copying executable to app bundle..."
cp target/release/qmk_viewer "dist/macos/QMK Viewer.app/Contents/MacOS/"

# Set proper permissions
echo "🔐 Setting permissions..."
chmod +x "dist/macos/QMK Viewer.app/Contents/MacOS/qmk_viewer"
chmod -R 755 "dist/macos/QMK Viewer.app"

# Remove quarantine attribute
echo "🛡️  Removing quarantine attribute..."
xattr -d com.apple.quarantine "dist/macos/QMK Viewer.app" 2>/dev/null || true
xattr -d com.apple.metadata:kMDItemWhereFroms "dist/macos/QMK Viewer.app" 2>/dev/null || true

# Verify the app bundle
echo "✅ Verifying app bundle..."
if [ -f "dist/macos/QMK Viewer.app/Contents/MacOS/qmk_viewer" ] && [ -f "dist/macos/QMK Viewer.app/Contents/Resources/qmk-viewer.icns" ]; then
    echo ""
    echo "🎉 macOS bundle created successfully!"
    echo "📍 Location: dist/macos/QMK Viewer.app"
    echo "🎯 You can now double-click the app to run it."
    echo ""
    echo "📊 Bundle size: $(du -sh 'dist/macos/QMK Viewer.app' | cut -f1)"
    echo ""
    echo "💡 If you get a security warning:"
    echo "   1. Right-click the app and select 'Open'"
    echo "   2. Or run: open 'dist/macos/QMK Viewer.app'"
    echo "   3. Or run: ./scripts/fix-macos-app.sh"
else
    echo "❌ Error: App bundle verification failed"
    echo "   Missing executable or icon file"
    exit 1
fi