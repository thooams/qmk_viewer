//! QMK keycode mapping and translation utilities
//!
//! This module provides comprehensive mapping of QMK keycodes to human-readable
//! labels and symbols, based on the official QMK documentation.

/// Translate a QMK keycode token to a human-readable label
pub fn translate_token(tok: &str) -> String {
    let t = tok.trim();

    // Normalize some malformed keypad tokens that may contain spaces or missing 'K'
    // Examples seen: "KC_KP 0", "KC_P 1", "KC_KP_ 2"
    let mut canonical = t.replace(' ', "");
    let upper = canonical.to_uppercase();
    if upper.starts_with("KC_KP") {
        // Ensure underscore form: KC_KP_<REST>
        let rest = &upper[5..];
        let rest = rest.trim_start_matches('_');
        if !rest.is_empty() {
            canonical = format!("KC_KP_{}", rest);
        }
    } else if upper.starts_with("KC_P") {
        // Accept aliases like KC_P0, KC_P 1 → treat as KC_KP_<REST>
        let rest = &upper[4..];
        if !rest.is_empty() {
            let rest = rest.trim_start_matches('_');
            canonical = format!("KC_KP_{}", rest);
        }
    }
    let t = canonical.as_str();
    if t == "TRNS" || t == "NO" || t == "_______" || t == "KC_TRNS" || t == "KC_NO" {
        return String::new();
    }

    // French accents and specials (KF_* keycodes from the keymap)
    if let Some(result) = translate_french_accents(t) {
        return result;
    }

    // Brackets / punctuation tokens
    if let Some(result) = translate_punctuation(t) {
        return result;
    }

    // Navigation / control
    if let Some(result) = translate_navigation(t) {
        return result;
    }

    // Modifiers / locks
    if let Some(result) = translate_modifiers(t) {
        return result;
    }

    // Basic letter keycodes (KC_A, KC_B, etc.)
    if t.starts_with("KC_") && t.len() == 4 {
        let letter = &t[3..4];
        if letter.chars().next().unwrap().is_ascii_alphabetic() {
            return letter.to_lowercase();
        }
    }

    // Other KC_ keycodes
    if let Some(result) = translate_kc_keycodes(t) {
        return result;
    }

    // Also handle single letter tokens (fallback)
    if t.len() == 1 && t.chars().next().unwrap().is_ascii_alphabetic() {
        return t.to_lowercase();
    }

    // Common icons
    if let Some(result) = translate_icons(t) {
        return result;
    }

    t.to_string()
}

fn translate_french_accents(t: &str) -> Option<String> {
    match t {
        "KF_EGRV" => Some("è".to_string()),
        "KF_EACU" => Some("é".to_string()),
        "KF_ECRC" => Some("ê".to_string()),
        "KF_AGRV" => Some("à".to_string()),
        "KF_UGRV" => Some("ù".to_string()),
        "KF_UCRC" => Some("û".to_string()),
        "KF_ICRC" => Some("î".to_string()),
        "KF_ACRC" => Some("â".to_string()),
        "KF_CCED" => Some("ç".to_string()),
        "KF_DIAE" => Some("¨".to_string()),
        "KF_AE" => Some("æ".to_string()),
        "KF_OE" => Some("œ".to_string()),
        "KF_OCRC" => Some("ô".to_string()),
        "KF_LAQT" => Some("«".to_string()),
        "KF_RAQT" => Some("»".to_string()),
        "KF_LDQT" => Some("\u{201C}".to_string()), // Left double quotation mark
        "KF_RDQT" => Some("\u{201D}".to_string()), // Right double quotation mark
        "KF_MDOT" => Some("·".to_string()),
        "KF_BDOT" => Some("•".to_string()),
        "KF_DEG" => Some("°".to_string()),
        "KF_EURO" => Some("€".to_string()),
        "KF_UNDS" => Some("_".to_string()),
        "KF_SUP2" => Some("²".to_string()),
        "KF_IQES" => Some("¿".to_string()),
        "KF_LARW" => Some("Left".to_string()),
        "KF_RARW" => Some("Right".to_string()),
        "KF_MICR" => Some("μ".to_string()),
        "KF_PSMS" => Some("±".to_string()),
        "KF_CROS" => Some("×".to_string()),
        "KF_QUOT" => Some("'".to_string()),
        "KF_SLCT" => Some("SelAll".to_string()),
        "KF_CUT" => Some("Cut".to_string()),
        "KF_COPY" => Some("Copy".to_string()),
        "KF_PSTE" => Some("Paste".to_string()),
        "KF_SAVE" => Some("Save".to_string()),
        "KF_UNDO" => Some("Undo".to_string()),
        "KF_REDO" => Some("Redo".to_string()),
        // Mappings for keycodes without KF_ prefix (as they appear in the UI)
        "OCRC" => Some("ô".to_string()),
        "ICRC" => Some("î".to_string()),
        "BDOT" => Some("•".to_string()),
        "IQES" => Some("¿".to_string()),
        "LARW" => Some("Left".to_string()),
        "RARW" => Some("Right".to_string()),
        "MDOT" => Some("·".to_string()),
        "DEG" => Some("°".to_string()),
        "UCRC" => Some("û".to_string()),
        "EURO" => Some("€".to_string()),
        "ACRC" => Some("â".to_string()),
        "LDQT" => Some("\u{201C}".to_string()), // Left double quotation mark
        "RDQT" => Some("\u{201D}".to_string()), // Right double quotation mark
        "MICR" => Some("μ".to_string()),
        "PSMS" => Some("±".to_string()),
        "CROS" => Some("×".to_string()),
        // Legacy mappings for compatibility
        "EGRV" => Some("è".to_string()),
        "EACU" => Some("é".to_string()),
        "ECRC" => Some("ê".to_string()),
        "E" => Some("e".to_string()),
        "AGRV" => Some("à".to_string()),
        "UGRV" => Some("ù".to_string()),
        "CCED" => Some("ç".to_string()),
        "DIAE" => Some("¨".to_string()),
        "AE" => Some("æ".to_string()),
        "OE" => Some("œ".to_string()),
        _ => None,
    }
}

