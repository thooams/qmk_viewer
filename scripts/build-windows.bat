@echo off
REM Build script for Windows executable with icon
setlocal enabledelayedexpansion

REM Change to the project root directory
cd /d "%~dp0\.."

echo 🪟 Building QMK Keyboard Viewer for Windows...
echo 📂 Working directory: %CD%

REM Clean previous builds
if exist dist\windows rmdir /s /q dist\windows
mkdir dist\windows

REM Create Windows icon from PNG
echo Creating Windows icon...
REM Note: This requires ImageMagick or similar tool
REM For now, we'll create a simple batch that can be extended

REM Build the application with rawhid feature
echo 🦀 Building Rust application with rawhid feature...
cargo build --release --features rawhid

REM Copy executable to dist folder
copy target\release\qmk_viewer.exe dist\windows\QMK_Keyboard_Viewer.exe

echo Windows executable created successfully!
echo Location: dist\windows\QMK_Keyboard_Viewer.exe
echo.
echo Note: To add an icon to the Windows executable, you need to:
echo 1. Convert the PNG to ICO format
echo 2. Use a resource editor or build tool to embed the icon
echo 3. Or use cargo-bundle or similar tool for proper Windows packaging
