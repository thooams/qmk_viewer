#!/bin/bash

# Script to fix macOS app bundle permissions and quarantine
set -e

APP_PATH="dist/macos/QMK Keyboard Viewer.app"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: App bundle not found at $APP_PATH"
    echo "Please run ./scripts/build-macos.sh first"
    exit 1
fi

echo "Fixing macOS app bundle..."

# Set proper permissions
chmod -R 755 "$APP_PATH"
chmod +x "$APP_PATH/Contents/MacOS/qmk_viewer"

# Remove quarantine attribute
echo "Removing quarantine attribute..."
xattr -d com.apple.quarantine "$APP_PATH" 2>/dev/null || true

# Remove other potentially problematic attributes
xattr -d com.apple.metadata:kMDItemWhereFroms "$APP_PATH" 2>/dev/null || true

echo "App bundle fixed!"
echo "You should now be able to run the app by double-clicking it."
echo ""
echo "If you still get an error, try:"
echo "1. Right-click the app and select 'Open'"
echo "2. Or run: open '$APP_PATH'"