fn translate_punctuation(t: &str) -> Option<String> {
    match t {
        "KF_LPRN" | "LPRN" => Some("(".to_string()),
        "KF_RPRN" | "RPRN" => Some(")".to_string()),
        "KF_LBRC" | "LBRC" => Some("[".to_string()),
        "KF_RBRC" | "RBRC" => Some("]".to_string()),
        "KF_LCBR" | "LCBR" => Some("{".to_string()),
        "KF_RCBR" | "RCBR" => Some("}".to_string()),
        "KF_LABK" | "LABK" => Some("<".to_string()),
        "KF_RABK" | "RABK" => Some(">".to_string()),
        "KF_SLSH" | "SLSH" => Some("/".to_string()),
        "KF_BSLS" | "BSLS" => Some("\\".to_string()),
        "KF_PIPE" | "PIPE" => Some("|".to_string()),
        "KF_COLN" | "COLN" => Some(":".to_string()),
        "KF_SCLN" | "SCLN" => Some(";".to_string()),
        "KF_DQUO" | "DQUO" => Some("\"".to_string()),
        "KF_GRV" | "GRV" => Some("`".to_string()),
        "KF_TILD" | "TILD" => Some("~".to_string()),
        "KF_AT" | "AT" => Some("@".to_string()),
        "KF_HASH" | "HASH" => Some("#".to_string()),
        "KF_DLR" | "DLR" => Some("$".to_string()),
        "KF_PERC" | "PERC" => Some("%".to_string()),
        "KF_AMPR" | "AMPR" => Some("&".to_string()),
        "KF_ASTR" | "ASTR" => Some("*".to_string()),
        "KF_EQL" | "EQL" => Some("=".to_string()),
        "KF_PLUS" | "PLUS" => Some("+".to_string()),
        "KF_CIRC" | "CIRC" => Some("^".to_string()),
        "COMM" => Some(",".to_string()),
        "DOT" => Some(".".to_string()),
        "QUOT" => Some("'".to_string()),
        "MINS" => Some("-".to_string()),
        "UNDS" => Some("_".to_string()),
        _ => None,
    }
}

