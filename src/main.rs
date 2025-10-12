use qmk_viewer::config::KeymapConfig;
use qmk_viewer::hid::{HidSource, MockHidSource, Report};
#[cfg(feature = "rawhid")]
use qmk_viewer::hid::RawHidSource;
#[cfg(feature = "qmk_console")]
use qmk_viewer::hid::QmkConsoleSource;
use qmk_viewer::keyboard::KeyboardState;
use qmk_viewer::keyboards::planck::PlanckLayout;
use qmk_viewer::ui::KeyboardViewerApp;

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
            let src = QmkConsoleSource::new_with_port(maybe_port);
            Box::new(src)
        };

        #[cfg(all(not(feature = "qmk_console"), feature = "rawhid"))]
        let mut source: Box<dyn HidSource + Send> = Box::new(RawHidSource::new());

		#[cfg(all(not(feature = "qmk_console"), not(feature = "rawhid")))]
		let mut source: Box<dyn HidSource + Send> = Box::new(MockHidSource::new());

		loop {
			if let Some(report) = source.poll() {
				let _ = tx.send(report);
			}
			thread::sleep(Duration::from_millis(8));
		}
	});

	let mut keyboard = PlanckLayout::default();
	if let Some(path) = maybe_json {
		if let Ok(cfg) = KeymapConfig::load_from_path(&path) {
			keyboard = cfg.to_keyboard_layout();
		}
	}

	let layout_state = KeyboardState::new(keyboard);

	let native_options = eframe::NativeOptions::default();
	let _ = eframe::run_native(
		"QMK Keyboard Viewer",
		native_options,
		Box::new(move |cc| Ok(Box::new(KeyboardViewerApp::new(cc, layout_state.clone(), rx)))),
	);
}
