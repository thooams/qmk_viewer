use qmk_viewer::keymap_c::parse_keymap_c;

const SAMPLE: &str = r#"
#include QMK_KEYBOARD_H

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
  [0] = LAYOUT(
    KC_Q, KC_W, KC_E, KC_R, KC_T, _______, _______, KC_Y, KC_U, KC_I, KC_O, KC_P,
    KC_A, KC_S, KC_D, KC_F, KC_G, _______, _______, KC_H, KC_J, KC_K, KC_L, KC_SCLN,
    KC_Z, KC_X, KC_C, KC_V, KC_B, _______, _______, KC_N, KC_M, KC_COMM, KC_DOT, KC_SLSH,
    _______, _______, _______, KC_LSFT, KC_SPC, _______, _______, KC_ENT, KC_BSPC, _______, _______, _______
  ),
  [1] = LAYOUT(
    KC_1, KC_2, KC_3, KC_4, KC_5, _______, _______, KC_6, KC_7, KC_8, KC_9, KC_0,
    KC_EXLM, KC_AT, KC_HASH, KC_DLR, KC_PERC, _______, _______, KC_CIRC, KC_AMPR, KC_ASTR, KC_LPRN, KC_RPRN,
    MO(NAV), LT(SYM, KC_EQL), MT(MOD_LALT, KC_TAB), KC_MINS, KC_EQL, _______, _______, KC_PLUS, KC_LCBR, KC_RCBR, KC_LBRC, KC_RBRC,
    _______, _______, _______, _______, _______, _______, _______, _______, _______, _______, _______, _______
  )
};
"#;

#[test]
fn parse_two_layers() {
    let cfg = parse_keymap_c(SAMPLE).expect("parse ok");
    assert_eq!(cfg.layers.len(), 2);
    assert_eq!(cfg.layers[0].len(), 48);
    assert_eq!(cfg.layers[1].len(), 48);
    // Wrapper normalization should pick inner keycode
    assert!(cfg.layers[1].iter().any(|s| s == "KC_TAB"));
}


