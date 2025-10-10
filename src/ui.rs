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
	pub fn new(_cc: &eframe::CreationContext<'_>, state: PlanckLayoutState, rx: Receiver<Report>) -> Self {
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
				ui.label(RichText::new(layer_name.clone()).strong());
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
			let key_size = Vec2::new(50.0, 50.0);
			let spacing_y = 8.0;
			let font_id = ui.style().text_styles[&egui::TextStyle::Body].clone();

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
							Color32::from_rgb(40, 180, 80)
						} else {
							Color32::from_rgb(60, 60, 60)
						};
						ui.painter().rect_filled(rect.shrink(3.0), 6.0, bg);

						// Subtle border for function keys
						if is_fn {
							let border = Color32::from_rgb(120, 180, 220);
							ui.painter().rect_stroke(rect.shrink(2.0), 6.0, egui::Stroke { width: 1.5, color: border });
						}

						if let Some(text) = self.state.legend_at(layer_idx, r, c) {
							if !text.is_empty() {
								let text = text.to_string();
								ui.painter().text(
									rect.center(),
									egui::Align2::CENTER_CENTER,
									text,
									font_id.clone(),
									Color32::WHITE,
								);
							}
						}
					}
				});
				ui.add_space(spacing_y);
			}
		});

		ctx.request_repaint_after(std::time::Duration::from_millis(16));
	}
}
