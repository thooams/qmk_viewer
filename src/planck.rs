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
		matches!(self.raw_legend_at(layer, row, col), Some(r) if r.trim() == "KC_TRNS")
	}

	pub fn is_function_key(&self, layer: usize, row: usize, col: usize) -> bool {
		match self.raw_legend_at(layer, row, col) {
			Some(r) => {
				let s = r.trim();
				s.starts_with("MO(") || s.starts_with("OSL(") || s.starts_with("TO(") || s.starts_with("DF(")
			}
			None => false,
		}
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
		if let Some(inner) = s.strip_prefix("TO(").and_then(|t| t.strip_suffix(')')) {
			return format!("→{}", inner);
		}
		if let Some(inner) = s.strip_prefix("DF(").and_then(|t| t.strip_suffix(')')) {
			return format!("DF:{}", inner);
		}
		if s.starts_with("KF_") { return s.trim_start_matches("KF_").to_string(); }
		if s.starts_with("KC_") { return s.trim_start_matches("KC_").to_string(); }
		if s.starts_with("OS_") { return s.trim_start_matches("OS_").to_string(); }
		if s.starts_with("SW_") { return s.trim_start_matches("SW_").to_string(); }
		if s.starts_with("ZM_") { return s.trim_start_matches("ZM_").to_string(); }
		s.to_string()
	}
}

