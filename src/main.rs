use qmk_viewer::config::KeymapConfig;
use qmk_viewer::config_persistence::get_saved_keymap_path;
#[cfg(feature = "qmk_console")]
use qmk_viewer::hid::QmkConsoleSource;
#[cfg(feature = "rawhid")]
use qmk_viewer::hid::RawHidSource;
use qmk_viewer::hid::{HidSource, Report};
use qmk_viewer::keyboard::KeyboardState;
use qmk_viewer::keyboards::planck::PlanckLayout;
use qmk_viewer::ui::KeyboardViewerApp;

use egui::{IconData, ViewportBuilder};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let maybe_json = args.get(1).cloned();
    let _maybe_port = args.get(2).cloned(); // optional: explicit serial port

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
        let mut source: Box<dyn HidSource + Send> = Box::new(qmk_viewer::hid::MockHidSource::new());

        loop {
            if let Some(report) = source.poll() {
                let _ = tx.send(report);
            }
            thread::sleep(Duration::from_millis(8));
        }
    });

    let mut keyboard = PlanckLayout::planck_default();
    let mut keyboard_loaded = false;

    // Try to load from command line argument first
    if let Some(path) = maybe_json {
        if let Ok(cfg) = KeymapConfig::load_from_path(&path) {
            keyboard = cfg.to_keyboard_layout();
            keyboard_loaded = true;
        }
    } else {
        // Try to load from saved keymap
        if let Ok(Some(saved_path)) = get_saved_keymap_path() {
            if let Ok(cfg) = KeymapConfig::load_from_path(&saved_path) {
                keyboard = cfg.to_keyboard_layout();
                keyboard_loaded = true;
            }
        }
    }

    let layout_state = KeyboardState::new(keyboard);

    // Load the application icon
    let icon_data = include_bytes!("assets/images/qmk-viewer.png");
    let icon = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    let icon_rgba = icon.into_raw();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_icon(IconData {
            rgba: icon_rgba,
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "QMK Keyboard Viewer",
        native_options,
        Box::new(move |cc| {
            let mut app = KeyboardViewerApp::new(cc, layout_state.clone(), rx);
            app.set_keyboard_loaded(keyboard_loaded);
            Ok(Box::new(app))
        }),
    );
}
