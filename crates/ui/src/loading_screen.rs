use crate::design::{
	has_window_background, medium, mix, paint_avatar, palette, semibold_family, window_palette,
};
use crate::i18n::translate;
use crate::icons::{self, Icon};
use egui::{
	Color32, CornerRadius, FontId, Frame, Margin, Mesh, Pos2, Rect, Sense, Shadow, Shape, Stroke,
	StrokeKind, Ui, UiBuilder, Vec2, epaint::Vertex, pos2, vec2,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreStep {
	CheckingLogin,
	Connecting,
}

fn ease_out_cubic(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	1.0 - (1.0 - t).powi(3)
}

fn ease_in_out_cubic(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	if t < 0.5 {
		4.0 * t * t * t
	} else {
		1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
	}
}

pub fn elapsed_seconds(ctx: &egui::Context) -> f32 {
	let now = ctx.input(|i| i.time);
	let id = egui::Id::unique("brume-restore-clock");
	let start = ctx.data_mut(|data| {
		let start = match data.get_temp::<(f64, f64)>(id) {
			Some((start, last_seen)) if now >= last_seen && now - last_seen <= 0.5 => start,
			_ => now,
		};
		data.insert_temp(id, (start, now));
		start
	});
	(now - start).max(0.0) as f32
}

fn paint_elliptical_glow(painter: &egui::Painter, center: Pos2, radii: Vec2, color: Color32) {
	const SEGMENTS: usize = 36;
	let mid_color = color.gamma_multiply(0.42);
	let mut mesh = Mesh::default();
	mesh.vertices.reserve(1 + SEGMENTS * 2);
	mesh.indices.reserve(SEGMENTS * 9);

	mesh.vertices.push(Vertex {
		pos: center,
		uv: egui::epaint::WHITE_UV,
		color,
	});

	for ring in [0.48_f32, 1.0_f32] {
		let ring_color = if ring < 0.9 {
			mid_color
		} else {
			Color32::TRANSPARENT
		};
		for i in 0..SEGMENTS {
			let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
			let (sin, cos) = angle.sin_cos();
			mesh.vertices.push(Vertex {
				pos: pos2(
					center.x + radii.x * ring * cos,
					center.y + radii.y * ring * sin,
				),
				uv: egui::epaint::WHITE_UV,
				color: ring_color,
			});
		}
	}

	for i in 0..SEGMENTS {
		let next = (i + 1) % SEGMENTS;
		let inner_a = 1 + i as u32;
		let inner_b = 1 + next as u32;
		let outer_a = 1 + SEGMENTS as u32 + i as u32;
		let outer_b = 1 + SEGMENTS as u32 + next as u32;
		mesh.add_triangle(0, inner_a, inner_b);
		mesh.add_triangle(inner_a, outer_a, outer_b);
		mesh.add_triangle(inner_a, outer_b, inner_b);
	}

	painter.add(Shape::mesh(mesh));
}

pub fn paint_backdrop(ui: &Ui, rect: Rect, elapsed: f32) {
	let painter = ui.painter().with_clip_rect(rect);
	let dark = ui.visuals().dark_mode;
	let p = palette(ui);
	let intro = ease_out_cubic((elapsed / 0.65).clamp(0.0, 1.0));
	let breath = 0.85 + 0.15 * (elapsed * 1.4).sin();

	let center = pos2(rect.center().x, rect.center().y - 20.0);
	let rx = (rect.width() * 0.38).clamp(220.0, 460.0);
	let ry = (rect.height() * 0.36).clamp(160.0, 320.0);
	let primary_alpha = (if dark { 0.13 } else { 0.07 }) * intro * breath;
	paint_elliptical_glow(
		&painter,
		center,
		vec2(rx, ry),
		p.accent.gamma_multiply(primary_alpha),
	);

	let secondary_center = pos2(center.x + rx * 0.26, center.y - ry * 0.20);
	let secondary_alpha = (if dark { 0.05 } else { 0.035 }) * intro;
	paint_elliptical_glow(
		&painter,
		secondary_center,
		vec2(rx * 0.52, ry * 0.52),
		p.link.gamma_multiply(secondary_alpha),
	);

	if !has_window_background(ui) && rect.width() >= 760.0 && rect.height() >= 480.0 {
		let fill = p
			.muted
			.gamma_multiply((if dark { 0.05 } else { 0.07 }) * intro);
		let line = p
			.border
			.gamma_multiply((if dark { 0.35 } else { 0.45 }) * intro);
		let stroke = Stroke::new(1.0, line);

		let rail_right = rect.left() + 68.0;
		let sidebar_right = rect.left() + 296.0;
		let header_bottom = rect.top() + 48.0;

		// Server rail
		painter.line_segment(
			[
				pos2(rail_right, rect.top()),
				pos2(rail_right, rect.bottom()),
			],
			stroke,
		);
		painter.rect_filled(
			Rect::from_min_size(
				pos2(rect.left() + 14.0, rect.top() + 14.0),
				vec2(40.0, 40.0),
			),
			12.0,
			fill,
		);
		painter.line_segment(
			[
				pos2(rect.left() + 18.0, rect.top() + 64.0),
				pos2(rect.left() + 50.0, rect.top() + 64.0),
			],
			stroke,
		);
		for i in 0..5 {
			let cy = rect.top() + 94.0 + i as f32 * 52.0;
			let alpha = (1.0 - i as f32 * 0.16).max(0.2);
			painter.circle_filled(
				pos2(rect.left() + 34.0, cy),
				20.0,
				fill.gamma_multiply(alpha),
			);
		}

		// Channel sidebar
		painter.line_segment(
			[
				pos2(sidebar_right, rect.top()),
				pos2(sidebar_right, rect.bottom()),
			],
			stroke,
		);
		painter.rect_filled(
			Rect::from_min_size(
				pos2(rect.left() + 84.0, rect.top() + 18.0),
				vec2(120.0, 12.0),
			),
			6.0,
			fill,
		);
		painter.line_segment(
			[
				pos2(rail_right, header_bottom),
				pos2(sidebar_right, header_bottom),
			],
			stroke,
		);

		let sidebar_bars: &[(f32, f32, f32)] = &[
			(66.0, 76.0, 8.0),
			(86.0, 140.0, 10.0),
			(114.0, 112.0, 10.0),
			(142.0, 156.0, 10.0),
			(170.0, 128.0, 10.0),
			(206.0, 88.0, 8.0),
			(226.0, 148.0, 10.0),
			(254.0, 118.0, 10.0),
			(282.0, 134.0, 10.0),
		];
		for &(y_off, w, h) in sidebar_bars {
			painter.rect_filled(
				Rect::from_min_size(pos2(rect.left() + 84.0, rect.top() + y_off), vec2(w, h)),
				h * 0.5,
				fill,
			);
		}

		let account_top = rect.bottom() - 52.0;
		painter.rect_filled(
			Rect::from_min_max(
				pos2(rail_right, account_top),
				pos2(sidebar_right, rect.bottom()),
			),
			0.0,
			fill,
		);
		painter.line_segment(
			[
				pos2(rail_right, account_top),
				pos2(sidebar_right, account_top),
			],
			stroke,
		);

		// Main pane
		painter.line_segment(
			[
				pos2(sidebar_right, header_bottom),
				pos2(rect.right(), header_bottom),
			],
			stroke,
		);
		let composer = Rect::from_min_max(
			pos2(rect.left() + 316.0, rect.bottom() - 60.0),
			pos2(rect.right() - 20.0, rect.bottom() - 16.0),
		);
		painter.rect_stroke(composer, 10.0, stroke, StrokeKind::Inside);
	}
}

pub fn stage_card<R>(
	ui: &mut Ui,
	max_width: f32,
	elapsed: f32,
	add: impl FnOnce(&mut Ui) -> R,
) -> R {
	let dark = ui.visuals().dark_mode;
	let p = palette(ui);
	let intro = ease_out_cubic((elapsed / 0.65).clamp(0.0, 1.0));
	let card_width = max_width.min((ui.available_width() - 32.0).max(240.0));
	let surface = mix(
		window_palette(ui).surface.to_opaque(),
		p.raised.to_opaque(),
		0.25,
	);

	let response = Frame::new()
		.fill(surface)
		.corner_radius(CornerRadius::same(20))
		.stroke(Stroke::new(1.0, p.border))
		.shadow(Shadow {
			offset: [0, 12],
			blur: 32,
			spread: 0,
			color: Color32::from_black_alpha(if dark { 110 } else { 28 }),
		})
		.inner_margin(Margin::symmetric(32, 28))
		.show(ui, |ui| {
			ui.set_width((card_width - 64.0).max(160.0));
			add(ui)
		});

	let card_rect = response.response.rect;
	if card_rect.width() > 40.0 {
		let highlight = p
			.accent
			.gamma_multiply((if dark { 0.35 } else { 0.22 }) * intro);
		ui.painter().line_segment(
			[
				pos2(card_rect.left() + 20.0, card_rect.top() + 0.5),
				pos2(card_rect.right() - 20.0, card_rect.top() + 0.5),
			],
			Stroke::new(1.0, highlight),
		);
	}

	response.inner
}

pub fn brand_lockup(ui: &mut Ui, brand: &str, elapsed: f32) {
	let p = palette(ui);
	let font_id = FontId::new(25.0, semibold_family(ui.ctx()));
	let measure = ui
		.painter()
		.layout_no_wrap(brand.to_uppercase(), font_id.clone(), p.text_strong);
	let word_w = measure.size().x;

	let mut expand = ease_in_out_cubic(((elapsed - 0.18) / 0.68).clamp(0.0, 1.0));
	if 56.0 + 14.0 + word_w > ui.available_width() {
		expand = 0.0;
	}

	let total_w = 56.0 + (14.0 + word_w) * expand;
	let avail = ui.available_width();
	let (outer, _) = ui.allocate_exact_size(vec2(avail.max(total_w), 64.0), Sense::hover());
	let row = Rect::from_center_size(outer.center(), vec2(total_w, 64.0));
	let mark = Rect::from_min_size(pos2(row.left(), row.center().y - 28.0), vec2(56.0, 56.0));

	let halo = mark.expand(4.0 + 1.5 * (elapsed * 2.0).sin().abs());
	ui.painter().rect_stroke(
		halo,
		17.0,
		Stroke::new(1.0, p.accent.gamma_multiply(0.28)),
		StrokeKind::Outside,
	);
	ui.painter().rect_filled(mark, 14.0, p.accent);
	icons::paint(ui.painter(), Icon::Brume, mark.shrink(13.0), p.accent_text);

	if (0.30..=1.15).contains(&elapsed) {
		let t = ((elapsed - 0.30) / (1.15 - 0.30)).clamp(0.0, 1.0);
		let cx = egui::lerp((mark.left() - 28.0)..=(mark.right() + 28.0), t);
		let shine_painter = ui.painter().with_clip_rect(mark);
		let peak = Color32::from_white_alpha(95);
		let half_w = 14.0;
		let tilt = 9.0;
		let mut mesh = Mesh::default();
		for &(y, shift) in &[(mark.top(), tilt), (mark.bottom(), -tilt)] {
			for &(dx, color) in &[
				(-half_w, Color32::TRANSPARENT),
				(0.0, peak),
				(half_w, Color32::TRANSPARENT),
			] {
				mesh.vertices.push(Vertex {
					pos: pos2(cx + shift + dx, y),
					uv: egui::epaint::WHITE_UV,
					color,
				});
			}
		}
		// Top row: 0, 1, 2; bottom row: 3, 4, 5
		mesh.add_triangle(0, 1, 4);
		mesh.add_triangle(0, 4, 3);
		mesh.add_triangle(1, 2, 5);
		mesh.add_triangle(1, 5, 4);
		shine_painter.add(Shape::mesh(mesh));
	}

	if expand > 0.01 {
		let clip = Rect::from_min_max(
			pos2(mark.right() + 14.0, row.top()),
			pos2(mark.right() + 14.0 + word_w * expand, row.bottom()),
		);
		let text_color = p.text_strong.gamma_multiply(expand);
		let galley = ui
			.painter()
			.layout_no_wrap(brand.to_uppercase(), font_id, text_color);
		let pos = pos2(mark.right() + 14.0, row.center().y - galley.size().y * 0.5);
		ui.painter()
			.with_clip_rect(clip)
			.galley(pos, galley, text_color);
	}
}

pub fn account_pill(ui: &mut Ui, name: &str, handle: Option<&str>) {
	let p = palette(ui);
	let pill_w = ui.available_width().min(296.0);
	let avail = ui.available_width();
	let (outer, _) = ui.allocate_exact_size(vec2(avail.max(pill_w), 48.0), Sense::hover());
	let rect = Rect::from_center_size(outer.center(), vec2(pill_w, 48.0));

	ui.painter().rect(
		rect,
		12.0,
		p.base,
		Stroke::new(1.0, p.border),
		StrokeKind::Inside,
	);

	let avatar_rect = Rect::from_min_size(
		pos2(rect.left() + 10.0, rect.center().y - 15.0),
		vec2(30.0, 30.0),
	);
	paint_avatar(ui, name, 30.0, avatar_rect);

	let dot_center = pos2(avatar_rect.right() - 3.0, avatar_rect.bottom() - 3.0);
	ui.painter().circle_filled(dot_center, 5.5, p.base);
	ui.painter().circle_filled(dot_center, 3.5, p.accent);

	let text_left = rect.left() + 50.0;
	let text_right = (rect.right() - 14.0).max(text_left);
	let handle = handle.filter(|h| !h.is_empty());

	let text_height = if handle.is_some() { 34.0 } else { 18.0 };
	let text_rect = Rect::from_min_max(
		pos2(text_left, rect.center().y - text_height * 0.5),
		pos2(text_right, rect.center().y + text_height * 0.5),
	);

	ui.scope_builder(UiBuilder::new().max_rect(text_rect), |ui| {
		ui.spacing_mut().item_spacing.y = 0.0;
		ui.add(egui::Label::new(medium(ui, name, 14.0).color(p.text_strong)).truncate());
		if let Some(h) = handle {
			ui.add(
				egui::Label::new(
					egui::RichText::new(format!("@{h}"))
						.size(12.0)
						.color(p.muted),
				)
				.truncate(),
			);
		}
	});
}

pub fn progress_steps(ui: &mut Ui, width: f32, step: RestoreStep, elapsed: f32) {
	let p = palette(ui);
	let width = width.min(ui.available_width()).max(120.0);
	let avail = ui.available_width();

	let step1_text = translate("main-restoring-step-credentials");
	let step2_text = translate("main-restoring-step-gateway");
	let font_id = FontId::proportional(12.0);
	let pulse = 0.72 + 0.28 * (elapsed * 3.0).sin().abs();

	let step1_color = match step {
		RestoreStep::CheckingLogin => p.text_strong,
		RestoreStep::Connecting => p.text,
	};
	let step2_color = match step {
		RestoreStep::CheckingLogin => p.muted,
		RestoreStep::Connecting => p.text_strong,
	};

	let galley1 = ui
		.painter()
		.layout_no_wrap(step1_text, font_id.clone(), step1_color);
	let galley2 = ui
		.painter()
		.layout_no_wrap(step2_text, font_id, step2_color);

	let badge_icon_w = 16.0;
	let step1_w = badge_icon_w + galley1.size().x;
	let step2_w = badge_icon_w + galley2.size().x;
	let connector_gap = 8.0;
	let connector_len = 14.0;
	let combined_w = step1_w + connector_gap * 2.0 + connector_len + step2_w;

	let (row1_outer, _) = ui.allocate_exact_size(vec2(avail.max(width), 18.0), Sense::hover());

	let paint_step1 = |painter: &egui::Painter, left: f32, cy: f32| {
		match step {
			RestoreStep::Connecting => {
				let icon_rect = Rect::from_center_size(pos2(left + 6.0, cy), vec2(12.0, 12.0));
				icons::paint(painter, Icon::Check, icon_rect, p.positive);
			}
			RestoreStep::CheckingLogin => {
				painter.circle_filled(pos2(left + 6.0, cy), 3.0, p.accent.gamma_multiply(pulse));
			}
		}
		let text_pos = pos2(left + badge_icon_w, cy - galley1.size().y * 0.5);
		painter.galley(text_pos, galley1, step1_color);
	};

	let paint_step2 = |painter: &egui::Painter, left: f32, cy: f32| {
		match step {
			RestoreStep::Connecting => {
				painter.circle_filled(pos2(left + 6.0, cy), 3.0, p.accent.gamma_multiply(pulse));
			}
			RestoreStep::CheckingLogin => {
				painter.circle_filled(pos2(left + 6.0, cy), 2.5, p.muted.gamma_multiply(0.45));
			}
		}
		let text_pos = pos2(left + badge_icon_w, cy - galley2.size().y * 0.5);
		painter.galley(text_pos, galley2, step2_color);
	};

	let cy = row1_outer.center().y;
	let row1_painter = ui.painter().with_clip_rect(row1_outer);
	if combined_w <= width {
		let start_x = row1_outer.center().x - combined_w * 0.5;
		paint_step1(&row1_painter, start_x, cy);
		let conn_start = start_x + step1_w + connector_gap;
		let conn_end = conn_start + connector_len;
		let conn_color = if step == RestoreStep::Connecting {
			p.accent.gamma_multiply(0.6)
		} else {
			p.border
		};
		row1_painter.line_segment(
			[pos2(conn_start, cy), pos2(conn_end, cy)],
			Stroke::new(1.5, conn_color),
		);
		paint_step2(&row1_painter, conn_end + connector_gap, cy);
	} else {
		match step {
			RestoreStep::CheckingLogin => {
				let start_x = row1_outer.center().x - step1_w * 0.5;
				paint_step1(&row1_painter, start_x, cy);
			}
			RestoreStep::Connecting => {
				let start_x = row1_outer.center().x - step2_w * 0.5;
				paint_step2(&row1_painter, start_x, cy);
			}
		}
	}

	ui.add_space(10.0);

	let (track_outer, _) = ui.allocate_exact_size(vec2(avail.max(width), 5.0), Sense::hover());
	let track = Rect::from_center_size(track_outer.center(), vec2(width, 5.0));
	ui.painter().rect(
		track,
		3.0,
		p.base,
		Stroke::new(1.0, p.border.gamma_multiply(0.5)),
		StrokeKind::Inside,
	);

	let phase = 0.5 - 0.5 * (elapsed * 2.2).cos();
	let span = track.width() * 0.42;
	let start_x = track.left() + (track.width() - span) * phase;
	let end_x = start_x + span;
	let inner_a = start_x + span * 0.25;
	let inner_b = start_x + span * 0.75;

	let clip_painter = ui.painter().with_clip_rect(track);
	let mut mesh = Mesh::default();
	for &y in &[track.top(), track.bottom()] {
		for &(x, color) in &[
			(start_x, Color32::TRANSPARENT),
			(inner_a, p.accent),
			(inner_b, p.link),
			(end_x, Color32::TRANSPARENT),
		] {
			mesh.vertices.push(Vertex {
				pos: pos2(x, y),
				uv: egui::epaint::WHITE_UV,
				color,
			});
		}
	}
	// Top row: 0, 1, 2, 3; bottom row: 4, 5, 6, 7
	for col in 0..3_u32 {
		mesh.add_triangle(col, col + 1, col + 5);
		mesh.add_triangle(col, col + 5, col + 4);
	}
	clip_painter.add(Shape::mesh(mesh));
	clip_painter.circle_filled(pos2(inner_a, track.center().y), 2.5, p.accent);
	clip_painter.circle_filled(pos2(inner_b, track.center().y), 2.5, p.link);
}

#[cfg(test)]
mod tests {
	use super::*;

	fn run_scene(dark: bool, size: Vec2, step: RestoreStep, elapsed: f32) {
		let ctx = egui::Context::default();
		crate::design::apply(&ctx);
		ctx.set_visuals(if dark {
			egui::Visuals::dark()
		} else {
			egui::Visuals::light()
		});
		let rect = Rect::from_min_size(Pos2::ZERO, size);
		let _ = ctx.run_ui(
			egui::RawInput {
				screen_rect: Some(rect),
				..Default::default()
			},
			|ui| {
				paint_backdrop(ui, rect, elapsed);
				egui::CentralPanel::default()
					.frame(Frame::NONE)
					.show(ui, |ui| {
						stage_card(ui, 420.0, elapsed, |ui| {
							brand_lockup(ui, "Brume", elapsed);
							account_pill(ui, "Alice Example", Some("alice"));
							account_pill(ui, "Bob", None);
							progress_steps(ui, 296.0, step, elapsed);
						});
					});
			},
		);
	}

	#[test]
	fn renders_across_themes_sizes_steps_and_times() {
		for dark in [false, true] {
			for size in [vec2(320.0, 360.0), vec2(1280.0, 800.0)] {
				for step in [RestoreStep::CheckingLogin, RestoreStep::Connecting] {
					for elapsed in [0.0, 0.5, 2.0] {
						run_scene(dark, size, step, elapsed);
					}
				}
			}
		}
	}

	#[test]
	fn elapsed_seconds_tracks_and_resets_on_gap_or_rewind() {
		let ctx = egui::Context::default();
		let step_time = |time: f64| -> f32 {
			let mut out = 0.0;
			let _ = ctx.run_ui(
				egui::RawInput {
					time: Some(time),
					..Default::default()
				},
				|ui| {
					out = elapsed_seconds(ui.ctx());
				},
			);
			out
		};

		assert!((step_time(10.0) - 0.0).abs() < 1e-5);
		assert!((step_time(10.25) - 0.25).abs() < 1e-5);
		assert!((step_time(10.5) - 0.5).abs() < 1e-5);
		// Gap > 0.5s resets start to current time
		assert!((step_time(11.5) - 0.0).abs() < 1e-5);
		assert!((step_time(11.7) - 0.2).abs() < 1e-5);
		// Time rewind resets start to current time
		assert!((step_time(9.0) - 0.0).abs() < 1e-5);
	}
}