fn translate_navigation(t: &str) -> Option<String> {
    match t {
        "NAV_LCK" => Some("NAV".to_string()),
        "SW_GRV" => Some("`".to_string()),
        "SW_TAB" => Some("Tab".to_string()),
        "CW_TOGG" => Some("Caps".to_string()),
        "OS_LALT" => Some("Alt".to_string()),
        "OS_LGUI" => Some("gui".to_string()),
        "OS_LSFT" => Some("Shift".to_string()),
        "OS_LCTL" => Some("Ctrl".to_string()),
        "OS_RCTL" => Some("Ctrl".to_string()),
        "OS_RSFT" => Some("Shift".to_string()),
        "OS_RGUI" => Some("gui".to_string()),
        "TO(_QWERTY)" => Some("QWERTY".to_string()),
        "KC_PSCR" => Some("PrtSc".to_string()),
        "KC_APP" => Some("Menu".to_string()),
        // Standard navigation
        "LEFT" => Some("Left".to_string()),
        "RGHT" | "RIGHT" => Some("Right".to_string()),
        "UP" => Some("Up".to_string()),
        "DOWN" => Some("Down".to_string()),
        "HOME" => Some("Home".to_string()),
        "END" => Some("End".to_string()),
        "PGUP" | "PG_U" | "PGUPD" => Some("PgUp".to_string()),
        "PGDN" | "PG_D" => Some("PgDn".to_string()),
        "BSPC" => Some("Bksp".to_string()),
        "DEL" => Some("Del".to_string()),
        "ENT" | "ENTER" => Some("Enter".to_string()),
        "ESC" => Some("Esc".to_string()),
        "TAB" => Some("Tab".to_string()),
        "SPC" | "SPACE" => Some("Space".to_string()),
        _ => None,
    }
}

fn translate_modifiers(t: &str) -> Option<String> {
    match t {
        "LSFT" | "RSFT" | "SFT" | "SHIFT" => Some("Shift".to_string()),
        "LCTL" | "RCTL" | "CTL" | "CTRL" | "LCTRL" | "RCTRL" => Some("Ctrl".to_string()),
        "LALT" | "RALT" | "ALT" | "LALT_T" => Some("Alt".to_string()),
        "LGUI" | "RGUI" | "GUI" | "CMD" | "WIN" => Some("gui".to_string()),
        "CAPS" | "CAPSLOCK" => Some("Caps".to_string()),
        _ => None,
    }
}

