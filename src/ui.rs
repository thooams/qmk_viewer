use crate::hid::Report;
use crate::keyboard::KeyboardState;
use crate::config::KeymapConfig;
use eframe::egui::{self, Color32, Context, RichText, Sense, Vec2};

// Catppuccin Mocha palette (subset)
struct Palette;
impl Palette {
    const BLUE: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa);
    const PEACH: Color32 = Color32::from_rgb(0xfa, 0xb3, 0x87);
    const YELLOW: Color32 = Color32::from_rgb(0xf9, 0xe2, 0xaf);
    const GREEN: Color32 = Color32::from_rgb(0xa6, 0xe3, 0xa1);
    const _SURFACE: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2e); // base
    const OVERLAY: Color32 = Color32::from_rgb(0x31, 0x31, 0x41); // overlay0
    const TEXT: Color32 = Color32::from_rgb(0xc6, 0xd0, 0xf5);
}
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct KeyboardViewerApp {
	state: KeyboardState,
	rx: Receiver<Report>,
    show_debug: bool,
    show_legend: bool,
    show_textarea: bool,
    pressed_started: HashMap<usize, Instant>,
    text_input: String,
    file_path: String,
    file_loading: bool,
    file_status: String,
	#[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
	manual_pressed: std::collections::HashSet<usize>,
}

impl KeyboardViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: KeyboardState, rx: Receiver<Report>) -> Self {
        // Try to install symbol-capable fonts so glyphs render (macOS paths first)
        let mut fonts = egui::FontDefinitions::default();
        // Known macOS fonts with symbols/emoji
        let candidates = [
            "/System/Library/Fonts/Apple Symbols.ttf",
            "/System/Library/Fonts/Supplemental/Apple Symbols.ttf",
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            "/System/Library/Fonts/Supplemental/NotoSansSymbols2-Regular.ttf",
            "/System/Library/Fonts/Supplemental/NotoSansSymbols-Regular.ttf",
        ];
        for path in candidates.iter() {
            if let Ok(bytes) = std::fs::read(path) {
                let key = format!("userfont:{}", path);
                fonts.font_data.insert(key.clone(), egui::FontData::from_owned(bytes).into());
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, key.clone());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, key);
            }
        }
        cc.egui_ctx.set_fonts(fonts);

        Self {
            state,
            rx,
            show_debug: false,
            show_legend: false,
            show_textarea: false,
            pressed_started: HashMap::new(),
            text_input: String::new(),
            file_path: String::new(),
            file_loading: false,
            file_status: String::new(),
            #[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
            manual_pressed: std::collections::HashSet::new(),
        }
    }
    
    fn load_keymap_file(&mut self) {
        if self.file_path.is_empty() {
            return;
        }
        
        self.file_loading = true;
        
        // Try to load the file
        match std::fs::read_to_string(&self.file_path) {
            Ok(content) => {
                // Determine file type by extension
                let is_json = self.file_path.ends_with(".json");
                let is_c = self.file_path.ends_with(".c") || self.file_path.ends_with(".h");
                
                if is_json {
                    // Try to parse as JSON
                    match serde_json::from_str::<KeymapConfig>(&content) {
                        Ok(config) => {
                            // Update the keyboard state with new layout
                            self.state = KeyboardState::new(config.to_keyboard_layout());
                            self.file_status = format!("✅ Loaded JSON keymap: {} layers", config.layers.len());
                            println!("✅ Successfully loaded JSON keymap from: {}", self.file_path);
                        }
                        Err(e) => {
                            self.file_status = format!("❌ JSON parse error: {}", e);
                            eprintln!("❌ Failed to parse JSON keymap: {}", e);
                        }
                    }
                } else if is_c {
                    // Try to parse as C keymap
                    match crate::keymap_c::parse_keymap_c(&content) {
                        Ok(config) => {
                            // Update the keyboard state with new layout
                            self.state = KeyboardState::new(config.to_keyboard_layout());
                            self.file_status = format!("✅ Loaded C keymap: {} layers", config.layers.len());
                            println!("✅ Successfully loaded C keymap from: {}", self.file_path);
                        }
                        Err(e) => {
                            self.file_status = format!("❌ C parse error: {}", e);
                            eprintln!("❌ Failed to parse C keymap: {}", e);
                        }
                    }
                } else {
                    self.file_status = "❌ Unsupported file type. Use .json, .c, or .h files.".to_string();
                    eprintln!("❌ Unsupported file type. Please use .json, .c, or .h files.");
                }
            }
            Err(e) => {
                self.file_status = format!("❌ File read error: {}", e);
                eprintln!("❌ Failed to read file '{}': {}", self.file_path, e);
            }
        }
        
        self.file_loading = false;
    }
    
    fn open_file_dialog(&mut self) {
        // For now, we'll use a simple approach
        // In a real implementation, you might want to use rfd or similar crate
        // For now, we'll just set a placeholder that won't cause file read errors
        self.file_path = "".to_string();
        
        // TODO: Implement proper file dialog using rfd crate
        // This would require adding rfd as a dependency and implementing async file selection
    }
}

