# QMK Keyboard Viewer

A cross-platform application for viewing and visualizing QMK keyboard layouts with support for any QMK-compatible keyboard.

## Demo

https://github.com/user-attachments/assets/screen.mov

## Features

- **Universal QMK Support**: Works with any QMK keyboard by auto-detecting dimensions
- **Comprehensive Keycode Mapping**: Supports all basic QMK keycodes with proper glyph conversion
- **Dynamic UI**: Automatically adapts to any keyboard size
- **Cross-Platform**: Native builds for macOS, Windows, and Linux
- **Layer Visualization**: View and switch between keyboard layers
- **Real-time Updates**: Live keyboard state monitoring (when connected)
- **File Loading**: Load keymap files directly (.json, .c, .h formats)
- **Interactive Interface**: Browse and load keymap files with visual feedback

## Quick Start

### Building

```bash
# Build for current platform (with rawhid feature enabled by default)
./scripts/build-all.sh

# Or build specifically:
./scripts/build-macos.sh             # macOS (with rawhid, production)
./scripts/build-macos-dev.sh         # macOS (with rawhid, development - faster)
./scripts/build-linux.sh             # Linux (with rawhid)
./scripts/build-windows.ps1          # Windows (PowerShell, with rawhid)
```

### Running

```bash
# Development
cargo run

# Or run the built application
# macOS: Double-click the .app bundle
# Linux: ./dist/linux/QMK_Keyboard_Viewer.AppDir/AppRun
# Windows: Run the .exe file
```

## Usage

### Command Line Arguments

```bash
# Run with a specific keymap file
cargo run path/to/keymap.json
cargo run path/to/keymap.c

# Run with serial port (QMK Console feature)
cargo run --features qmk_console path/to/keymap.json /dev/ttyUSB0

# Run with Raw HID support
cargo run --features rawhid path/to/keymap.json
```

### Build Features

The application supports optional features for different input sources:

```bash
# Default build (Mock HID - for testing/development)
cargo run

# With Raw HID support (USB keyboard input)
cargo run --features rawhid

# With QMK Console support (Serial port input)
cargo run --features qmk_console

# With both features enabled
cargo run --features rawhid,qmk_console
```

**Note**: The build scripts (`build-macos.sh`, `build-linux.sh`, `build-windows.ps1`) automatically enable the `rawhid` feature by default for production builds.

### Arguments

- **`<keymap_file>`** (optional): Path to a keymap file to load on startup
  - Supported formats: `.json`, `.c`, `.h`
  - If not provided, the application will show a drag & drop zone
  - If a saved keymap exists, it will be loaded automatically

- **`<serial_port>`** (optional, requires `qmk_console` feature): Serial port for QMK Console input
  - Examples: `/dev/ttyUSB0` (Linux), `COM3` (Windows), `/dev/cu.usbserial-*` (macOS)
  - Only used when `qmk_console` feature is enabled

### Examples

```bash
# Load a specific keymap file
cargo run tests/files/test_keymap.json

# Load keymap with Raw HID support
cargo run --features rawhid tests/files/test_keymap.c

# Load keymap with QMK Console on specific port
cargo run --features qmk_console tests/files/test_keymap.json /dev/ttyUSB0

# Run without any keymap (shows drag & drop zone)
cargo run
```

### Loading Keymap Files

The application provides multiple ways to load keymap files:

#### Drag & Drop (Recommended)
1. **Start the application** (without arguments)
2. **Drag and drop** a keymap file onto the central area
3. **Or click** the drop zone to browse for a file
4. The keymap will load automatically and be saved for future sessions

#### Command Line
```bash
# Load a keymap file directly
cargo run path/to/keymap.json
```

#### File Browser
1. **Click** on the drag & drop zone
2. **Select** your keymap file from the file browser
3. The keymap will load automatically

**Supported formats:**
- `.json` - QMK JSON keymap files
- `.c` / `.h` - QMK C keymap files

**Example files:**
- `tests/files/test_keymap.json` - Sample JSON keymap
- `tests/files/test_keymap.c` - Sample C keymap

### QMK Keyboard Configuration

To enable real-time communication between your QMK keyboard and the application, you need to configure your keyboard firmware:

#### Raw HID Support

Add the following to your `rules.mk`:
```makefile
# Enable Raw HID for real-time communication
RAW_ENABLE = yes
```

Add this to your `keymap.c`:
```c
#include "raw_hid.h"

// Send keyboard state to the application
void raw_hid_receive(uint8_t *data, uint8_t length) {
    // Handle incoming data from the application if needed
}

// Call this function to send keyboard state
void send_keyboard_state(void) {
    uint8_t data[32] = {0};
    
    // Pack the data: [layer][pressed_bits...]
    data[0] = get_highest_layer(layer_state);
    
    // Pack pressed keys into bits (up to 32 keys)
    for (int i = 0; i < 32 && i < MATRIX_ROWS * MATRIX_COLS; i++) {
        uint8_t row = i / MATRIX_COLS;
        uint8_t col = i % MATRIX_COLS;
        if (matrix_is_on(row, col)) {
            data[1 + (i / 8)] |= (1 << (i % 8));
        }
    }
    
    raw_hid_send(data, 32);
}

// Call this in your matrix scan or layer change functions
void matrix_scan_user(void) {
    static uint32_t last_send = 0;
    if (timer_elapsed32(last_send) > 50) { // Send every 50ms
        send_keyboard_state();
        last_send = timer_read32();
    }
}
```

#### QMK Console Support

Add the following to your `rules.mk`:
```makefile
# Enable console for serial communication
CONSOLE_ENABLE = yes
```

