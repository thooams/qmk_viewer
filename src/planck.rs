use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanckKeyboard {
	pub rows: usize,
	pub cols: usize,
	pub layer_names: Vec<String>,
	pub legends: Vec<Vec<String>>, // [layer][index] normalized label text
	pub raw_legends: Vec<Vec<String>>, // [layer][index] original tokens
}

impl Default for PlanckKeyboard {
	fn default() -> Self {
		Self {
			rows: 4,
			cols: 12,
			layer_names: vec![
				"Base".to_string(),
				"Lower".to_string(),
				"Raise".to_string(),
				"Adjust".to_string(),
			],
			legends: vec![vec![String::new(); 48]; 4],
			raw_legends: vec![vec![String::new(); 48]; 4],
		}
	}
}

#[derive(Debug, Clone)]
pub struct PlanckLayoutState {
	pub keyboard: PlanckKeyboard,
	pub active_layer: u8,
	pub pressed_bits: u64,
}

impl PlanckLayoutState {
	pub fn new(keyboard: PlanckKeyboard) -> Self {
		Self { keyboard, active_layer: 0, pressed_bits: 0 }
	}

	pub fn set_layer(&mut self, layer: u8) {
		self.active_layer = layer;
	}

	pub fn set_pressed_bits(&mut self, bits: u64) {
		self.pressed_bits = bits;
	}

	pub fn index_for(&self, row: usize, col: usize) -> Option<usize> {
		if row < self.keyboard.rows && col < self.keyboard.cols {
			Some(row * self.keyboard.cols + col)
		} else {
			None
		}
	}

	pub fn is_pressed(&self, row: usize, col: usize) -> bool {
		match self.index_for(row, col) {
			Some(i) => ((self.pressed_bits >> i) & 1) == 1,
			None => false,
		}
	}

	pub fn legend_at(&self, layer: usize, row: usize, col: usize) -> Option<&str> {
		let idx = self.index_for(row, col)?;
		self.keyboard.legends.get(layer)?.get(idx).map(|s| s.as_str())
	}

	pub fn raw_legend_at(&self, layer: usize, row: usize, col: usize) -> Option<&str> {
		let idx = self.index_for(row, col)?;
		self.keyboard.raw_legends.get(layer)?.get(idx).map(|s| s.as_str())
	}

	pub fn is_transparent_key(&self, layer: usize, row: usize, col: usize) -> bool {
		match self.raw_legend_at(layer, row, col) {
			Some(r) => {
				let t = r.trim();
				t == "KC_TRNS" || t == "KC_NO" || t == "_______" || self.legend_at(layer, row, col).map(|s| s.is_empty()).unwrap_or(false)
			}
			None => false,
		}
	}

	pub fn is_function_key(&self, layer: usize, row: usize, col: usize) -> bool {
		match self.raw_legend_at(layer, row, col) {
			Some(r) => {
				let s = r.trim();
				s.starts_with("MO(") || s.starts_with("OSL(") || s.starts_with("TO(") || s.starts_with("DF(") || s.starts_with("LT(") || s.starts_with("MT(")
			}
			None => false,
		}
	}

	pub fn is_dual_role_key(&self, layer: usize, row: usize, col: usize) -> bool {
		match self.raw_legend_at(layer, row, col) {
			Some(r) => {
				let s = r.trim();
				s.starts_with("LT(") || s.starts_with("MT(")
			}
			None => false,
		}
	}

	pub fn is_mt_key(&self, layer: usize, row: usize, col: usize) -> bool {
		matches!(self.raw_legend_at(layer, row, col), Some(r) if r.trim_start().starts_with("MT("))
	}

	pub fn is_lt_key(&self, layer: usize, row: usize, col: usize) -> bool {
		matches!(self.raw_legend_at(layer, row, col), Some(r) if r.trim_start().starts_with("LT("))
	}

	pub fn is_osl_key(&self, layer: usize, row: usize, col: usize) -> bool {
		matches!(self.raw_legend_at(layer, row, col), Some(r) if r.trim_start().starts_with("OSL("))
	}

