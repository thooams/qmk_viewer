use crate::hid::Report;
use crate::planck::PlanckLayoutState;
use eframe::egui::{self, Color32, Context, RichText, Sense, Vec2};

// Catppuccin Mocha palette (subset)
struct Palette;
impl Palette {
    const BLUE: Color32 = Color32::from_rgb(0x89, 0xb4, 0xfa);
    const PEACH: Color32 = Color32::from_rgb(0xfa, 0xb3, 0x87);
    const YELLOW: Color32 = Color32::from_rgb(0xf9, 0xe2, 0xaf);
    const GREEN: Color32 = Color32::from_rgb(0xa6, 0xe3, 0xa1);
    const SURFACE: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x2e); // base
    const OVERLAY: Color32 = Color32::from_rgb(0x31, 0x31, 0x41); // overlay0
    const TEXT: Color32 = Color32::from_rgb(0xc6, 0xd0, 0xf5);
}
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PlanckViewerApp {
	state: PlanckLayoutState,
	rx: Receiver<Report>,
    show_debug: bool,
    pressed_started: HashMap<usize, Instant>,
	#[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
	manual_pressed: std::collections::HashSet<usize>,
}

impl PlanckViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, state: PlanckLayoutState, rx: Receiver<Report>) -> Self {
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
            pressed_started: HashMap::new(),
            #[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
            manual_pressed: std::collections::HashSet::new(),
        }
    }
}

impl eframe::App for PlanckViewerApp {
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

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
			ui.horizontal(|ui| {
                ui.label("Layer:");
                ui.label(RichText::new(format!("{} (#{})", layer_name.clone(), layer_idx)).strong());
                ui.separator();
                let btn = if self.show_debug { "Hide debug" } else { "Show debug" };
                if ui.button(btn).clicked() { self.show_debug = !self.show_debug; }
				#[cfg(not(any(feature = "rawhid", feature = "qmk_console")))]
				{
					ui.separator();
					ui.label("Mode: Mock");
					if ui.button("Layer +").clicked() {
						let new_layer = (self.state.active_layer + 1) % self.state.keyboard.layer_names.len() as u8;
						self.state.set_layer(new_layer);
					}
					if ui.button("Layer -").clicked() {
						let new_layer = if self.state.active_layer == 0 {
							self.state.keyboard.layer_names.len() as u8 - 1
						} else {
							self.state.active_layer - 1
						};
						self.state.set_layer(new_layer);
					}
				}
			});
		});

        if self.show_debug {
            egui::SidePanel::right("debug").resizable(true).show(ctx, |ui| {
                ui.heading("Debug");
                ui.label(format!("Active layer index: {}", layer_idx));
                ui.monospace(format!("Pressed bits: 0x{:012X}", self.state.pressed_bits));
                let mut pressed_indices: Vec<usize> = (0..(self.state.keyboard.rows * self.state.keyboard.cols))
                    .filter(|i| ((self.state.pressed_bits >> i) & 1) == 1)
                    .collect();
                pressed_indices.sort_unstable();
                ui.monospace(format!("Pressed indices: {:?}", pressed_indices));
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
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
            // Single legend under the keyboard (outside rows loop)
            ui.add_space(10.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Legend");
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
                });
            });
        });
        

		ctx.request_repaint_after(std::time::Duration::from_millis(16));
	}
}
