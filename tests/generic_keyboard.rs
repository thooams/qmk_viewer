use qmk_viewer::config::KeymapConfig;
use qmk_viewer::keyboard::{KeyboardLayout, KeyboardState};

#[test]
fn test_different_keyboard_sizes() {
    // Test 60% keyboard (5x12)
    let layers_60 = vec![vec!["KC_A".to_string(); 60], vec!["KC_B".to_string(); 60]];
    let layout_60 = KeyboardLayout::from_layout_data(layers_60, None);
    assert_eq!(layout_60.rows, 5);
    assert_eq!(layout_60.cols, 12);
    assert_eq!(layout_60.legends[0].len(), 60);

    // Test 100% keyboard (6x18)
    let layers_100 = vec![vec!["KC_A".to_string(); 108], vec!["KC_B".to_string(); 108]];
    let layout_100 = KeyboardLayout::from_layout_data(layers_100, None);
    assert_eq!(layout_100.rows, 6);
    assert_eq!(layout_100.cols, 18);
    assert_eq!(layout_100.legends[0].len(), 108);
}

#[test]
fn test_keyboard_state_with_different_sizes() {
    // Test with 3x10 keyboard
    let layout = KeyboardLayout::new(3, 10, vec!["Base".to_string()]);
    let mut state = KeyboardState::new(layout);

    // Test index calculation
    assert_eq!(state.index_for(0, 0), Some(0));
    assert_eq!(state.index_for(1, 0), Some(10));
    assert_eq!(state.index_for(2, 9), Some(29));
    assert_eq!(state.index_for(3, 0), None); // Out of bounds
    assert_eq!(state.index_for(0, 10), None); // Out of bounds

    // Test key press detection
    state.set_pressed_bits(1 << 15); // Press key at row 1, col 5
    assert!(state.is_pressed(1, 5));
    assert!(!state.is_pressed(0, 0));
}

#[test]
fn test_keycode_mapping_comprehensive() {
    let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
    let mut state = KeyboardState::new(layout);

    // Set up some test keycodes
    state.keyboard.raw_legends[0][0] = "KC_A".to_string();
    state.keyboard.raw_legends[0][1] = "MO(1)".to_string();
    state.keyboard.raw_legends[0][2] = "KC_LSFT".to_string();
    state.keyboard.raw_legends[0][3] = "KC_ESC".to_string();
    state.keyboard.raw_legends[0][4] = "MT(MOD_LCTL, KC_SPC)".to_string();
    state.keyboard.raw_legends[0][5] = "LT(1, KC_TAB)".to_string();
    state.keyboard.raw_legends[0][6] = "MO(2)".to_string();
    state.keyboard.raw_legends[0][7] = "OSL(3)".to_string();

    // Test display parts for various keycode types
    let (main, sub) = state.display_parts(0, 0, 0);
    assert_eq!(main, "a");
    assert_eq!(sub, "");

    let (main, sub) = state.display_parts(0, 0, 1);
    assert_eq!(main, "1");
    assert_eq!(sub, "MO");

    let (main, sub) = state.display_parts(0, 0, 2);
    assert_eq!(main, "Shift");
    assert_eq!(sub, "");

    let (main, sub) = state.display_parts(0, 0, 3);
    assert_eq!(main, "Esc");
    assert_eq!(sub, "");

    let (main, sub) = state.display_parts(0, 0, 4);
    assert_eq!(main, "Space");
    assert_eq!(sub, "Ctrl");

    let (main, sub) = state.display_parts(0, 0, 5);
    assert_eq!(main, "Tab");
    assert_eq!(sub, "1");

    let (main, sub) = state.display_parts(0, 0, 1);
    assert_eq!(main, "1");
    assert_eq!(sub, "MO");

    let (main, sub) = state.display_parts(0, 0, 7);
    assert_eq!(main, "★");
    assert_eq!(sub, "");
}

#[test]
fn test_keyboard_function_detection() {
    let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
    let mut state = KeyboardState::new(layout);

    // Set up test keycodes
    state.keyboard.raw_legends[0][0] = "KC_A".to_string();
    state.keyboard.raw_legends[0][1] = "MO(1)".to_string();
    state.keyboard.raw_legends[0][2] = "LT(1, KC_TAB)".to_string();
    state.keyboard.raw_legends[0][3] = "MT(MOD_LCTL, KC_C)".to_string();
    state.keyboard.raw_legends[0][4] = "OSL(3)".to_string();
    state.keyboard.raw_legends[0][5] = "KC_TRNS".to_string();

    // Also update the processed legends
    state.keyboard.legends[0][0] = "a".to_string();
    state.keyboard.legends[0][1] = "Layer 0".to_string();
    state.keyboard.legends[0][2] = "b".to_string();
    state.keyboard.legends[0][3] = "c".to_string();
    state.keyboard.legends[0][4] = "★".to_string();
    state.keyboard.legends[0][5] = "".to_string();

    // Test function key detection
    assert!(!state.is_function_key(0, 0, 0)); // KC_A
    assert!(state.is_function_key(0, 0, 1)); // MO(1)
    assert!(state.is_function_key(0, 0, 2)); // LT(2, KC_B)
    assert!(state.is_function_key(0, 0, 3)); // MT(MOD_LCTL, KC_C)
    assert!(state.is_function_key(0, 0, 4)); // OSL(3)
    assert!(!state.is_function_key(0, 0, 5)); // KC_TRNS

    // Test dual role key detection
    assert!(!state.is_dual_role_key(0, 0, 0)); // KC_A
    assert!(!state.is_dual_role_key(0, 0, 1)); // MO(1)
    assert!(state.is_dual_role_key(0, 0, 2)); // LT(2, KC_B)
    assert!(state.is_dual_role_key(0, 0, 3)); // MT(MOD_LCTL, KC_C)
    assert!(!state.is_dual_role_key(0, 0, 4)); // OSL(3)

    // Test specific function types
    assert!(state.is_mt_key(0, 0, 3)); // MT(MOD_LCTL, KC_C)
    assert!(state.is_lt_key(0, 0, 2)); // LT(2, KC_B)
    assert!(state.is_osl_key(0, 0, 4)); // OSL(3)

    // Test transparent key detection
    assert!(!state.is_transparent_key(0, 0, 0)); // KC_A
    assert!(state.is_transparent_key(0, 0, 5)); // KC_TRNS
}

#[test]
fn test_config_to_keyboard_layout() {
    let config = KeymapConfig {
        keyboard: "test".to_string(),
        keymap: "test".to_string(),
        layers: vec![
            vec!["KC_A".to_string(), "KC_B".to_string(), "KC_C".to_string()],
            vec!["KC_1".to_string(), "KC_2".to_string(), "KC_3".to_string()],
        ],
        layout: None,
        layer_names: Some(vec!["QWERTY".to_string(), "NUMBERS".to_string()]),
    };

    let layout = config.to_keyboard_layout();
    assert_eq!(layout.layer_names, vec!["QWERTY", "NUMBERS"]);
    assert_eq!(layout.legends[0][0], "a");
    assert_eq!(layout.legends[1][0], "1");
    // Should auto-detect dimensions (3 keys -> 3x1 or similar)
    assert!(layout.rows > 0);
    assert!(layout.cols > 0);
}