	pub fn is_shift_pressed(&self) -> bool {
		// Check if any shift key is pressed by looking for MOD_LSFT, MOD_RSFT, KC_LSFT, KC_RSFT, etc.
		for row in 0..self.keyboard.rows {
			for col in 0..self.keyboard.cols {
				if self.is_pressed(row, col) {
					if let Some(raw) = self.raw_legend_at(self.active_layer as usize, row, col) {
						let s = raw.trim();
						if s.contains("MOD_LSFT") || s.contains("MOD_RSFT") || 
						   s.contains("KC_LSFT") || s.contains("KC_RSFT") ||
						   s.contains("LSFT") || s.contains("RSFT") {
							return true;
						}
					}
				}
			}
		}
		false
	}

	pub fn normalized_label(raw: &str) -> String {
		// Collapse common QMK-style tokens to readable labels
		let s = raw.trim();
		if s == "KC_TRNS" || s == "KC_NO" { return String::new(); }
		if let Some(inner) = s.strip_prefix("MO(").and_then(|t| t.strip_suffix(')')) {
			return inner.to_string();
		}
		if let Some(inner) = s.strip_prefix("OSL(").and_then(|t| t.strip_suffix(')')) {
			return inner.to_string();
		}
		if let Some(inner) = s.strip_prefix("MT(").and_then(|t| t.strip_suffix(')')) {
			// Prefer the keycode part (last arg)
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			return parts.last().unwrap_or(&inner).to_string();
		}
		if let Some(inner) = s.strip_prefix("LT(").and_then(|t| t.strip_suffix(')')) {
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			return parts.last().unwrap_or(&inner).to_string();
		}
		if let Some(inner) = s.strip_prefix("TO(").and_then(|t| t.strip_suffix(')')) {
			return format!("→{}", inner);
		}
		if let Some(inner) = s.strip_prefix("DF(").and_then(|t| t.strip_suffix(')')) {
			return format!("DF:{}", inner);
		}
		if s.starts_with("KF_") { return translate_token(s); }
		if s.starts_with("KC_") { return translate_token(s); }
		if s.starts_with("OS_") { return s.trim_start_matches("OS_").to_string(); }
		if s.starts_with("SW_") { return s.trim_start_matches("SW_").to_string(); }
		if s.starts_with("ZM_") { return s.trim_start_matches("ZM_").to_string(); }
		translate_token(s)
	}

	pub fn display_parts(&self, layer: usize, row: usize, col: usize) -> (String, String) {
		let Some(raw) = self.raw_legend_at(layer, row, col) else { return (String::new(), String::new()); };
		let s = raw.trim();
		// Transparent / empty
		if s == "KC_TRNS" || s == "KC_NO" || s == "_______" { return (String::new(), String::new()); }
		
		let shift_pressed = self.is_shift_pressed();
		
		// MT(mod, key) => main=key, sub=mod glyph
		if let Some(inner) = s.strip_prefix("MT(").and_then(|t| t.strip_suffix(')')) {
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			if parts.len() >= 2 {
				let mut main = Self::normalized_label(parts[1]);
				// Apply shift transformation to letters
				if shift_pressed && main.len() == 1 && main.chars().next().unwrap().is_ascii_lowercase() {
					main = main.to_uppercase();
				}
				let sub = mod_to_glyph(parts[0]);
				return (main, sub);
			}
		}
		// LT(layer, key) => main=key, sub=layer name/token
		if let Some(inner) = s.strip_prefix("LT(").and_then(|t| t.strip_suffix(')')) {
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			if parts.len() >= 2 {
				let mut main = Self::normalized_label(parts[1]);
				// Apply shift transformation to letters
				if shift_pressed && main.len() == 1 && main.chars().next().unwrap().is_ascii_lowercase() {
					main = main.to_uppercase();
				}
				let layer_tok = parts[0];
				let sub = layer_display_name(&self.keyboard, layer_tok);
				return (main, sub);
			}
		}
		// MO(layer) / OSL(layer) => main=layer, sub=MO/OSL
		if let Some(inner) = s.strip_prefix("MO(").and_then(|t| t.strip_suffix(')')) {
			let main = layer_display_name(&self.keyboard, inner);
			return (main, "MO".to_string());
		}
        if let Some(_inner) = s.strip_prefix("OSL(").and_then(|t| t.strip_suffix(')')) {
            // OSL: show only a star, single line (colored in UI)
            return ("★".to_string(), String::new());
		}
		// Default: single label
		let mut main = Self::normalized_label(s);
		// Apply shift transformation to letters
		if shift_pressed && main.len() == 1 && main.chars().next().unwrap().is_ascii_lowercase() {
			main = main.to_uppercase();
		}
		(main, String::new())
	}
}

