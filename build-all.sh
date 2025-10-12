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
        ./build-macos.sh
        ;;
    "linux")
        echo "Building for Linux..."
        ./build-linux.sh
        ;;
    "windows")
        echo "Building for Windows..."
        if command -v pwsh &> /dev/null; then
            pwsh -File build-windows.ps1 -WithIcon
        else
            ./build-windows.bat
        fi
        ;;
esac

echo ""
echo "Build completed for $PLATFORM!"
echo ""
echo "Available build scripts:"
echo "  ./build-macos.sh    - Build macOS .app bundle"
echo "  ./build-linux.sh    - Build Linux AppDir"
echo "  ./build-windows.ps1 - Build Windows executable (PowerShell)"
echo "  ./build-windows.bat - Build Windows executable (Batch)"
echo ""
echo "To build for other platforms, run the appropriate script."
