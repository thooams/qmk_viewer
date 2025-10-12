#!/bin/bash

# Build script for all platforms
set -e

echo "Building QMK Keyboard Viewer for all platforms..."

# Detect current platform
case "$(uname -s)" in
    Darwin*)
        PLATFORM="macos"
        ;;
    Linux*)
        PLATFORM="linux"
        ;;
    CYGWIN*|MINGW32*|MSYS*|MINGW*)
        PLATFORM="windows"
        ;;
    *)
        echo "Unknown platform: $(uname -s)"
        exit 1
        ;;
esac

echo "Detected platform: $PLATFORM"

# Build for current platform
case $PLATFORM in
    "macos")
        echo "Building for macOS..."
        ./scripts/build-macos.sh
        ;;
    "linux")
        echo "Building for Linux..."
        ./scripts/build-linux.sh
        ;;
    "windows")
        echo "Building for Windows..."
        if command -v pwsh &> /dev/null; then
            pwsh -File scripts/build-windows.ps1 -WithIcon
        else
            ./scripts/build-windows.bat
        fi
        ;;
esac

echo ""
echo "Build completed for $PLATFORM!"
echo ""
echo "Available build scripts:"
echo "  ./scripts/build-macos.sh    - Build macOS .app bundle"
echo "  ./scripts/build-linux.sh    - Build Linux AppDir"
echo "  ./scripts/build-windows.ps1 - Build Windows executable (PowerShell)"
echo "  ./scripts/build-windows.bat - Build Windows executable (Batch)"
echo ""
echo "To build for other platforms, run the appropriate script."