fn translate_token(tok: &str) -> String {
    // Map special names to glyphs/characters
    let t = tok.trim();
    if t == "TRNS" || t == "NO" || t == "_______" { return String::new(); }
    
    // French accents and specials (KF_* keycodes from the keymap)
    match t {
        "KF_EGRV" => return "è".to_string(),
        "KF_EACU" => return "é".to_string(),
        "KF_ECRC" => return "ê".to_string(),
        "KF_AGRV" => return "à".to_string(),
        "KF_UGRV" => return "ù".to_string(),
        "KF_UCRC" => return "û".to_string(),
        "KF_ICRC" => return "î".to_string(),
        "KF_ACRC" => return "â".to_string(),
        "KF_CCED" => return "ç".to_string(),
        "KF_DIAE" => return "¨".to_string(),
        "KF_AE" => return "æ".to_string(),
        "KF_OE" => return "œ".to_string(),
        "KF_OCRC" => return "ô".to_string(),
        "KF_LAQT" => return "«".to_string(),
        "KF_RAQT" => return "»".to_string(),
        "KF_LDQT" => return "\u{201C}".to_string(), // Left double quotation mark
        "KF_RDQT" => return "\u{201D}".to_string(), // Right double quotation mark
        "KF_MDOT" => return "·".to_string(),
        "KF_BDOT" => return "•".to_string(),
        "KF_DEG" => return "°".to_string(),
        "KF_EURO" => return "€".to_string(),
        "KF_UNDS" => return "_".to_string(),
        "KF_SUP2" => return "²".to_string(),
        "KF_IQES" => return "¿".to_string(),
        "KF_LARW" => return "←".to_string(),
        "KF_RARW" => return "→".to_string(),
        "KF_MICR" => return "μ".to_string(),
        "KF_PSMS" => return "±".to_string(),
        "KF_CROS" => return "×".to_string(),
        "KF_QUOT" => return "'".to_string(),
        "KF_SLCT" => return "⌘A".to_string(),
        "KF_CUT" => return "⌘X".to_string(),
        "KF_COPY" => return "⌘C".to_string(),
        "KF_PSTE" => return "⌘V".to_string(),
        "KF_SAVE" => return "⌘S".to_string(),
        "KF_UNDO" => return "⌘Z".to_string(),
        "KF_REDO" => return "⌘⇧Z".to_string(),
        // Mappings for keycodes without KF_ prefix (as they appear in the UI)
        "OCRC" => return "ô".to_string(),
        "ICRC" => return "î".to_string(),
        "BDOT" => return "•".to_string(),
        "IQES" => return "¿".to_string(),
        "LARW" => return "←".to_string(),
        "RARW" => return "→".to_string(),
        "MDOT" => return "·".to_string(),
        "DEG" => return "°".to_string(),
        "UCRC" => return "û".to_string(),
        "EURO" => return "€".to_string(),
        "ACRC" => return "â".to_string(),
        "LDQT" => return "\u{201C}".to_string(), // Left double quotation mark
        "RDQT" => return "\u{201D}".to_string(), // Right double quotation mark
        "MICR" => return "μ".to_string(),
        "PSMS" => return "±".to_string(),
        "CROS" => return "×".to_string(),
        // Legacy mappings for compatibility
        "EGRV" => return "è".to_string(),
        "EACU" => return "é".to_string(),
        "ECRC" => return "ê".to_string(),
        "E" => return "e".to_string(),
        "AGRV" => return "à".to_string(),
        "UGRV" => return "ù".to_string(),
        "CCED" => return "ç".to_string(),
        "DIAE" => return "¨".to_string(),
        "AE" => return "æ".to_string(),
        "OE" => return "œ".to_string(),
        _ => {}
    }
    // Brackets / punctuation tokens (including KF_* variants from SYM layer)
    match t {
        "KF_LPRN" => return "(".to_string(),
        "KF_RPRN" => return ")".to_string(),
        "KF_LBRC" => return "[".to_string(),
        "KF_RBRC" => return "]".to_string(),
        "KF_LCBR" => return "{".to_string(),
        "KF_RCBR" => return "}".to_string(),
        "KF_LABK" => return "<".to_string(),
        "KF_RABK" => return ">".to_string(),
        "KF_SLSH" => return "/".to_string(),
        "KF_BSLS" => return "\\".to_string(),
        "KF_PIPE" => return "|".to_string(),
        "KF_COLN" => return ":".to_string(),
        "KF_SCLN" => return ";".to_string(),
        "KF_DQUO" => return "\"".to_string(),
        "KF_GRV" => return "`".to_string(),
        "KF_TILD" => return "~".to_string(),
        "KF_AT" => return "@".to_string(),
        "KF_HASH" => return "#".to_string(),
        "KF_DLR" => return "$".to_string(),
        "KF_PERC" => return "%".to_string(),
        "KF_AMPR" => return "&".to_string(),
        "KF_ASTR" => return "*".to_string(),
        "KF_EQL" => return "=".to_string(),
        "KF_PLUS" => return "+".to_string(),
        "KF_CIRC" => return "^".to_string(),
        // Legacy mappings for compatibility
        "LPRN" => return "(".to_string(),
        "RPRN" => return ")".to_string(),
        "LBRC" => return "[".to_string(),
        "RBRC" => return "]".to_string(),
        "LCBR" => return "{".to_string(),
        "RCBR" => return "}".to_string(),
        "LABK" => return "<".to_string(),
        "RABK" => return ">".to_string(),
        "COMM" => return ",".to_string(),
        "DOT" => return ".".to_string(),
        "SLSH" => return "/".to_string(),
        "BSLS" => return "\\".to_string(),
        "PIPE" => return "|".to_string(),
        "COLN" => return ":".to_string(),
        "SCLN" => return ";".to_string(),
        "QUOT" => return "'".to_string(),
        "DQUO" => return "\"".to_string(),
        "GRV" => return "`".to_string(),
        "TILD" => return "~".to_string(),
        "AT" => return "@".to_string(),
        "HASH" => return "#".to_string(),
        "DLR" => return "$".to_string(),
        "PERC" => return "%".to_string(),
        "AMPR" => return "&".to_string(),
        "ASTR" => return "*".to_string(),
        "MINS" => return "-".to_string(),
        "UNDS" => return "_".to_string(),
        "EQL" => return "=".to_string(),
        "PLUS" => return "+".to_string(),
        _ => {}
    }
    // Navigation / control glyphs (including special keycodes from NAV layer)
    match t {
        "NAV_LCK" => return "NAV".to_string(),
        "SW_GRV" => return "`".to_string(),
        "SW_TAB" => return "⇥".to_string(),
        "CW_TOGG" => return "Caps".to_string(),
        "OS_LALT" => return "⌥".to_string(),
        "OS_LGUI" => return "⌘".to_string(),
        "OS_LSFT" => return "⇧".to_string(),
        "OS_LCTL" => return "⌃".to_string(),
        "OS_RCTL" => return "⌃".to_string(),
        "OS_RSFT" => return "⇧".to_string(),
        "OS_RGUI" => return "⌘".to_string(),
        "TO(_QWERTY)" => return "QWERTY".to_string(),
        "KC_PSCR" => return "PrtSc".to_string(),
        "KC_APP" => return "Menu".to_string(),
        // Standard navigation
        "LEFT" => return "←".to_string(),
        "RGHT" | "RIGHT" => return "→".to_string(),
        "UP" => return "↑".to_string(),
        "DOWN" => return "↓".to_string(),
        "HOME" => return "⇱".to_string(),
        "END" => return "⇲".to_string(),
        "PGUP" | "PG_U" | "PGUPD" => return "⇞".to_string(),
        "PGDN" | "PG_D" => return "⇟".to_string(),
        "BSPC" => return "⌫".to_string(),
        "DEL" => return "⌦".to_string(),
        "ENT" | "ENTER" => return "↩".to_string(),
        "ESC" => return "⎋".to_string(),
        "TAB" => return "⇥".to_string(),
        "SPC" | "SPACE" => return "␣".to_string(),
        _ => {}
    }
    // Modifiers / locks
    match t {
        "LSFT" | "RSFT" | "SFT" | "SHIFT" => return "⇧".to_string(),
        "LCTL" | "RCTL" | "CTL" | "CTRL" | "LCTRL" | "RCTRL" => return "⌃".to_string(),
        "LALT" | "RALT" | "ALT" | "LALT_T" => return "⌥".to_string(),
        "LGUI" | "RGUI" | "GUI" | "CMD" | "WIN" => return "⌘".to_string(),
        "CAPS" | "CAPSLOCK" => return "⇪".to_string(),
        _ => {}
    }
    // Basic letter keycodes (KC_A, KC_B, etc.)
    if t.starts_with("KC_") && t.len() == 4 {
        let letter = &t[3..4];
        if letter.chars().next().unwrap().is_ascii_alphabetic() {
            return letter.to_lowercase();
        }
    }
    
    // Other KC_ keycodes
    match t {
        "KC_1" => return "1".to_string(),
        "KC_2" => return "2".to_string(),
        "KC_3" => return "3".to_string(),
        "KC_4" => return "4".to_string(),
        "KC_5" => return "5".to_string(),
        "KC_6" => return "6".to_string(),
        "KC_7" => return "7".to_string(),
        "KC_8" => return "8".to_string(),
        "KC_9" => return "9".to_string(),
        "KC_0" => return "0".to_string(),
        "KC_SPC" | "KC_SPACE" => return "␣".to_string(),
        "KC_ENT" | "KC_ENTER" => return "↩".to_string(),
        "KC_ESC" => return "⎋".to_string(),
        "KC_TAB" => return "⇥".to_string(),
        "KC_BSPC" => return "⌫".to_string(),
        "KC_DEL" => return "⌦".to_string(),
        "KC_LEFT" => return "←".to_string(),
        "KC_RGHT" | "KC_RIGHT" => return "→".to_string(),
        "KC_UP" => return "↑".to_string(),
        "KC_DOWN" => return "↓".to_string(),
        "KC_HOME" => return "⇱".to_string(),
        "KC_END" => return "⇲".to_string(),
        "KC_PGUP" | "KC_PG_U" => return "⇞".to_string(),
        "KC_PGDN" | "KC_PG_D" => return "⇟".to_string(),
        "KC_LSFT" | "KC_RSFT" => return "⇧".to_string(),
        "KC_LCTL" | "KC_RCTL" => return "⌃".to_string(),
        "KC_LALT" | "KC_RALT" => return "⌥".to_string(),
        "KC_LGUI" | "KC_RGUI" => return "⌘".to_string(),
        "KC_CAPS" | "KC_CAPSLOCK" => return "⇪".to_string(),
        "KC_LPRN" => return "(".to_string(),
        "KC_RPRN" => return ")".to_string(),
        "KC_LBRC" => return "[".to_string(),
        "KC_RBRC" => return "]".to_string(),
        "KC_LCBR" => return "{".to_string(),
        "KC_RCBR" => return "}".to_string(),
        "KC_LABK" => return "<".to_string(),
        "KC_RABK" => return ">".to_string(),
        "KC_COMM" => return ",".to_string(),
        "KC_DOT" => return ".".to_string(),
        "KC_SLSH" => return "/".to_string(),
        "KC_BSLS" => return "\\".to_string(),
        "KC_PIPE" => return "|".to_string(),
        "KC_COLN" => return ":".to_string(),
        "KC_SCLN" => return ";".to_string(),
        "KC_QUOT" => return "'".to_string(),
        "KC_DQUO" => return "\"".to_string(),
        "KC_GRV" => return "`".to_string(),
        "KC_TILD" => return "~".to_string(),
        "KC_AT" => return "@".to_string(),
        "KC_HASH" => return "#".to_string(),
        "KC_DLR" => return "$".to_string(),
        "KC_PERC" => return "%".to_string(),
        "KC_AMPR" => return "&".to_string(),
        "KC_ASTR" => return "*".to_string(),
        "KC_MINS" => return "-".to_string(),
        "KC_UNDS" => return "_".to_string(),
        "KC_EQL" => return "=".to_string(),
        "KC_PLUS" => return "+".to_string(),
        "KC_EXLM" => return "!".to_string(),
        "KC_CIRC" => return "^".to_string(),
        "KC_F1" => return "F1".to_string(),
        "KC_F2" => return "F2".to_string(),
        "KC_F3" => return "F3".to_string(),
        "KC_F4" => return "F4".to_string(),
        "KC_F5" => return "F5".to_string(),
        "KC_F6" => return "F6".to_string(),
        "KC_F7" => return "F7".to_string(),
        "KC_F8" => return "F8".to_string(),
        "KC_F9" => return "F9".to_string(),
        "KC_F10" => return "F10".to_string(),
        "KC_F11" => return "F11".to_string(),
        "KC_F12" => return "F12".to_string(),
        "KC_PSCR" => return "PrtSc".to_string(),
        "KC_APP" => return "Menu".to_string(),
        _ => {}
    }
    
    // Also handle single letter tokens (fallback)
    if t.len() == 1 && t.chars().next().unwrap().is_ascii_alphabetic() {
        return t.to_lowercase();
    }
    
    // Common icons
    match t {
        "UNDO" => return "↺".to_string(),
        "REDO" => return "↻".to_string(),
        "COPY" => return "⎘".to_string(),
        "CUT" => return "✂".to_string(),
        "PSTE" | "PASTE" => return "📋".to_string(),
        "SAVE" => return "💾".to_string(),
        // Quotes and guillemets tokens
        "LAQT" => return "«".to_string(),
        "RAQT" => return "»".to_string(),
        // Superscript 2
        "SUP2" | "SUP" => return "²".to_string(),
        // Enter explicit
        "ENT" | "ENTER" => return "↩".to_string(),
        _ => {}
    }
    sanitize_glyphs(t)
}

