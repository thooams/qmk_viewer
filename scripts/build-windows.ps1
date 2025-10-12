# PowerShell build script for Windows
param(
    [switch]$WithIcon = $false
)

Write-Host "Building QMK Keyboard Viewer for Windows..." -ForegroundColor Green

# Clean previous builds
if (Test-Path "dist\windows") {
    Remove-Item -Recurse -Force "dist\windows"
}
New-Item -ItemType Directory -Path "dist\windows" -Force | Out-Null

# Build the application
Write-Host "Building Rust application..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Copy executable
Copy-Item "target\release\qmk_viewer.exe" "dist\windows\QMK_Keyboard_Viewer.exe"

if ($WithIcon) {
    Write-Host "Creating Windows icon..." -ForegroundColor Yellow
    
    # Check if ImageMagick is available
    $magick = Get-Command magick -ErrorAction SilentlyContinue
    if ($magick) {
        # Convert PNG to ICO
        & magick "src\assets\images\qmk-viewer.png" -resize 256x256 "dist\windows\qmk-viewer.ico"
        Write-Host "Icon created: dist\windows\qmk-viewer.ico" -ForegroundColor Green
    } else {
        Write-Host "ImageMagick not found. Skipping icon creation." -ForegroundColor Yellow
        Write-Host "To create an icon, install ImageMagick and run with -WithIcon flag" -ForegroundColor Yellow
    }
}

Write-Host "Windows executable created successfully!" -ForegroundColor Green
Write-Host "Location: dist\windows\QMK_Keyboard_Viewer.exe" -ForegroundColor Cyan