fn translate_kc_keycodes(t: &str) -> Option<String> {
    match t {
        // Numbers
        "KC_1" => Some("1".to_string()),
        "KC_2" => Some("2".to_string()),
        "KC_3" => Some("3".to_string()),
        "KC_4" => Some("4".to_string()),
        "KC_5" => Some("5".to_string()),
        "KC_6" => Some("6".to_string()),
        "KC_7" => Some("7".to_string()),
        "KC_8" => Some("8".to_string()),
        "KC_9" => Some("9".to_string()),
        "KC_0" => Some("0".to_string()),

        // Special keys
        "KC_SPC" | "KC_SPACE" => Some("Space".to_string()),
        "KC_ENT" | "KC_ENTER" => Some("Enter".to_string()),
        "KC_ESC" => Some("Esc".to_string()),
        "KC_TAB" => Some("Tab".to_string()),
        "KC_BSPC" => Some("Bksp".to_string()),
        "KC_DEL" => Some("Del".to_string()),

        // Navigation
        "KC_LEFT" => Some("Left".to_string()),
        "KC_RGHT" | "KC_RIGHT" => Some("Right".to_string()),
        "KC_UP" => Some("Up".to_string()),
        "KC_DOWN" => Some("Down".to_string()),
        "KC_HOME" => Some("Home".to_string()),
        "KC_END" => Some("End".to_string()),
        "KC_PGUP" | "KC_PG_U" => Some("PgUp".to_string()),
        "KC_PGDN" | "KC_PG_D" => Some("PgDn".to_string()),

        // Modifiers
        "KC_LSFT" | "KC_RSFT" => Some("Shift".to_string()),
        "KC_LCTL" | "KC_RCTL" => Some("Ctrl".to_string()),
        "KC_LALT" | "KC_RALT" => Some("Alt".to_string()),
        "KC_LGUI" | "KC_RGUI" => Some("gui".to_string()),
        "KC_CAPS" | "KC_CAPSLOCK" => Some("Caps".to_string()),

        // Punctuation
        "KC_LPRN" => Some("(".to_string()),
        "KC_RPRN" => Some(")".to_string()),
        "KC_LBRC" => Some("[".to_string()),
        "KC_RBRC" => Some("]".to_string()),
        "KC_LCBR" => Some("{".to_string()),
        "KC_RCBR" => Some("}".to_string()),
        "KC_LABK" => Some("<".to_string()),
        "KC_RABK" => Some(">".to_string()),
        "KC_COMM" => Some(",".to_string()),
        "KC_DOT" => Some(".".to_string()),
        "KC_SLSH" => Some("/".to_string()),
        "KC_BSLS" => Some("\\".to_string()),
        "KC_PIPE" => Some("|".to_string()),
        "KC_COLN" => Some(":".to_string()),
        "KC_SCLN" => Some(";".to_string()),
        "KC_QUOT" => Some("'".to_string()),
        "KC_DQUO" => Some("\"".to_string()),
        "KC_GRV" => Some("`".to_string()),
        "KC_TILD" => Some("~".to_string()),
        "KC_AT" => Some("@".to_string()),
        "KC_HASH" => Some("#".to_string()),
        "KC_DLR" => Some("$".to_string()),
        "KC_PERC" => Some("%".to_string()),
        "KC_AMPR" => Some("&".to_string()),
        "KC_ASTR" => Some("*".to_string()),
        "KC_MINS" => Some("-".to_string()),
        "KC_UNDS" => Some("_".to_string()),
        "KC_EQL" => Some("=".to_string()),
        "KC_PLUS" => Some("+".to_string()),
        "KC_EXLM" => Some("!".to_string()),
        "KC_CIRC" => Some("^".to_string()),

        // Function keys
        "KC_F1" => Some("F1".to_string()),
        "KC_F2" => Some("F2".to_string()),
        "KC_F3" => Some("F3".to_string()),
        "KC_F4" => Some("F4".to_string()),
        "KC_F5" => Some("F5".to_string()),
        "KC_F6" => Some("F6".to_string()),
        "KC_F7" => Some("F7".to_string()),
        "KC_F8" => Some("F8".to_string()),
        "KC_F9" => Some("F9".to_string()),
        "KC_F10" => Some("F10".to_string()),
        "KC_F11" => Some("F11".to_string()),
        "KC_F12" => Some("F12".to_string()),
        "KC_F13" => Some("F13".to_string()),
        "KC_F14" => Some("F14".to_string()),
        "KC_F15" => Some("F15".to_string()),
        "KC_F16" => Some("F16".to_string()),
        "KC_F17" => Some("F17".to_string()),
        "KC_F18" => Some("F18".to_string()),
        "KC_F19" => Some("F19".to_string()),
        "KC_F20" => Some("F20".to_string()),
        "KC_F21" => Some("F21".to_string()),
        "KC_F22" => Some("F22".to_string()),
        "KC_F23" => Some("F23".to_string()),
        "KC_F24" => Some("F24".to_string()),

        // System keys
        "KC_PSCR" => Some("PrtSc".to_string()),
        "KC_APP" => Some("Menu".to_string()),

        // Keypad (numpad) keys
        "KC_KP_0" => Some("0".to_string()),
        "KC_KP_1" => Some("1".to_string()),
        "KC_KP_2" => Some("2".to_string()),
        "KC_KP_3" => Some("3".to_string()),
        "KC_KP_4" => Some("4".to_string()),
        "KC_KP_5" => Some("5".to_string()),
        "KC_KP_6" => Some("6".to_string()),
        "KC_KP_7" => Some("7".to_string()),
        "KC_KP_8" => Some("8".to_string()),
        "KC_KP_9" => Some("9".to_string()),
        "KC_KP_DOT" | "KC_KP_POINT" | "KC_KP_PERIOD" => Some(".".to_string()),
        "KC_KP_COMMA" => Some(",".to_string()),
        "KC_KP_PLUS" => Some("+".to_string()),
        "KC_KP_MINUS" | "KC_KP_SUBTRACT" => Some("-".to_string()),
        "KC_KP_ASTERISK" | "KC_KP_MULTIPLY" => Some("*".to_string()),
        "KC_KP_SLASH" | "KC_KP_DIVIDE" => Some("/".to_string()),
        "KC_KP_ENTER" => Some("Enter".to_string()),
        "KC_KP_EQUAL" | "KC_KP_EQUAL_AS400" => Some("=".to_string()),
        "KC_NUMLOCK" | "KC_NUM" | "KC_LOCKING_NUM" => Some("Num".to_string()),
        _ => None,
    }
}

fn translate_icons(t: &str) -> Option<String> {
    match t {
        "UNDO" => Some("↺".to_string()),
        "REDO" => Some("↻".to_string()),
        "COPY" => Some("⎘".to_string()),
        "CUT" => Some("✂".to_string()),
        "PSTE" | "PASTE" => Some("📋".to_string()),
        "SAVE" => Some("💾".to_string()),
        // Quotes and guillemets tokens
        "LAQT" => Some("«".to_string()),
        "RAQT" => Some("»".to_string()),
        // Superscript 2
        "SUP2" | "SUP" => Some("²".to_string()),
        // Enter explicit
        "ENT" | "ENTER" => Some("Enter".to_string()),
        _ => None,
    }
}