impl eframe::App for KeyboardViewerApp {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		// Drain any pending reports
		while let Ok(rep) = self.rx.try_recv() {
			self.state.set_layer(rep.active_layer);
			self.state.set_pressed_bits(rep.pressed_bits);
		}

		#[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
		{
			// In mock mode, use manual pressed keys
			let mut bits = 0u64;
			for &idx in &self.manual_pressed {
				bits |= 1u64 << idx;
			}
			self.state.set_pressed_bits(bits);
		}

		let layer_idx = self.state.active_layer as usize;
		let layer_name = self.state.keyboard.layer_names.get(layer_idx).cloned().unwrap_or_else(|| format!("Layer {}", layer_idx));

        egui::TopBottomPanel::top("top")
            .min_height(60.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Add padding to the panel
                    ui.add_space(10.0);
                    
                    // Layer info
                    ui.label("Layer:");
                    ui.label(RichText::new(format!("{} (#{})", layer_name.clone(), layer_idx)).strong());
                    
                    ui.separator();
                    
                    // File input section
                    ui.label("File:");
                    ui.add(egui::TextEdit::singleline(&mut self.file_path)
                        .hint_text("keymap.c or .json file path")
                        .desired_width(200.0));
                    
                    if ui.add(egui::Button::new("Browse")
                        .fill(Palette::OVERLAY)
                        .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::Vec2::new(80.0, 30.0))).clicked() {
                        self.open_file_dialog();
                    }
                    
