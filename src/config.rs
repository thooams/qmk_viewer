use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KeymapConfig {
	pub keyboard: String,
	pub keymap: String,
	pub layers: Vec<Vec<String>>, // each layer is 48 entries for Planck grid
	pub layout: Option<String>,
	pub layer_names: Option<Vec<String>>, // optional human-friendly names
}

impl KeymapConfig {
	pub fn load_from_path(path: &str) -> anyhow::Result<Self> {
		let data = std::fs::read_to_string(path)?;
		let cfg: Self = serde_json::from_str(&data)?;
		Ok(cfg)
	}
}
