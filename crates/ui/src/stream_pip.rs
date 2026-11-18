//! One bounded, session-scoped in-app viewer; reuses the call's existing video textures.
use crate::{MessagingUi, StageFocus, design, i18n};
use client_core::{Command, State, voice::Phase};
use model::Id;
type Scope = (u64, Id, u64);
#[derive(Default)]
pub(super) struct StreamPip {
	scope: Option<Scope>,
	sources: Vec<StageFocus>,
	selected: Option<StageFocus>,
	requested: Option<(Scope, StageFocus)>,
	dismissed: bool,
	forced: bool,
	pub stage_visible: bool,
}
impl StreamPip {
	pub fn request_open(&mut self, scope: Scope, source: StageFocus) {
		self.requested = Some((scope, source));
	}
	fn reconcile(&mut self, scope: Scope, sources: Vec<StageFocus>) {
		let requested = self.requested.take();
		let visible = self.stage_visible;
		if self.scope != Some(scope) {
			*self = Self {
				scope: Some(scope),
				stage_visible: visible,
				..Default::default()
			};
		}
		if self.sources != sources {
			self.dismissed = false;
		}
		self.sources = sources;
		if !self.selected.is_some_and(|s| self.sources.contains(&s)) {
			self.selected = self.sources.last().copied();
		}
		if let Some((request_scope, source)) = requested
			&& request_scope == scope
			&& self.sources.contains(&source)
		{
			self.selected = Some(source);
			self.forced = true;
			self.dismissed = false;
		}
	}
}
impl MessagingUi {
	pub(super) fn request_stream_pip(&mut self, state: &State, source: StageFocus) {
		if let Some(call) = &state.voice.active {
			self.stream_pip
				.request_open((state.generation, call.channel, call.request), source);
		}
	}
	pub(super) fn show_stream_pip(
		&mut self,
		ctx: &egui::Context,
		state: &mut State,
		commands: &mut Vec<Command>,
	) {
		let Some(call) = state
			.voice
			.active
			.as_ref()
			.filter(|c| c.phase != Phase::Failed && state.can_view(c.channel))
		else {
			self.stream_pip = StreamPip::default();
			return;
		};
		let channel = call.channel;
		let scope = (state.generation, channel, call.request);
		let mut sources: Vec<_> = call
			.watching
			.iter()
			.take(client_core::voice::MAX_WATCHED_STREAMS)
			.copied()
			.map(StageFocus::Stream)
			.collect();
		if self.screen.busy && self.screen.context == Some(scope) {
			sources.push(StageFocus::LocalScreen);
		}
		self.stream_pip.reconcile(scope, sources);
		if self.stream_pip.dismissed || (self.stream_pip.stage_visible && !self.stream_pip.forced) {
			return;
		}
		let Some(mut selected) = self.stream_pip.selected else {
			return;
		};
		let bounds = ctx.content_rect().shrink(8.0);
		if bounds.width() < 100.0 || bounds.height() < 80.0 {
			return;
		}
		let max = bounds.size().min(egui::vec2(640.0, 400.0));
		let min = max.min(egui::vec2(180.0, 100.0));
		let size = max.min(egui::vec2(320.0, 180.0));
		let choices = self.stream_pip.sources.clone();
		let mut close = false;
		let mut stop = false;
		let mut restore = false;
		egui::Window::new(i18n::translate("voice-pip-title"))
			.id(egui::Id::unique("stream-pip-window"))
			.title_bar(false)
			.collapsible(false)
			.resizable(true)
			.drag_area(egui::WindowDrag::Anywhere)
			.scroll(false)
			.default_size(size)
			.min_size(min)
			.max_size(max)
			.constrain_to(bounds)
			.default_pos(
				bounds.right_top()
					+ egui::vec2(-size.x, 64.0_f32.min((bounds.height() - size.y).max(0.0))),
			)
			.frame(
				egui::Frame::new()
					.fill(egui::Color32::BLACK)
					.inner_margin(0)
					.corner_radius(8)
					.stroke(egui::Stroke::new(1.0, design::palette_for(ctx).border)),
			)
			.show(ctx, |ui| {
				let available = ui.available_size().min(max).max(min);
				let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click());
				match selected {
					StageFocus::LocalScreen => self.screen_tile(ui, rect, true),
					StageFocus::Stream(user) => {
						self.stream_tile(ui, state, rect, channel, user, false)
					}
					StageFocus::Participant(_) => {}
				}
				let viewers = match selected {
					StageFocus::LocalScreen => self.screen.viewers.as_ref(),
					StageFocus::Stream(_) => {
						crate::voice::watched_viewers(&self.voice_stream_views, selected)
					}
					StageFocus::Participant(_) => None,
				};
				if let Some(viewers) = viewers {
					pip_viewers(ui, &mut self.avatars, state, channel, rect, viewers);
				}
				if response.double_clicked() {
					restore = true;
				}
				egui::Popup::context_menu(&response).show(|ui| {
					ui.label(i18n::translate("voice-pip-source"));
					for source in &choices {
						if ui
							.selectable_label(
								selected == *source,
								pip_label(state, channel, *source),
							)
							.clicked()
						{
							selected = *source;
							ui.close();
						}
					}
					ui.separator();
					if matches!(selected, StageFocus::Stream(_)) {
						ui.checkbox(
							&mut self.voice_stream_muted,
							i18n::translate("voice-pip-mute"),
						);
					}
					if ui.button(i18n::translate("voice-pip-return")).clicked() {
						restore = true;
						ui.close();
					}
					if ui.button(i18n::translate("voice-pip-close")).clicked() {
						close = true;
						ui.close();
					}
				});
				if ui.rect_contains_pointer(rect) || response.has_focus() {
					let controls = egui::Rect::from_min_size(
						rect.right_bottom() + egui::vec2(-60.0, -32.0),
						egui::vec2(56.0, 28.0),
					);
					ui.scope_builder(egui::UiBuilder::new().max_rect(controls), |ui| {
						ui.horizontal(|ui| {
							restore |= crate::icons::button(
								ui,
								crate::icons::Icon::External,
								24.0,
								&i18n::translate("voice-pip-return"),
							)
							.clicked();
							stop |= crate::icons::button(
								ui,
								crate::icons::Icon::Close,
								24.0,
								&i18n::builtin_text(
									if matches!(selected, StageFocus::LocalScreen) {
										"Stop sharing"
									} else {
										"Stop watching"
									},
								),
							)
							.clicked();
						});
					});
				}
			});
		self.stream_pip.selected = Some(selected);
		if stop {
			match selected {
				StageFocus::Stream(user) => self.watch_request = Some((user, false)),
				StageFocus::LocalScreen => self.screen.request = Some(crate::screen::Request::Stop),
				StageFocus::Participant(_) => {}
			}
		}
		if close {
			self.stream_pip.dismissed = true;
			self.stream_pip.forced = false;
		}
		if restore && self.server_settings.navigate_away(state) {
			self.guild = state.channel(channel).and_then(|channel| channel.guild);
			self.voice_focus = Some(selected);
			self.voice_chat_open = false;
			self.stream_pip.forced = false;
			self.stream_pip.dismissed = false;
			if let Some(command) = state.select(channel) {
				commands.push(command);
			}
		}
	}
}
fn pip_viewers(
	ui: &mut egui::Ui,
	avatars: &mut crate::avatars::Avatars,
	state: &State,
	channel: Id,
	rect: egui::Rect,
	viewers: &client_core::screen::Viewers,
) {
	let bounds = egui::Rect::from_min_max(
		egui::pos2(rect.left() + 4.0, rect.bottom() - 32.0),
		egui::pos2(rect.right() - 66.0, rect.bottom() - 4.0),
	);
	let mut row = ui.new_child(
		egui::UiBuilder::new()
			.id_salt("pip-viewers")
			.max_rect(bounds)
			.layout(egui::Layout::right_to_left(egui::Align::Center)),
	);
	let background = row.painter().add(egui::Shape::Noop);
	let fill = egui::Color32::from_black_alpha(170);
	let size = if rect.width() < 220.0 { 14.0 } else { 18.0 };
	let response = crate::voice::viewer_row(
		&mut row,
		avatars,
		state,
		channel,
		viewers,
		size,
		fill,
		egui::Color32::WHITE,
	);
	row.painter().set(
		background,
		egui::Shape::rect_filled(response.rect.expand(3.0), 6, fill),
	);
	response.on_hover_text(crate::voice::viewer_names(state, channel, viewers));
}
fn pip_label(state: &State, channel: Id, source: StageFocus) -> String {
	match source {
		StageFocus::LocalScreen => i18n::translate("voice-pip-own"),
		StageFocus::Stream(user) => {
			let name = crate::voice::participant_user(state, channel, user)
				.map(|u| u.name.as_str())
				.unwrap_or("");
			i18n::translate_args("voice-pip-remote", &[("name", name)])
		}
		StageFocus::Participant(_) => String::new(),
	}
}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn dismissal_selection_and_pending_requests_do_not_cross_call_scopes() {
		let scope = (1, Id(2), 3);
		let a = StageFocus::Stream(Id(4));
		let b = StageFocus::Stream(Id(5));
		let mut pip = StreamPip::default();
		pip.reconcile(scope, vec![a, b]);
		pip.dismissed = true;
		pip.reconcile(scope, vec![a, b]);
		assert!(pip.dismissed);
		pip.request_open(scope, a);
		pip.reconcile(scope, vec![a, b]);
		assert_eq!(pip.selected, Some(a));
		assert!(pip.forced && !pip.dismissed);
		pip.reconcile(scope, vec![b]);
		assert_eq!(pip.selected, Some(b));
		pip.request_open(scope, a);
		pip.reconcile((2, Id(2), 3), vec![b]);
		assert_eq!(pip.selected, Some(b));
		assert!(!pip.forced);
		pip.reconcile((2, Id(2), 3), vec![]);
		assert!(pip.selected.is_none());
	}
}