fn mod_to_glyph(m: &str) -> String {
    let mm = m.trim();
    let g = match mm {
        // QMK-style MOD_* constants
        "MOD_LSFT" | "MOD_RSFT" | "MOD_MASK_SHIFT" => "⇧".to_string(),
        "MOD_LCTL" | "MOD_RCTL" | "MOD_MASK_CTRL" => "⌃".to_string(),
        "MOD_LALT" | "MOD_RALT" | "MOD_MASK_ALT" => "⌥".to_string(),
        "MOD_LGUI" | "MOD_RGUI" | "MOD_MASK_GUI" => "⌘".to_string(),
        // KC_* fallbacks
        "KC_LSFT" | "KC_RSFT" => "⇧".to_string(),
        "KC_LCTL" | "KC_RCTL" => "⌃".to_string(),
        "KC_LALT" | "KC_RALT" => "⌥".to_string(),
        "KC_LGUI" | "KC_RGUI" => "⌘".to_string(),
        other => translate_token(other),
    };
    sanitize_glyphs(&g)
}

fn layer_display_name(kbd: &PlanckKeyboard, token: &str) -> String {
    // If token matches an existing layer name, keep it; otherwise, return token
    let t = token.trim();
    let friendly = match t {
        "DEF" | "BASE" => "Base",
        "DEF2" => "Base 2",
        "SPC" => "Space",
        "SYM" => "Symbols",
        "SYM_SFT" => "Symbols Shift",
        "NAV" => "Nav",
        "NAV_ALT" => "Nav Alt",
        "NAV_GUI" => "Nav GUI",
        "NAV_CTL" => "Nav Ctrl",
        "NUM" => "Num",
        "MOS" => "Mouse",
        other => other,
    };
    if kbd.layer_names.iter().any(|n| n.eq_ignore_ascii_case(friendly)) {
        friendly.to_string()
    } else {
        friendly.to_string()
    }
}

