mod hid;
mod planck;
mod ui;
mod config;

use crate::config::KeymapConfig;
use crate::hid::{HidSource, MockHidSource, Report};
use crate::planck::{PlanckKeyboard, PlanckLayoutState};
use crate::ui::PlanckViewerApp;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
	env_logger::init();

	let args: Vec<String> = std::env::args().collect();
	let maybe_json = args.get(1).cloned();
	let maybe_port = args.get(2).cloned(); // optional: explicit serial port

	let (tx, rx) = mpsc::channel::<Report>();

	// Spawn reader thread (mock by default; real when feature enabled)
	thread::spawn(move || {
		#[cfg(feature = "qmk_console")]
		let mut source: Box<dyn HidSource + Send> = {
			let src = crate::hid::QmkConsoleSource::new_with_port(maybe_port);
			Box::new(src)
		};

		#[cfg(all(not(feature = "qmk_console"), feature = "rawhid"))]
		let mut source: Box<dyn HidSource + Send> = Box::new(crate::hid::RawHidSource::new());

		#[cfg(all(not(feature = "qmk_console"), not(feature = "rawhid")))]
		let mut source: Box<dyn HidSource + Send> = Box::new(MockHidSource::new());

		loop {
			if let Some(report) = source.poll() {
				let _ = tx.send(report);
			}
			thread::sleep(Duration::from_millis(8));
		}
	});

	let mut keyboard = PlanckKeyboard::default();
	if let Some(path) = maybe_json {
		if let Ok(cfg) = KeymapConfig::load_from_path(&path) {
			let layer_count = cfg.layers.len().max(1);
			keyboard.layer_names = cfg.layer_names.unwrap_or_else(|| (0..layer_count).map(|i| format!("Layer {}", i)).collect());
			keyboard.raw_legends = cfg.layers.clone();
			keyboard.legends = cfg.layers
				.into_iter()
				.map(|layer| layer.into_iter().map(|s| crate::planck::PlanckLayoutState::normalized_label(&s)).collect())
				.collect();
		}
	}

	let layout_state = PlanckLayoutState::new(keyboard);

	let native_options = eframe::NativeOptions::default();
	let _ = eframe::run_native(
		"QMK Planck Viewer",
		native_options,
		Box::new(move |cc| Ok(Box::new(PlanckViewerApp::new(cc, layout_state.clone(), rx)))),
	);
}
