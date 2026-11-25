use crate::design::{palette, semibold_family};
use crate::icons::{self, Icon};
use egui::{
	Color32, FontId, Mesh, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, epaint::Vertex,
	pos2, vec2,
};

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
	let primary_alpha = (if dark { 0.14 } else { 0.075 }) * intro * breath;
	paint_elliptical_glow(
		&painter,
		center,
		vec2(rx, ry),
		p.accent.gamma_multiply(primary_alpha),
	);

	let secondary_center = pos2(center.x + rx * 0.24, center.y - ry * 0.18);
	let secondary_alpha = (if dark { 0.055 } else { 0.035 }) * intro;
	paint_elliptical_glow(
		&painter,
		secondary_center,
		vec2(rx * 0.52, ry * 0.52),
		p.link.gamma_multiply(secondary_alpha),
	);
}

pub fn brand_lockup(ui: &mut Ui, brand: &str, elapsed: f32) {
	let p = palette(ui);
	let intro = ease_out_cubic((elapsed / 0.45).clamp(0.0, 1.0));
	let lift = (1.0 - intro) * 8.0;
	let font_id = FontId::new(28.0, semibold_family(ui.ctx()));
	let measure = ui
		.painter()
		.layout_no_wrap(brand.to_uppercase(), font_id.clone(), p.text_strong);
	let word_w = measure.size().x;

	let mark_size = 64.0;
	let gap = 16.0;
	let mut expand = ease_in_out_cubic(((elapsed - 0.18) / 0.68).clamp(0.0, 1.0));
	if mark_size + gap + word_w > ui.available_width() {
		expand = 0.0;
	}

	let total_w = mark_size + (gap + word_w) * expand;
	let avail = ui.available_width();
	let (outer, _) = ui.allocate_exact_size(vec2(avail.max(total_w), 72.0), Sense::hover());
	let row = Rect::from_center_size(
		pos2(outer.center().x, outer.center().y + lift),
		vec2(total_w, 72.0),
	);
	let mark = Rect::from_min_size(
		pos2(row.left(), row.center().y - mark_size * 0.5),
		vec2(mark_size, mark_size),
	);

	let halo = mark.expand(4.0 + 1.5 * (elapsed * 2.0).sin().abs());
	ui.painter().rect_stroke(
		halo,
		19.0,
		Stroke::new(1.0, p.accent.gamma_multiply(0.28 * intro)),
		StrokeKind::Outside,
	);
	ui.painter()
		.rect_filled(mark, 16.0, p.accent.gamma_multiply(intro));
	icons::paint(
		ui.painter(),
		Icon::Brume,
		mark.shrink(14.0),
		p.accent_text.gamma_multiply(intro),
	);

	if (0.30..=1.15).contains(&elapsed) {
		let t = ((elapsed - 0.30) / (1.15 - 0.30)).clamp(0.0, 1.0);
		let cx = egui::lerp((mark.left() - 28.0)..=(mark.right() + 28.0), t);
		let shine_painter = ui.painter().with_clip_rect(mark);
		let peak = Color32::from_white_alpha(105);
		let half_w = 14.0;
		let tilt = 10.0;
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
			pos2(mark.right() + gap, row.top()),
			pos2(mark.right() + gap + word_w * expand, row.bottom()),
		);
		let text_color = p.text_strong.gamma_multiply(expand * intro);
		let galley = ui
			.painter()
			.layout_no_wrap(brand.to_uppercase(), font_id, text_color);
		let pos = pos2(mark.right() + gap, row.center().y - galley.size().y * 0.5);
		ui.painter()
			.with_clip_rect(clip)
			.galley(pos, galley, text_color);
	}
}

pub fn progress_line(ui: &mut Ui, width: f32, elapsed: f32) {
	let p = palette(ui);
	let dark = ui.visuals().dark_mode;
	let width = width.min(ui.available_width()).max(120.0);
	let avail = ui.available_width();
	let height = 3.5;

	let (track_outer, _) = ui.allocate_exact_size(vec2(avail.max(width), height), Sense::hover());
	let track = Rect::from_center_size(track_outer.center(), vec2(width, height));
	let track_bg = p.muted.gamma_multiply(if dark { 0.18 } else { 0.14 });
	ui.painter().rect_filled(track, height * 0.5, track_bg);

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
	clip_painter.circle_filled(pos2(inner_a, track.center().y), height * 0.5, p.accent);
	clip_painter.circle_filled(pos2(inner_b, track.center().y), height * 0.5, p.link);
}

#[cfg(test)]
mod tests {
	use super::*;
	use egui::Frame;

	fn run_scene(dark: bool, size: Vec2, elapsed: f32) {
		let ctx = egui::Context::default();
		crate::design::apply(&ctx);
		ctx.set_visuals(if dark {
			egui::Visuals::dark()
		} else {
			egui::Visuals::light()
		});
		let rect = Rect::from_min_size(Pos2::ZERO, size);
		// Fonts upload a texture on the first frame; epaint panics if that delta is dropped.
		let mut output = ctx.run_ui(
			egui::RawInput {
				screen_rect: Some(rect),
				..Default::default()
			},
			|ui| {
				paint_backdrop(ui, rect, elapsed);
				egui::CentralPanel::default()
					.frame(Frame::NONE)
					.show(ui, |ui| {
						brand_lockup(ui, "Brume", elapsed);
						progress_line(ui, 240.0, elapsed);
					});
			},
		);
		output.textures_delta.clear();
	}

	#[test]
	fn renders_across_themes_sizes_and_times() {
		for dark in [false, true] {
			for size in [vec2(320.0, 360.0), vec2(1280.0, 800.0)] {
				for elapsed in [0.0, 0.5, 2.0] {
					run_scene(dark, size, elapsed);
				}
			}
		}
	}

	#[test]
	fn elapsed_seconds_tracks_and_resets_on_gap_or_rewind() {
		let ctx = egui::Context::default();
		let step_time = |time: f64| -> f32 {
			let mut out = 0.0;
			let mut output = ctx.run_ui(
				egui::RawInput {
					time: Some(time),
					..Default::default()
				},
				|ui| {
					out = elapsed_seconds(ui.ctx());
				},
			);
			output.textures_delta.clear();
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
