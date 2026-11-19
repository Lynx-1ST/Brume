//! One session-scoped stream viewport, with bounded zoom and pointer-anchored scrolling.
use crate::StageFocus;

struct Viewport {
	source: StageFocus,
	factor: f32,
	center: egui::Vec2,
}
#[derive(Default)]
pub(super) struct StreamZoom {
	views: Vec<Viewport>,
}
impl StreamZoom {
	pub fn reset(&mut self, source: StageFocus) {
		if let Some(view) = self.views.iter_mut().find(|view| view.source == source) {
			view.factor = 1.0;
			view.center = egui::vec2(0.5, 0.5);
		}
	}
	pub fn paint(
		&mut self,
		ui: &mut egui::Ui,
		source: StageFocus,
		rect: egui::Rect,
		texture: egui::TextureId,
	) {
		let index = self
			.views
			.iter()
			.position(|view| view.source == source)
			.unwrap_or_else(|| {
				if self.views.len() == client_core::voice::MAX_WATCHED_STREAMS + 1 {
					self.views.remove(0);
				}
				self.views.push(Viewport {
					source,
					factor: 1.0,
					center: egui::vec2(0.5, 0.5),
				});
				self.views.len() - 1
			});
		self.views[index].paint(ui, rect, texture);
	}
}
impl Viewport {
	fn zoom_at(&mut self, at: egui::Vec2, scroll: f32) {
		let next = (self.factor * (scroll * 0.004).exp()).clamp(1.0, 8.0);
		self.center += at * (1.0 / self.factor - 1.0 / next);
		self.factor = next;
		let half = 0.5 / self.factor;
		self.center = self
			.center
			.clamp(egui::Vec2::splat(half), egui::Vec2::splat(1.0 - half));
	}
	fn paint(&mut self, ui: &mut egui::Ui, rect: egui::Rect, texture: egui::TextureId) {
		if !rect.is_positive() {
			return;
		}
		let key = match self.source {
			StageFocus::LocalScreen => (0, 0),
			StageFocus::Stream(user) => (1, user.0),
			StageFocus::Participant(user) => (2, user.0),
		};
		let response = ui.interact(
			rect,
			ui.id().with(("stream-zoom", key)),
			egui::Sense::drag(),
		);
		if response.hovered() {
			let scroll = ui.input_mut(|input| {
				let scroll = input.smooth_scroll_delta.y;
				input.smooth_scroll_delta.y = 0.0;
				scroll
			});
			if scroll != 0.0
				&& let Some(pointer) = response.hover_pos()
			{
				let at = (pointer - rect.min) / rect.size() - egui::vec2(0.5, 0.5);
				self.zoom_at(at, scroll);
			}
		}
		if response.dragged() {
			self.center -= response.drag_delta() / rect.size() / self.factor;
		}
		if response.hovered()
			&& ui.input(|input| {
				input
					.pointer
					.button_double_clicked(egui::PointerButton::Primary)
			}) {
			self.factor = 1.0;
		}
		let half = 0.5 / self.factor;
		self.center = self
			.center
			.clamp(egui::Vec2::splat(half), egui::Vec2::splat(1.0 - half));
		let uv = egui::Rect::from_center_size(
			self.center.to_pos2(),
			egui::Vec2::splat(1.0 / self.factor),
		);
		ui.painter()
			.with_clip_rect(rect)
			.image(texture, rect, uv, egui::Color32::WHITE);
		response.on_hover_cursor(if self.factor > 1.0 {
			egui::CursorIcon::Grab
		} else {
			egui::CursorIcon::ZoomIn
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn zoom_keeps_the_pointer_on_the_same_pixel_and_never_exposes_outside_the_frame() {
		let mut view = Viewport {
			source: StageFocus::LocalScreen,
			factor: 1.0,
			center: egui::vec2(0.5, 0.5),
		};
		let pointer = egui::vec2(0.1, -0.1);
		let pixel = view.center + pointer / view.factor;
		view.zoom_at(pointer, 100.0);
		assert!((view.center + pointer / view.factor - pixel).length() < 0.00001);
		for (pointer, scroll) in [
			(egui::vec2(0.5, -0.5), 100000.0),
			(egui::vec2(-0.5, 0.5), -100000.0),
		] {
			view.zoom_at(pointer, scroll);
			let half = 0.5 / view.factor;
			assert!(view.factor >= 1.0 && view.factor <= 8.0);
			assert!(view.center.x - half >= 0.0 && view.center.y - half >= 0.0);
			assert!(view.center.x + half <= 1.0 && view.center.y + half <= 1.0);
		}
	}
}
