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
		if s.starts_with("KF_") { return translate_token(s.trim_start_matches("KF_")); }
		if s.starts_with("KC_") { return translate_token(s.trim_start_matches("KC_")); }
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
		// MT(mod, key) => main=key, sub=mod glyph
		if let Some(inner) = s.strip_prefix("MT(").and_then(|t| t.strip_suffix(')')) {
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			if parts.len() >= 2 {
				let main = Self::normalized_label(parts[1]);
				let sub = mod_to_glyph(parts[0]);
				return (main, sub);
			}
		}
		// LT(layer, key) => main=key, sub=layer name/token
		if let Some(inner) = s.strip_prefix("LT(").and_then(|t| t.strip_suffix(')')) {
			let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
			if parts.len() >= 2 {
				let main = Self::normalized_label(parts[1]);
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
		(Self::normalized_label(s), String::new())
	}
}

fn translate_token(tok: &str) -> String {
    // Map special names to glyphs/characters
    let t = tok.trim();
    if t == "TRNS" || t == "NO" || t == "_______" { return String::new(); }
    // French accents and specials
    match t {
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
    // Brackets / punctuation tokens
    match t {
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
    // Navigation / control glyphs
    match t {
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

