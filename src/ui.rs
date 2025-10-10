use crate::hid::Report;
use crate::planck::PlanckLayoutState;
use eframe::egui::{self, Color32, Context, RichText, Sense, Vec2};
use std::sync::mpsc::Receiver;

pub struct PlanckViewerApp {
	state: PlanckLayoutState,
	rx: Receiver<Report>,
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

        egui::CentralPanel::default().show(ctx, |ui| {
			let rows = self.state.keyboard.rows;
			let cols = self.state.keyboard.cols;
            let key_size = Vec2::new(56.0, 56.0);
			let spacing_y = 8.0;
            let mut font_id = ui.style().text_styles[&egui::TextStyle::Body].clone();
            font_id.size *= 1.15;

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
                            // Single accent color for pressed state
                            Color32::from_rgb(44, 160, 44)
                        } else {
                            Color32::from_rgb(60, 60, 60)
                        };
						ui.painter().rect_filled(rect.shrink(3.0), 6.0, bg);

                        // Colored border by function type
                        if is_fn {
                            let mt = self.state.is_mt_key(layer_idx, r, c);
                            let lt = self.state.is_lt_key(layer_idx, r, c);
                            let osl = self.state.is_osl_key(layer_idx, r, c);
                            let color = if mt {
                                Color32::from_rgb(220, 120, 60) // orange for MT
                            } else if lt {
                                Color32::from_rgb(60, 160, 220) // blue for LT
                            } else if osl {
                                Color32::from_rgb(200, 160, 40) // gold for OSL
                            } else {
                                Color32::from_rgb(120, 150, 180) // default
                            };
                            ui.painter().rect_stroke(rect.shrink(2.5), 6.0, egui::Stroke { width: 1.2, color });
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
                                Color32::from_rgb(200, 220, 255),
                            );
                        }
					}
            ui.add_space(8.0);
            // Legend for function colors
            ui.horizontal(|ui| {
                let badge = |ui: &mut egui::Ui, color: Color32, label: &str| {
                    let rect = ui.available_rect_before_wrap().shrink2(egui::vec2(0.0, 0.0));
                    let r = egui::Rect::from_min_size(rect.min, egui::vec2(16.0, 16.0));
                    ui.painter().rect_stroke(r, 3.0, egui::Stroke { width: 2.0, color });
                    ui.allocate_ui_with_layout(egui::vec2(4.0, 0.0), egui::Layout::right_to_left(egui::Align::Center), |_| {});
                    ui.label(label);
                };
                badge(ui, Color32::from_rgb(220, 120, 60), "MT(mod, key)");
                ui.separator();
                badge(ui, Color32::from_rgb(60, 160, 220), "LT(layer, key)");
                ui.separator();
                badge(ui, Color32::from_rgb(200, 160, 40), "OSL(layer) ★");
            });
        });
				ui.add_space(spacing_y);
			}
		});

		ctx.request_repaint_after(std::time::Duration::from_millis(16));
	}
}
