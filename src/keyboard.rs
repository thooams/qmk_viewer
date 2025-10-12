use serde::{Deserialize, Serialize};
use crate::keycodes::{translate_token, mod_to_glyph, layer_display_name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
	pub rows: usize,
	pub cols: usize,
	pub layer_names: Vec<String>,
	pub legends: Vec<Vec<String>>, // [layer][index] normalized label text
	pub raw_legends: Vec<Vec<String>>, // [layer][index] original tokens
}

impl KeyboardLayout {
	/// Create a new keyboard layout with the given dimensions
	pub fn new(rows: usize, cols: usize, layer_names: Vec<String>) -> Self {
		let layer_count = layer_names.len().max(1);
		let total_keys = rows * cols;
		Self {
			rows,
			cols,
			layer_names,
			legends: vec![vec![String::new(); total_keys]; layer_count],
			raw_legends: vec![vec![String::new(); total_keys]; layer_count],
		}
	}

	/// Auto-detect dimensions from layout data and create keyboard layout
	pub fn from_layout_data(layers: Vec<Vec<String>>, layer_names: Option<Vec<String>>) -> Self {
		let layer_count = layers.len().max(1);
		let default_layer_names = (0..layer_count).map(|i| format!("Layer {}", i)).collect();
		let layer_names = layer_names.unwrap_or(default_layer_names);
		
		// Calculate dimensions by finding the maximum number of keys in any layer
		let max_keys = layers.iter().map(|layer| layer.len()).max().unwrap_or(0);
		
		// Try to determine rows/cols from common keyboard layouts
		let (rows, cols) = Self::estimate_dimensions(max_keys);
		
		let total_keys = rows * cols;
		
		// Process layers to normalize keycodes and pad to total_keys
		let processed_layers: Vec<Vec<String>> = layers.iter()
			.map(|layer| {
				let mut processed = layer.iter()
					.map(|s| translate_token(s))
					.collect::<Vec<_>>();
				// Pad with empty strings if needed
				while processed.len() < total_keys {
					processed.push(String::new());
				}
				processed
			})
			.collect();
		
		// Create raw legends (original tokens) with padding
		let raw_layers: Vec<Vec<String>> = layers.iter()
			.map(|layer| {
				let mut raw = layer.clone();
				while raw.len() < total_keys {
					raw.push("_______".to_string());
				}
				raw
			})
			.collect();

		Self {
			rows,
			cols,
			layer_names,
			legends: processed_layers,
			raw_legends: raw_layers,
		}
	}

	/// Estimate keyboard dimensions based on total key count
	fn estimate_dimensions(total_keys: usize) -> (usize, usize) {
		match total_keys {
			0..=20 => (3, 7),      // Small keyboards
			21..=40 => (4, 10),    // 40% keyboards
			41..=50 => (4, 12),    // Planck-like
			51..=60 => (5, 12),    // 60% keyboards
			61..=70 => (5, 14),    // 65% keyboards
			71..=80 => (6, 14),    // 75% keyboards
			81..=90 => (6, 15),    // 80% keyboards
			91..=100 => (6, 17),   // 90% keyboards
			101..=110 => (6, 18),  // 100% keyboards
			_ => {
				// For very large keyboards, try to make a reasonable grid
				let cols = ((total_keys as f64).sqrt().ceil() as usize).max(10);
				let rows = (total_keys + cols - 1) / cols;
				(rows, cols)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct KeyboardState {
	pub keyboard: KeyboardLayout,
	pub active_layer: u8,
	pub pressed_bits: u64,
}

impl KeyboardState {
	pub fn new(keyboard: KeyboardLayout) -> Self {
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
				let mut main = translate_token(parts[1]);
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
				let mut main = translate_token(parts[1]);
				// Apply shift transformation to letters
				if shift_pressed && main.len() == 1 && main.chars().next().unwrap().is_ascii_lowercase() {
					main = main.to_uppercase();
				}
				let layer_tok = parts[0];
				let sub = layer_display_name(layer_tok);
				return (main, sub);
			}
		}
		// MO(layer) / OSL(layer) => main=layer, sub=MO/OSL
		if let Some(inner) = s.strip_prefix("MO(").and_then(|t| t.strip_suffix(')')) {
			let main = layer_display_name(inner);
			return (main, "MO".to_string());
		}
        if let Some(_inner) = s.strip_prefix("OSL(").and_then(|t| t.strip_suffix(')')) {
            // OSL: show only a star, single line (colored in UI)
            return ("★".to_string(), String::new());
		}
		// Default: single label
		let mut main = translate_token(s);
		// Apply shift transformation to letters
		if shift_pressed && main.len() == 1 && main.chars().next().unwrap().is_ascii_lowercase() {
			main = main.to_uppercase();
		}
		(main, String::new())
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_layout_new() {
        let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string(), "Lower".to_string()]);
        assert_eq!(layout.rows, 4);
        assert_eq!(layout.cols, 12);
        assert_eq!(layout.layer_names, vec!["Base", "Lower"]);
        assert_eq!(layout.legends.len(), 2);
        assert_eq!(layout.legends[0].len(), 48); // 4 * 12
    }

    #[test]
    fn test_keyboard_layout_from_data() {
        let layers = vec![
            vec!["KC_A".to_string(), "KC_B".to_string(), "KC_C".to_string()],
            vec!["KC_1".to_string(), "KC_2".to_string(), "KC_3".to_string()],
        ];
        let layout = KeyboardLayout::from_layout_data(layers, None);
        assert_eq!(layout.layer_names, vec!["Layer 0", "Layer 1"]);
        assert_eq!(layout.legends[0][0], "a");
        assert_eq!(layout.legends[1][0], "1");
    }

    #[test]
    fn test_estimate_dimensions() {
        assert_eq!(KeyboardLayout::estimate_dimensions(48), (4, 12)); // Planck-like
        assert_eq!(KeyboardLayout::estimate_dimensions(60), (5, 12)); // 60% keyboard
        assert_eq!(KeyboardLayout::estimate_dimensions(100), (6, 17)); // 100% keyboard
    }

    #[test]
    fn test_keyboard_state() {
        let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
        let state = KeyboardState::new(layout);
        assert_eq!(state.active_layer, 0);
        assert_eq!(state.pressed_bits, 0);
    }

    #[test]
    fn test_index_for() {
        let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
        let state = KeyboardState::new(layout);
        assert_eq!(state.index_for(0, 0), Some(0));
        assert_eq!(state.index_for(1, 0), Some(12));
        assert_eq!(state.index_for(0, 11), Some(11));
        assert_eq!(state.index_for(3, 11), Some(47));
        assert_eq!(state.index_for(4, 0), None); // Out of bounds
        assert_eq!(state.index_for(0, 12), None); // Out of bounds
    }

    #[test]
    fn test_is_pressed() {
        let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
        let mut state = KeyboardState::new(layout);
        state.set_pressed_bits(1); // First key pressed
        assert!(state.is_pressed(0, 0));
        assert!(!state.is_pressed(0, 1));
    }

    #[test]
    fn test_display_parts() {
        let layout = KeyboardLayout::new(4, 12, vec!["Base".to_string()]);
        let mut state = KeyboardState::new(layout);
        state.keyboard.raw_legends[0][0] = "KC_A".to_string();
        let (main, sub) = state.display_parts(0, 0, 0);
        assert_eq!(main, "a");
        assert_eq!(sub, "");
    }
}
