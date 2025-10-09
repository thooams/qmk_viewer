# QMK Planck Viewer (Rust + egui)

Visualise a Planck keyboard in real-time: pressed keys and active QMK layer.

## Run

- Mock mode (no device needed):

```bash
cargo run
```

- With Raw HID (requires QMK firmware exposing raw HID packets `[layer:u8][pressed_bits:6 bytes le]`):

```bash
cargo run --features rawhid
```

## Tests

```bash
cargo test
```

## QMK Firmware Notes

- Enable `RAW_ENABLE = yes` in your keyboard/keymap `rules.mk`.
- In your QMK code, send a 7-byte packet where `data[0]` is `get_highest_layer(layer_state)` and bytes `[1..7]` contain the first 6 bytes of a little-endian `uint64_t` bitmask of pressed keys (Planck uses 48 keys: 4x12). You can derive bit index as `row*12 + col`.
- Example sketch (C):

```c
void raw_hid_task(void) {
  static uint8_t buf[7];
  buf[0] = get_highest_layer(layer_state);
  uint64_t bits = compute_pressed_bits();
  memcpy(&buf[1], &bits, 6);
  raw_hid_send(buf, 7);
}
```

