use serde::Deserialize;
use anyhow::Context;
use crate::keyboard::KeyboardLayout;

#[derive(Debug, Deserialize)]
pub struct KeymapConfig {
	pub keyboard: String,
	pub keymap: String,
	pub layers: Vec<Vec<String>>, // each layer contains keycodes for the keyboard
	pub layout: Option<String>,
	pub layer_names: Option<Vec<String>>, // optional human-friendly names
}

impl KeymapConfig {
	pub fn load_from_path(path: &str) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path))?;
        if path.ends_with(".json") {
            let cfg: Self = serde_json::from_str(&data)
                .with_context(|| format!("failed to parse JSON: {}", path))?;
            return Ok(cfg);
        }
        if path.ends_with(".c") {
            return crate::keymap_c::parse_keymap_c(&data)
                .with_context(|| format!("failed to parse keymap.c: {}", path));
        }
        anyhow::bail!("unsupported config format (expected .json or .c): {}", path)
	}

	/// Convert this keymap configuration to a generic keyboard layout
	pub fn to_keyboard_layout(&self) -> KeyboardLayout {
		KeyboardLayout::from_layout_data(self.layers.clone(), self.layer_names.clone())
	}
}
