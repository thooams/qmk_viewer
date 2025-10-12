# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-01-15

### Added
- **Generalized QMK keyboard support**: Application now supports any QMK-compatible keyboard, not just Planck
- **Dynamic keyboard layout detection**: Automatically detects rows/columns from loaded keymap data
- **Comprehensive keycode mapping**: Complete mapping of QMK keycodes with human-readable labels
- **Drag & drop file loading**: Intuitive file loading with drag & drop zone
- **File browser integration**: Native file dialog for selecting keymap files
- **Persistent configuration**: Automatically saves and reloads last used keymap file
- **Raw HID support**: Real-time keyboard state monitoring via USB HID
- **QMK Console support**: Serial communication for keyboard state monitoring
- **Cross-platform builds**: Automated builds for macOS, Windows, and Linux
- **Comprehensive testing**: Compatibility testing with 200+ QMK keyboards
- **Application icon**: Custom icon for macOS app bundle
- **CI/CD workflows**: Automated testing and release management

### Changed
- **Refactored architecture**: Moved from Planck-specific to generic keyboard support
- **Improved UI**: Dynamic layout that adapts to any keyboard size
- **Enhanced keycode translation**: Replaced glyphs with plain text for better compatibility
- **Simplified build process**: Unified build scripts with automatic rawhid feature activation
- **Better error handling**: More robust file parsing and error reporting

### Fixed
- **Bit shift overflow**: Fixed potential overflow when handling keyboards with >64 keys
- **File path handling**: Resolved issues with file selection and loading
- **macOS app bundle**: Fixed icon generation and app bundle structure
- **Build timeouts**: Optimized build process for faster compilation
- **Test compatibility**: Fixed failing tests and improved test coverage

### Technical Improvements
- **Modular design**: Separated concerns into dedicated modules (keycodes, keyboard, config_persistence)
- **Enhanced parser**: Improved C keymap parser to handle complex QMK structures
- **Better documentation**: Comprehensive README with usage instructions and configuration guides
- **Test organization**: Centralized test files and improved test structure

### Breaking Changes
- **File structure**: Moved test files to `tests/files/` directory
- **Build scripts**: Moved all build scripts to `scripts/` directory
- **Keycode format**: Changed from glyph-based to plain text keycode display

## [0.1.0] - 2024-01-01

### Added
- Initial release with Planck keyboard support
- Basic keymap visualization
- Layer switching functionality
- Simple UI for keyboard state display