                    let load_button = egui::Button::new(if self.file_loading { "Loading..." } else { "Load" })
                        .fill(if self.file_loading { Palette::YELLOW } else { Palette::GREEN })
                        .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::Vec2::new(60.0, 30.0));
                    
                    let response = ui.add(load_button);
                    if response.clicked() && !self.file_loading && !self.file_path.is_empty() {
                        self.load_keymap_file();
                    }
                    
                    // Display file status
                    if !self.file_status.is_empty() {
                        ui.separator();
                        ui.label(RichText::new(&self.file_status)
                            .color(if self.file_status.starts_with("✅") { Palette::GREEN } else { Palette::PEACH }));
                    }
                    
                    ui.separator();
                    
                    // Control buttons
                    let debug_btn = if self.show_debug { "Debug" } else { "Debug" };
                    if ui.add(egui::Button::new(debug_btn)
                        .fill(Palette::OVERLAY)
                        .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::Vec2::new(60.0, 30.0))).clicked() { 
                        self.show_debug = !self.show_debug; 
                    }
                    
                    let legend_btn = if self.show_legend { "Legend" } else { "Legend" };
                    if ui.add(egui::Button::new(legend_btn)
                        .fill(Palette::OVERLAY)
                        .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::Vec2::new(60.0, 30.0))).clicked() { 
                        self.show_legend = !self.show_legend; 
                    }
                    
                    let textarea_btn = if self.show_textarea { "Textarea" } else { "Textarea" };
                    if ui.add(egui::Button::new(textarea_btn)
                        .fill(Palette::OVERLAY)
                        .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::Vec2::new(70.0, 30.0))).clicked() { 
                        self.show_textarea = !self.show_textarea; 
                    }
                    
                    #[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
                    {
                        ui.separator();
                        ui.label("Mode: Mock");
                        if ui.add(egui::Button::new("Layer +")
                            .fill(Palette::OVERLAY)
                            .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::Vec2::new(60.0, 30.0))).clicked() {
                            let new_layer = (self.state.active_layer + 1) % self.state.keyboard.layer_names.len() as u8;
                            self.state.set_layer(new_layer);
                        }
                        if ui.add(egui::Button::new("Layer -")
                            .fill(Palette::OVERLAY)
                            .stroke(egui::Stroke::new(1.0, Palette::TEXT))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::Vec2::new(60.0, 30.0))).clicked() {
                            let new_layer = if self.state.active_layer == 0 {
                                self.state.keyboard.layer_names.len() as u8 - 1
                            } else {
                                self.state.active_layer - 1
                            };
                            self.state.set_layer(new_layer);
                        }
                    }
                    
                    // Add padding to the right
                    ui.add_space(10.0);
                });
            });

        if self.show_debug {
            egui::SidePanel::right("debug").resizable(true).show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("Debug");
                ui.add_space(5.0);
                ui.label(format!("Active layer index: {}", layer_idx));
                ui.monospace(format!("Pressed bits: 0x{:012X}", self.state.pressed_bits));
                let mut pressed_indices: Vec<usize> = (0..(self.state.keyboard.rows * self.state.keyboard.cols))
                    .filter(|i| ((self.state.pressed_bits >> i) & 1) == 1)
                    .collect();
                pressed_indices.sort_unstable();
                ui.monospace(format!("Pressed indices: {:?}", pressed_indices));
                ui.add_space(10.0);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Ajouter plus d'espace autour du clavier
            ui.add_space(20.0);
            
            // Centrer le clavier avec un conteneur
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Conteneur avec padding pour le clavier
                egui::Frame::none()
                    .inner_margin(egui::Margin::same(30.0))
                    .show(ui, |ui| {
                        let rows = self.state.keyboard.rows;
                        let cols = self.state.keyboard.cols;
                        let key_size = Vec2::new(56.0, 56.0);
                        let spacing_y = 8.0;
                        let mut font_id = ui.style().text_styles[&egui::TextStyle::Body].clone();
                        font_id.size *= 1.15;

            // Track press start times for color transition (MT keys after 2s)
            let total_keys = rows * cols;
            for i in 0..total_keys {
                let pressed = ((self.state.pressed_bits >> i) & 1) == 1;
                if pressed {
                    self.pressed_started.entry(i).or_insert_with(Instant::now);
                } else {
                    self.pressed_started.remove(&i);
                }
            }

            for r in 0..rows {
				ui.horizontal(|ui| {
					for c in 0..cols {
						let pressed = self.state.is_pressed(r, c);
						let is_trns = self.state.is_transparent_key(layer_idx, r, c);
						let is_fn = self.state.is_function_key(layer_idx, r, c);
						let resp = ui.add_sized(key_size, egui::Label::new(" ").sense(Sense::click()));
						let rect = resp.rect;
						
						#[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
						{
							if resp.clicked() {
								if let Some(idx) = self.state.index_for(r, c) {
									if self.manual_pressed.contains(&idx) {
										self.manual_pressed.remove(&idx);
									} else {
										self.manual_pressed.insert(idx);
									}
								}
							}
						}
                        let bg = if is_trns {
                            Color32::from_rgba_unmultiplied(0, 0, 0, 0)
                        } else if pressed {
                            // Pressed color: start green; for MT keys after 2s switch to border color
                            let idx = self.state.index_for(r, c).unwrap_or(usize::MAX);
                            let mt = self.state.is_mt_key(layer_idx, r, c);
                            if mt {
                                if let Some(t0) = self.pressed_started.get(&idx) {
                                    if t0.elapsed() >= Duration::from_millis(500) {
                                        Palette::PEACH
                                    } else {
                                        Palette::GREEN
                                    }
                                } else {
                                    Palette::GREEN
                                }
                            } else {
                                Palette::GREEN
                            }
                        } else {
                            Palette::OVERLAY
                        };
						ui.painter().rect_filled(rect.shrink(3.0), 6.0, bg);

                        // Colored border by function type (Catppuccin Mocha)
                        if is_fn {
                            let mt = self.state.is_mt_key(layer_idx, r, c);
                            let lt = self.state.is_lt_key(layer_idx, r, c);
                            let osl = self.state.is_osl_key(layer_idx, r, c);
                            let color = if mt { Palette::PEACH } else if lt { Palette::BLUE } else if osl { Palette::YELLOW } else { Palette::TEXT };
                            ui.painter().rect_stroke(rect.shrink(2.5), 6.0, egui::Stroke { width: 1.2, color });
                            // Second line color will match this border color
                            let (main, sub) = self.state.display_parts(layer_idx, r, c);
                            // For MT: after 1s hold, swap main/sub display
                            let idx = self.state.index_for(r, c).unwrap_or(usize::MAX);
                            let held_swap = mt && self.pressed_started.get(&idx).map(|t0| t0.elapsed() >= Duration::from_millis(500)).unwrap_or(false);
                            let (big, small) = if held_swap { (sub.clone(), main.clone()) } else { (main.clone(), sub.clone()) };
                            if !big.is_empty() {
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    big,
                                    font_id.clone(),
                                    Color32::WHITE,
                                );
                            }
                            if !small.is_empty() && !osl {
                                let sub_pos = egui::pos2(rect.center().x, rect.center().y + 10.0);
                                ui.painter().text(
                                    sub_pos,
                                    egui::Align2::CENTER_TOP,
                                    small,
                                    egui::FontId { size: font_id.size * 0.7, family: font_id.family.clone() },
                                    color,
                                );
                            }
                            continue; // already drew labels above
                        }

                        // Draw main and sub labels (for MT/LT, etc.)
                        let (main, sub) = self.state.display_parts(layer_idx, r, c);
                        if !main.is_empty() {
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                main,
                                font_id.clone(),
                                Color32::WHITE,
                            );
                        }
                        if !sub.is_empty() {
                            let sub_pos = egui::pos2(rect.center().x, rect.center().y + 10.0);
                            ui.painter().text(
                                sub_pos,
                                egui::Align2::CENTER_TOP,
                                sub,
                                egui::FontId { size: font_id.size * 0.7, family: font_id.family.clone() },
                                Palette::TEXT,
                            );
                        }
                    }
                        });
                        ui.add_space(spacing_y);
                    }
                });
                
                ui.add_space(20.0);
            });
            
            // Legend and text input under the keyboard (outside the centered container)
            if self.show_legend || self.show_textarea {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    // Legend on the left (if enabled)
                    if self.show_legend {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0);
                                ui.heading("Legend");
                                ui.add_space(5.0);
                                let row = |ui: &mut egui::Ui, color: Color32, title: &str, desc: &str| {
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                                        ui.painter().rect_stroke(rect, 4.0, egui::Stroke { width: 2.0, color });
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(title).strong());
                                    });
                                    ui.label(desc);
                                    ui.add_space(4.0);
                                };
                                row(ui, Palette::PEACH, "MT(mod, key)", "");
                                row(ui, Palette::BLUE, "LT(layer, key)", "");
                                row(ui, Palette::YELLOW, "OSL ★", "");
                                ui.add_space(10.0);
                            });
                        });
                        
                        if self.show_textarea {
                            ui.add_space(20.0);
                        }
                    }
                    
                    // Text input on the right (if enabled)
                    if self.show_textarea {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0);
                                ui.heading("Text Input");
                                ui.add_space(5.0);
                                ui.add(egui::TextEdit::multiline(&mut self.text_input)
                                    .desired_width(ui.available_width())
                                    .desired_rows(8)
                                    .hint_text("Type here to test your keyboard layout..."));
                                ui.add_space(10.0);
                            });
                        });
                    }
                });
            }
        });
        

		ctx.request_repaint_after(std::time::Duration::from_millis(16));
	}
}
