use qmk_viewer::config::KeymapConfig;
use qmk_viewer::keyboard::KeyboardState;
use std::fs;

#[test]
fn test_json_file_loading() {
    // Create a test JSON file
    let test_json = r#"{
        "keyboard": "planck",
        "keymap": "test",
        "layers": [
            ["KC_A", "KC_B", "KC_C"],
            ["KC_1", "KC_2", "KC_3"]
        ],
        "layer_names": ["Base", "Numbers"]
    }"#;
    
    let test_file = "test_file_loading.json";
    fs::write(test_file, test_json).expect("Failed to write test file");
    
    // Test loading the file
    let content = fs::read_to_string(test_file).expect("Failed to read test file");
    let config: KeymapConfig = serde_json::from_str(&content).expect("Failed to parse JSON");
    let keyboard_state = KeyboardState::new(config.to_keyboard_layout());
    
    // Verify the loaded data
    assert_eq!(keyboard_state.keyboard.layer_names.len(), 2);
    assert_eq!(keyboard_state.keyboard.layer_names[0], "Base");
    assert_eq!(keyboard_state.keyboard.layer_names[1], "Numbers");
    assert_eq!(keyboard_state.keyboard.legends[0][0], "a");
    assert_eq!(keyboard_state.keyboard.legends[1][0], "1");
    
    // Clean up
    fs::remove_file(test_file).expect("Failed to remove test file");
}

#[test]
fn test_c_file_loading() {
    // Create a test C file
    let test_c = r#"#include QMK_KEYBOARD_H

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    [0] = LAYOUT_planck_grid(
        KC_A, KC_B, KC_C
    ),
    [1] = LAYOUT_planck_grid(
        KC_1, KC_2, KC_3
    )
};"#;
    
    let test_file = "test_file_loading.c";
    fs::write(test_file, test_c).expect("Failed to write test file");
    
    // Test loading the file
    let content = fs::read_to_string(test_file).expect("Failed to read test file");
    let config = qmk_viewer::keymap_c::parse_keymap_c(&content).expect("Failed to parse C file");
    let keyboard_state = KeyboardState::new(config.to_keyboard_layout());
    
    // Verify the loaded data
    assert_eq!(keyboard_state.keyboard.layer_names.len(), 2);
    assert_eq!(keyboard_state.keyboard.legends[0][0], "a");
    assert_eq!(keyboard_state.keyboard.legends[1][0], "1");
    
    // Clean up
    fs::remove_file(test_file).expect("Failed to remove test file");
}

#[test]
fn test_invalid_file_handling() {
    // Test with non-existent file
    let result = fs::read_to_string("non_existent_file.json");
    assert!(result.is_err());
    
    // Test with invalid JSON
    let invalid_json = "{ invalid json }";
    let result: Result<KeymapConfig, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
    
    // Test with invalid C file
    let invalid_c = "not a valid C file";
    let result = qmk_viewer::keymap_c::parse_keymap_c(invalid_c);
    assert!(result.is_err());
}
