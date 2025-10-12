use qmk_viewer::config_persistence::{save_keymap_file, get_saved_keymap_path, clear_saved_keymap};
use std::fs;

#[test]
fn test_config_persistence() {
    // Create a test keymap file
    let test_content = r#"{
        "keyboard": "planck",
        "keymap": "test",
        "layers": [
            ["KC_A", "KC_B", "KC_C"],
            ["KC_1", "KC_2", "KC_3"]
        ],
        "layer_names": ["Base", "Numbers"]
    }"#;
    
    let test_file = "test_drag_drop.json";
    fs::write(test_file, test_content).expect("Failed to write test file");
    
    // Test saving keymap file
    let saved_path = save_keymap_file(test_file).expect("Failed to save keymap file");
    assert!(std::path::Path::new(&saved_path).exists());
    
    // Test getting saved keymap path
    let retrieved_path = get_saved_keymap_path().expect("Failed to get saved keymap path");
    assert!(retrieved_path.is_some());
    assert_eq!(retrieved_path.unwrap(), saved_path);
    
    // Test clearing saved keymap
    clear_saved_keymap().expect("Failed to clear saved keymap");
    
    // Verify it's cleared
    let cleared_path = get_saved_keymap_path().expect("Failed to get saved keymap path after clear");
    assert!(cleared_path.is_none());
    
    // Clean up test file
    fs::remove_file(test_file).ok();
}