Add this to your `keymap.c`:
```c
#include "print.h"

// Send keyboard state via console
void send_console_state(void) {
    // Send layer and pressed keys
    uprintf("LAYER:%d\n", get_highest_layer(layer_state));
    
    // Send pressed keys
    for (int row = 0; row < MATRIX_ROWS; row++) {
        for (int col = 0; col < MATRIX_COLS; col++) {
            if (matrix_is_on(row, col)) {
                uprintf("PRESS:%d,%d\n", row, col);
            }
        }
    }
    uprintf("END\n");
}

// Call this in your matrix scan function
void matrix_scan_user(void) {
    static uint32_t last_send = 0;
    if (timer_elapsed32(last_send) > 100) { // Send every 100ms
        send_console_state();
        last_send = timer_read32();
    }
}
```

#### Building and Flashing

After adding the configuration:

```bash
# Build your keyboard firmware
qmk compile -kb your_keyboard -km your_keymap

# Flash to your keyboard
qmk flash -kb your_keyboard -km your_keymap
```

**Note**: The application will automatically detect and connect to your keyboard when the appropriate feature is enabled (`--features rawhid` or `--features qmk_console`).

### Interface Features

#### Top Panel
- **Layer Information**: Shows current layer name and number
- **Layer Navigation**: Previous/Next layer buttons (when not using real HID)
- **Mode Indicator**: Shows "Mock" mode for development
- **Control Buttons**:
  - **Textarea**: Toggle text input area for testing
  - **Legend**: Toggle key legend display
  - **Debug**: Show debug information (pressed keys, layer data)
  - **Unload**: Remove current keymap and return to drag & drop zone

#### Central Area
- **Drag & Drop Zone**: Appears when no keymap is loaded
  - Drop keymap files directly onto the zone
  - Click to open file browser
  - Supports `.json`, `.c`, `.h` files
- **Keyboard Display**: Shows loaded keyboard layout
  - Dynamic sizing for any keyboard dimensions
  - Layer switching with visual feedback
  - Key press visualization (when connected to real keyboard)

#### Side Panels (Optional)
- **Textarea Panel**: Text input for testing key mappings
- **Legend Panel**: Shows keycode mappings and translations
- **Debug Panel**: Technical information about keyboard state

## Compatibility Testing

QMK Keyboard Viewer includes comprehensive compatibility testing to ensure it works with keyboards from the QMK firmware repository.

### Running Compatibility Tests

```bash
# 1. Collect keymaps from QMK firmware (requires ~/qmk_firmware)
./scripts/collect-qmk-keymaps.sh

# 2. Run parsing compatibility tests
cargo test qmk_compatibility --release -- --nocapture

# 3. Run UI rendering tests
cargo test ui_rendering --release -- --nocapture

# 4. Generate comprehensive report
cargo test compatibility_report --release -- --nocapture
```

### Test Reports

The compatibility tests generate several reports:

- **`tests/compatibility_report.md`** - Parsing compatibility results
- **`tests/ui_rendering_report.md`** - UI rendering compatibility results  
- **`tests/comprehensive_compatibility_report.md`** - Combined analysis with recommendations

### Test Coverage

The compatibility tests cover:

- **Parsing Tests**: Verify that keymap.c files can be parsed without errors
- **UI Rendering Tests**: Ensure keyboards can be displayed in the UI
- **Performance Tests**: Measure parsing and rendering times
- **Error Analysis**: Categorize and analyze parsing failures

### Requirements

- QMK firmware repository at `~/qmk_firmware`
- At least 4GB RAM for testing large numbers of keyboards
- 10-30 minutes for complete test suite (depending on hardware)

## Project Structure

```
src/
├── keycodes.rs          # QMK keycode mappings
├── keyboard.rs          # Generic keyboard structures
├── keyboards/
│   ├── mod.rs          # Keyboards module
│   └── planck.rs       # Planck-specific defaults
├── assets/
│   └── images/         # Application icons
├── ui.rs               # Dynamic UI
├── main.rs             # Entry point
├── config.rs           # Configuration loading
└── lib.rs              # Module exports

scripts/
├── build-all.sh         # Build for all platforms
├── build-macos.sh       # macOS build with icons (production)
├── build-macos-dev.sh   # macOS build with icons (development - faster)
├── build-linux.sh       # Linux build
├── build-windows.bat    # Windows build (batch)
├── build-windows.ps1    # Windows build (PowerShell)
├── fix-macos-app.sh     # macOS app fix script
└── collect-qmk-keymaps.sh # Collect keymaps from QMK firmware

tests/
├── files/               # Test keymap files
│   ├── test_keymap.json
│   ├── test_keymap.c
│   ├── thooams.json
│   └── *_keymap.c       # Collected QMK keymaps
├── qmk_compatibility.rs # QMK keyboard compatibility tests
├── ui_rendering.rs      # UI rendering compatibility tests
├── compatibility_report.rs # Comprehensive reporting
└── *.rs                 # Other test modules
```

## Build Artifacts

The following files and directories are automatically ignored by git:

- `/dist/` - Build distributions for all platforms
- `/target/` - Rust build artifacts
- `*.icns`, `*.ico` - Platform-specific icon files
- `*.app/`, `*.AppDir/`, `*.exe` - Built applications
- `*.DS_Store` - macOS system files
- Temporary icon files (`qmk-viewer-*.png`)

## Development

### Prerequisites

- Rust (latest stable)
- Platform-specific tools (see BUILD.md)

### Testing

```bash
cargo test
```

### Adding New Keyboards

1. Create a new file in `src/keyboards/`
2. Implement the `Default` trait for your keyboard layout
3. Add the module to `src/keyboards/mod.rs`
4. Update the main application to use your keyboard

## Documentation

- [BUILD.md](BUILD.md) - Detailed build instructions
- [QMK Documentation](https://docs.qmk.fm/) - QMK firmware reference

## License

MIT License - see LICENSE file for details.