fn sanitize_glyphs(s: &str) -> String {
    // Replace uncommon glyphs with ASCII fallbacks to avoid tofu squares
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let repl = match ch {
            // macOS modifiers and UI arrows
            '⌘' => "CMD",
            '⌥' => "ALT",
            '⌃' => "CTRL",
            '⇧' => "SHIFT",
            '⇪' => "CAPS",
            '←' => "<",
            '→' => ">",
            '↑' => "^",
            '↓' => "v",
            '⇱' => "Home",
            '⇲' => "End",
            '⇞' => "PgUp",
            '⇟' => "PgDn",
            '⌫' => "Bksp",
            '⌦' => "Del",
            '↩' => "Enter",
            '⎋' => "Esc",
            '⇥' => "Tab",
            '␣' => "Space",
            // Geometric placeholders that render as tofu
            '□' | '■' | '◻' | '◼' | '▢' | '▣' => "",
            // Keep common Latin accents we intend to show
            c if c == 'é' || c == 'è' || c == 'ê' || c == 'à' || c == 'ù' || c == 'ç' || c == 'æ' || c == 'œ' => {
                out.push(c);
                continue;
            }
            _ => "",
        };
        if repl.is_empty() {
            // keep ASCII printable
            if ch.is_ascii() {
                out.push(ch);
            }
        } else {
            if !out.is_empty() { out.push(' '); }
            out.push_str(repl);
        }
    }
    out
}

