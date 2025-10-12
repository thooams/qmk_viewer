use qmk_viewer::config::KeymapConfig;

#[test]
fn parse_thooams_ok() {
	let cfg = KeymapConfig::load_from_path("tests/files/thooams.json").expect("load json");
	assert!(cfg.layers.len() >= 1);
	for layer in &cfg.layers {
		assert_eq!(layer.len(), 48, "each layer must have 48 entries");
	}
}