/// Convert modifier token to glyph representation
pub fn mod_to_glyph(m: &str) -> String {
    let mm = m.trim();
    match mm {
        // QMK-style MOD_* constants
        "MOD_LSFT" | "MOD_RSFT" | "MOD_MASK_SHIFT" => "Shift".to_string(),
        "MOD_LCTL" | "MOD_RCTL" | "MOD_MASK_CTRL" => "Ctrl".to_string(),
        "MOD_LALT" | "MOD_RALT" | "MOD_MASK_ALT" => "Alt".to_string(),
        "MOD_LGUI" | "MOD_RGUI" | "MOD_MASK_GUI" => "gui".to_string(),
        // KC_* fallbacks
        "KC_LSFT" | "KC_RSFT" => "Shift".to_string(),
        "KC_LCTL" | "KC_RCTL" => "Ctrl".to_string(),
        "KC_LALT" | "KC_RALT" => "Alt".to_string(),
        "KC_LGUI" | "KC_RGUI" => "gui".to_string(),
        other => translate_token(other),
    }
}

/// Get display name for layer token
pub fn layer_display_name(token: &str) -> String {
    let t = token.trim();
    let friendly = match t {
        "DEF" | "BASE" => "Base",
        "DEF2" => "Base 2",
        "SPC" => "Space",
        "SYM" => "Symbols",
        "SYM_SFT" => "Symbols Shift",
        "NAV" => "Nav",
        "NAV_ALT" => "Nav Alt",
        "NAV_GUI" => "Nav gui",
        "NAV_CTL" => "Nav Ctrl",
        "NUM" => "Num",
        "MOS" => "Mouse",
        other => other,
    };
    friendly.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_letter_keycodes() {
        assert_eq!(translate_token("KC_A"), "a");
        assert_eq!(translate_token("KC_B"), "b");
        assert_eq!(translate_token("KC_Z"), "z");
    }

    #[test]
    fn test_number_keycodes() {
        assert_eq!(translate_token("KC_1"), "1");
        assert_eq!(translate_token("KC_0"), "0");
    }

    #[test]
    fn test_modifier_keycodes() {
        assert_eq!(translate_token("KC_LSFT"), "Shift");
        assert_eq!(translate_token("KC_LCTL"), "Ctrl");
        assert_eq!(translate_token("KC_LALT"), "Alt");
        assert_eq!(translate_token("KC_LGUI"), "gui");
    }

    #[test]
    fn test_navigation_keycodes() {
        assert_eq!(translate_token("KC_LEFT"), "Left");
        assert_eq!(translate_token("KC_RGHT"), "Right");
        assert_eq!(translate_token("KC_UP"), "Up");
        assert_eq!(translate_token("KC_DOWN"), "Down");
    }

    #[test]
    fn test_special_keycodes() {
        assert_eq!(translate_token("KC_ESC"), "Esc");
        assert_eq!(translate_token("KC_TAB"), "Tab");
        assert_eq!(translate_token("KC_BSPC"), "Bksp");
        assert_eq!(translate_token("KC_DEL"), "Del");
    }

    #[test]
    fn test_french_accents() {
        assert_eq!(translate_token("KF_EGRV"), "è");
        assert_eq!(translate_token("KF_EACU"), "é");
        assert_eq!(translate_token("KF_CCED"), "ç");
    }

    #[test]
    fn test_transparent_keys() {
        assert_eq!(translate_token("KC_TRNS"), "");
        assert_eq!(translate_token("KC_NO"), "");
        assert_eq!(translate_token("_______"), "");
        assert_eq!(translate_token("TRNS"), "");
        assert_eq!(translate_token("NO"), "");
    }

    #[test]
    fn test_mod_to_glyph() {
        assert_eq!(mod_to_glyph("MOD_LSFT"), "Shift");
        assert_eq!(mod_to_glyph("MOD_LCTL"), "Ctrl");
        assert_eq!(mod_to_glyph("KC_LALT"), "Alt");
    }

    #[test]
    fn test_layer_display_name() {
        assert_eq!(layer_display_name("DEF"), "Base");
        assert_eq!(layer_display_name("SYM"), "Symbols");
        assert_eq!(layer_display_name("NAV"), "Nav");
    }
}
