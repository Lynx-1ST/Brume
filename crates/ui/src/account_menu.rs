//! Session-scoped account menu; the host owns presence publication.
use crate::{MessagingUi, design, dialog, icons, profiles};
use client_core::{Command, State};
use egui::{RichText, vec2};
use model::PresenceStatus;

#[derive(Default)]
pub(super) struct AccountMenu {
	open: bool,
	/// Set inside the popout: the popup owns `open` until its frame finishes.
	close: bool,
	generation: u64,
	draft: String,
	custom_open: bool,
	/// Chosen while the editor is open; only Apply commits it to a deadline.
	clear_after: ClearAfter,
	profile_session: profiles::ProfileSession,
}

/// Only global metadata for the currently signed-in identity may enter this menu.
fn account_profile(state: &State) -> Option<&model::UserProfile> {
	let own = state.user.as_ref()?.id;
	let global_own =
		|profile: &&model::UserProfile| profile.user.id == own && profile.guild.is_none();
	state
		.known_profile(own)
		.filter(global_own)
		.or_else(|| state.own_profile.data.as_ref().filter(global_own))
}

/// The menu loads the ordinary global profile; editor failures belong to the editor.
fn account_profile_progress(state: &State) -> (bool, Option<&'static str>) {
	let view = state.profile.as_ref().filter(|view| {
		view.guild.is_none() && state.user.as_ref().is_some_and(|own| own.id == view.user)
	});
	view.map_or((false, None), |view| {
		(view.loading, view.error.filter(|_| !view.loading))
	})
}

/// Leave room for the equipped frame around the visible card, outside its scrolling details.
fn account_popup_geometry(
	width: f32,
	height: f32,
	frame: Option<&model::ProfileFrame>,
) -> (f32, f32, [f32; 3]) {
	let available = width.max(1.0);
	let mut card_width = 340.0_f32.min(available);
	if let Some(frame) = frame {
		let [top, bottom, side] = frame.overflow(1.0);
		card_width = card_width.min(available / (1.0 + 2.0 * side));
		if top + bottom > 0.0 {
			card_width = card_width.min((height - 80.0).max(1.0) / (top + bottom));
		}
		let overflow = frame.overflow(card_width);
		return (
			card_width,
			(height - overflow[0] - overflow[1]).max(1.0),
			overflow,
		);
	}
	(card_width, height.max(1.0), [0.0; 3])
}

/// Discord's own "Clear after" choices. The deadline is absolute once applied, so a status
/// set for an hour still clears an hour later even if the editor is reopened meanwhile.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ClearAfter {
	#[default]
	Never,
	Minutes30,
	Hour,
	Hours4,
	Today,
}

impl ClearAfter {
	const ALL: [Self; 5] = [
		Self::Never,
		Self::Minutes30,
		Self::Hour,
		Self::Hours4,
		Self::Today,
	];
	fn label(self) -> String {
		crate::i18n::translate_if_key(match self {
			Self::Never => "account-menu-label-don-t-clear",
			Self::Minutes30 => "account-menu-label-30-minutes",
			Self::Hour => "account-menu-label-1-hour",
			Self::Hours4 => "account-menu-label-4-hours",
			Self::Today => "account-menu-label-today",
		})
	}
	/// Absolute deadline in milliseconds since the Unix epoch; `None` never clears.
	fn deadline(self) -> Option<u64> {
		let now = crate::local_time::now();
		let seconds = match self {
			Self::Never => return None,
			Self::Minutes30 => now.unix_timestamp() + 30 * 60,
			Self::Hour => now.unix_timestamp() + 60 * 60,
			Self::Hours4 => now.unix_timestamp() + 4 * 60 * 60,
			// End of the local day, which is what Discord means by "Today".
			Self::Today => {
				let midnight = now.replace_time(time::Time::MIDNIGHT);
				(midnight + time::Duration::days(1)).unix_timestamp()
			}
		};
		u64::try_from(seconds).ok().map(|seconds| seconds * 1000)
	}
	/// Plain-language moment this choice lands on, for the line under the dropdown.
	fn clears_at(self) -> Option<String> {
		if self == Self::Never {
			return None;
		}
		let now = crate::local_time::now();
		let at = match self {
			Self::Never => return None,
			Self::Minutes30 => now + time::Duration::minutes(30),
			Self::Hour => now + time::Duration::hours(1),
			Self::Hours4 => now + time::Duration::hours(4),
			Self::Today => now.replace_time(time::Time::MIDNIGHT) + time::Duration::days(1),
		};
		let clock = crate::local_time::clock(at);
		Some(if at.date() == now.date() {
			format!(
				"{} {clock}",
				crate::i18n::translate("account-menu-clears-at-at")
			)
		} else {
			format!(
				"{} {clock} {}",
				crate::i18n::translate("account-menu-clears-at-at"),
				crate::i18n::translate("account-menu-clears-at-tomorrow")
			)
		})
	}
	/// Nearest choice for an existing deadline, so reopening the editor shows what is set.
	fn nearest(expires: Option<u64>) -> Self {
		let Some(expires) = expires else {
			return Self::Never;
		};
		Self::ALL
			.into_iter()
			.skip(1)
			.find(|choice| {
				choice
					.deadline()
					.is_some_and(|deadline| expires <= deadline)
			})
			.unwrap_or(Self::Today)
	}
}

impl AccountMenu {
	/// Fixture-only: show the popout on the first frame of a demo capture.
	#[cfg(any(test, feature = "demo"))]
	pub(super) fn preview(&mut self, generation: u64) {
		self.open = true;
		self.generation = generation;
	}
	/// Fixture-only: open the custom-status editor over the popout.
	#[cfg(any(test, feature = "demo"))]
	pub(super) fn preview_editor(&mut self, generation: u64, draft: String) {
		self.preview(generation);
		self.clear_after = ClearAfter::Hour;
		self.draft = draft;
		self.custom_open = true;
	}
}

impl MessagingUi {
	pub fn adopt_account_presence(&mut self, presence: model::OwnPresence) {
		if !presence.custom_status.is_empty() && !presence.valid() {
			return;
		}
		if self.account_menu.draft == self.own_presence.custom_status {
			self.account_menu.draft.clone_from(&presence.custom_status);
			self.account_menu.clear_after = ClearAfter::nearest(presence.expires_at_ms);
		}
		self.own_presence_expires = presence.expires_at_ms;
		self.own_presence.expires_at_ms = presence.expires_at_ms;
		self.own_presence.status = presence.status;
		self.own_presence.custom_status = presence.custom_status;
	}

	/// The status to show the owner: Idle while inactivity applies to an Online choice.
	pub fn shown_presence_status(&self) -> PresenceStatus {
		if self.auto_idle_active && self.own_presence.status == PresenceStatus::Online {
			PresenceStatus::Idle
		} else {
			self.own_presence.status
		}
	}

	pub(super) fn account_menu(
		&mut self,
		anchor: &egui::Response,
		state: &mut State,
		commands: &mut Vec<Command>,
	) {
		if self.account_menu.generation != state.generation {
			self.account_menu = AccountMenu {
				generation: state.generation,
				..Default::default()
			};
		}
		if anchor.clicked() {
			self.account_menu.open = !self.account_menu.open;
			if self.account_menu.open {
				self.account_menu.draft = self.own_presence.custom_status.clone();
				self.profile.hide();
				// Reuse the normal bounded profile read, including optional cosmetics and
				// badge metadata. This is a read, never the profile-edit command.
				if let Some(own) = state.user.as_ref().map(|user| user.id)
					&& let Some(command) = state.request_profile(own, None)
				{
					commands.push(command);
				}
			}
		}
		let colors = design::palette_for(&anchor.ctx);
		let mut open = self.account_menu.open;
		let id = anchor.id.with("account-menu");
		let popup_anchor = egui::PopupAnchor::from(anchor);
		const GAP: f32 = 6.0;
		let bounds = anchor.ctx.content_rect().shrink(8.0);
		let available_height = popup_anchor
			.rect(id, &anchor.ctx)
			.map_or(620.0, |rect| (rect.top() - bounds.top() - GAP).max(180.0))
			.min(bounds.height().max(1.0));
		let (width, height, [top, bottom, side]) = account_popup_geometry(
			bounds.width(),
			available_height,
			account_profile(state)
				.and_then(|profile| profile.frame.as_deref())
				.filter(|_| self.avatars.cosmetics.frames),
		);
		let outer_width = width + 2.0 * side;
		egui::Popup::from_response(anchor)
			.id(id)
			.anchor(popup_anchor)
			.gap(GAP)
			.open_bool(&mut open)
			.align(egui::RectAlign::TOP_START)
			.close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
			.width(outer_width)
			.frame(egui::Frame::NONE)
			.show(|ui| {
				ui.set_width(outer_width);
				ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
				let frame_back = ui.painter().add(egui::Shape::Noop);
				ui.add_space(top);
				let card = ui
					.horizontal(|ui| {
						ui.add_space(side);
						let card = egui::Frame::popup(ui.style())
							.fill(colors.raised)
							.stroke(egui::Stroke::NONE)
							.inner_margin(0)
							.corner_radius(10)
							.show(ui, |ui| {
								ui.set_width(width);
								ui.spacing_mut().item_spacing = vec2(8.0, 4.0);
								egui::ScrollArea::vertical()
									.min_scrolled_height(height)
									.max_height(height)
									.show(ui, |ui| {
										self.account_menu_contents(ui, state, commands, height)
									});
							});
						ui.add_space(side);
						card.response.rect
					})
					.inner;
				ui.add_space(bottom);
				if let Some(profile) = account_profile(state) {
					profiles::paint_collectibles(
						ui,
						&mut self.avatars,
						profile,
						card,
						frame_back,
						(false, None),
						state.demo,
					);
				}
			});
		self.account_menu.open = open && !std::mem::take(&mut self.account_menu.close);
		if self.account_menu.custom_open {
			let ctx = anchor.ctx.clone();
			let response = crate::dialog::Dialog::new(
				"custom-status-editor",
				crate::i18n::translate("account-menu-account-menu-custom-status"),
			)
			.subtitle(crate::i18n::translate(
				"account-menu-account-menu-shown-next-to-your-name-across-discord",
			))
			.width(420.0)
			.show(&ctx, |d| {
				d.content(|ui| self.custom_status_editor(ui, state));
				d.footer(|ui| self.custom_status_actions(ui));
			});
			if response.close {
				self.account_menu.custom_open = false;
			}
		}
	}

	fn account_menu_contents(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
		height: f32,
	) {
		// Keep account actions reachable on short windows while richer personal details
		// scroll independently. The enclosing scroll area still handles very small screens.
		egui::ScrollArea::vertical()
			.id_salt("account-profile-details")
			.max_height((height - 210.0).max(120.0))
			.show(ui, |ui| self.account_profile_contents(ui, state, commands));
		egui::Frame::new()
			.inner_margin(egui::Margin::symmetric(12, 8))
			.show(ui, |ui| {
				ui.spacing_mut().item_spacing.y = 2.0;
				self.account_status_row(ui);
				self.account_custom_status_row(ui);
				self.account_switcher(ui, state);
			});
	}

	fn account_profile_contents(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
	) {
		let colors = design::palette(ui);
		let theme = profiles::Theme::new(
			&colors,
			account_profile(state)
				.and_then(|profile| profiles::shown_theme_colors(ui.ctx(), profile)),
		);
		ui.visuals_mut().override_text_color = Some(theme.text);
		ui.visuals_mut().hyperlink_color = theme.link;
		let top = ui.cursor().top();
		let left = ui.cursor().left();
		let width = ui.available_width();
		let background = ui.painter().add(egui::Shape::Noop);
		let (banner, _) =
			ui.allocate_exact_size(vec2(ui.available_width(), 100.0), egui::Sense::hover());
		let corner = egui::CornerRadius {
			nw: 10,
			ne: 10,
			sw: 0,
			se: 0,
		};
		if let Some(profile) = account_profile(state) {
			self.avatars
				.paint_banner(ui, profile, banner, corner, state.demo);
		} else {
			ui.painter().rect_filled(
				banner,
				corner,
				design::mix(colors.accent, colors.base, 0.55),
			);
		}
		if let Some(user) = account_profile(state)
			.map(|profile| &profile.user)
			.or(state.user.as_ref())
		{
			let avatar = egui::Rect::from_min_size(
				banner.left_bottom() + vec2(14.0, -38.0),
				vec2(72.0, 72.0),
			);
			// Thick popover-coloured ring, as Discord punches the avatar out of the banner.
			ui.painter()
				.circle_filled(avatar.center(), 42.0, theme.card);
			ui.scope_builder(egui::UiBuilder::new().max_rect(avatar), |ui| {
				self.avatars.with_avatar_animation(true, |avatars| {
					if let Some(profile) = account_profile(state) {
						avatars.show_profile_avatar(ui, profile, 72.0, state.demo)
					} else {
						avatars.show(ui, user, 72.0, state.demo)
					}
				});
			});
			// Same 6 px ring as the avatar, so the dot is cut out of the banner alike.
			let dot = avatar.right_bottom() - egui::Vec2::splat(12.0);
			ui.painter().circle_filled(dot, 17.5, theme.card);
			presence_dot(
				ui.painter(),
				dot,
				11.5,
				self.shown_presence_status(),
				theme.card,
			);
			ui.add_space((avatar.bottom() + 12.0 - ui.cursor().top()).max(0.0));
		} else {
			ui.add_space(12.0);
		}
		egui::Frame::new()
			.inner_margin(egui::Margin {
				left: 12,
				right: 12,
				top: 0,
				bottom: 12,
			})
			.show(ui, |ui| {
				ui.set_width(ui.available_width());
				self.account_identity_card(ui, state, commands, &theme);
				if let Some(user) = &state.user {
					let guild = state
						.selected
						.and_then(|id| state.channel(id))
						.and_then(|c| c.guild);
					let (_, _, activities, _) = profiles::presence(state, user.id, guild);
					if !activities.is_empty() {
						ui.add_space(8.0);
						profiles::activity_list(
							ui,
							egui::Id::unique(("account-activity", user.id)),
							activities,
							&mut self.avatars,
							state.demo,
							(theme.panel, theme.muted),
						);
					}
				}
			});
		let rect = egui::Rect::from_min_max(
			egui::pos2(left, top),
			egui::pos2(left + width, ui.cursor().top()),
		);
		ui.painter().set(background, theme.background(rect));
	}

	/// Other accounts remembered on this device, plus a row to remember one more.
	fn account_switcher(&mut self, ui: &mut egui::Ui, state: &State) {
		let colors = design::palette(ui);
		let current = state.user.as_ref().map(|user| user.id);
		// Bounded by MAX_SAVED_ACCOUNTS; cloned so the avatar cache stays mutably borrowable.
		let others: Vec<model::SavedAccount> = self
			.accounts
			.iter()
			.filter(|account| Some(account.id) != current)
			.cloned()
			.collect();
		ui.add_space(10.0);
		let (line, _) =
			ui.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
		ui.painter().rect_filled(line, 0, colors.border);
		ui.add_space(10.0);
		ui.label(design::eyebrow(
			ui,
			crate::i18n::translate("account-menu-account-switcher-switch-accounts"),
			colors.muted,
		));
		ui.add_space(4.0);
		for account in &others {
			self.account_switcher_row(ui, account, state.demo);
		}
		let response = ui
			.scope(|ui| {
				let width = ui.available_width();
				ui.spacing_mut().button_padding = vec2(34.0, 10.0);
				ui.add(
					egui::Button::new(())
						.left_text(
							design::medium(
								ui,
								crate::i18n::translate(
									"account-menu-account-switcher-add-an-account",
								),
								14.0,
							)
							.color(colors.text_strong),
						)
						.frame_when_inactive(false)
						.corner_radius(6)
						.min_size(vec2(width, 40.0)),
				)
			})
			.inner;
		icons::paint(
			ui.painter(),
			icons::Icon::Plus,
			egui::Rect::from_center_size(
				egui::pos2(response.rect.left() + 17.0, response.rect.center().y),
				egui::Vec2::splat(17.0),
			),
			colors.muted,
		);
		if response.clicked() {
			self.add_account_requested = true;
			self.account_menu.close = true;
		}
	}

	/// One saved account: click to switch, trailing bin to forget it on this device.
	fn account_switcher_row(
		&mut self,
		ui: &mut egui::Ui,
		account: &model::SavedAccount,
		demo: bool,
	) {
		let colors = design::palette(ui);
		let (rect, _) =
			ui.allocate_exact_size(vec2(ui.available_width(), 44.0), egui::Sense::hover());
		let bin = egui::Rect::from_center_size(
			egui::pos2(rect.right() - 20.0, rect.center().y),
			egui::Vec2::splat(28.0),
		);
		let user = account.user();
		let avatar = egui::Rect::from_center_size(
			egui::pos2(rect.left() + 22.0, rect.center().y),
			egui::Vec2::splat(32.0),
		);
		let text = egui::Rect::from_min_max(
			egui::pos2(rect.left() + 48.0, rect.top() + 3.0),
			egui::pos2(bin.left() - 8.0, rect.bottom() - 3.0),
		);
		let over_bin = ui.rect_contains_pointer(bin);
		if ui.rect_contains_pointer(rect) {
			ui.painter().rect_filled(rect, 6, colors.hover);
		}
		ui.scope_builder(egui::UiBuilder::new().max_rect(avatar), |ui| {
			self.avatars.show_plain(ui, &user, 32.0, demo);
		});
		ui.scope_builder(egui::UiBuilder::new().max_rect(text), |ui| {
			ui.spacing_mut().item_spacing.y = 0.0;
			ui.add(
				egui::Label::new(
					design::medium(ui, account.label(), 14.0).color(colors.text_strong),
				)
				.truncate()
				.selectable(false),
			);
			// The handle sits under the display name; the blank line keeps the row height when an
			// entry does not know it yet, instead of naming the account twice.
			ui.add(
				egui::Label::new(
					RichText::new(account.handle().unwrap_or_default())
						.size(12.0)
						.color(colors.muted),
				)
				.truncate()
				.selectable(false),
			);
		});
		// Same affordance as the sign-in screen's saved accounts.
		icons::paint(
			ui.painter(),
			icons::Icon::Close,
			bin.shrink(8.0),
			if over_bin {
				colors.danger
			} else {
				colors.muted
			},
		);
		// Registered after the contents so the row, not a label, receives the click.
		let row = ui.interact(
			rect,
			ui.scope_id().with(("switch-account", account.id.0)),
			egui::Sense::click(),
		);
		let forget = ui.interact(
			bin,
			ui.scope_id().with(("forget-account", account.id.0)),
			egui::Sense::click(),
		);
		if row.has_focus() || forget.has_focus() {
			ui.painter().rect_stroke(
				rect.shrink(1.0),
				6,
				egui::Stroke::new(1.0, colors.accent),
				egui::StrokeKind::Inside,
			);
		}
		row.widget_info(|| {
			egui::WidgetInfo::labeled(
				egui::Role::Button,
				true,
				format!(
					"{} {}",
					crate::i18n::translate("account-menu-account-switcher-row-switch-to"),
					account.label()
				),
			)
		});
		forget.widget_info(|| {
			egui::WidgetInfo::labeled(
				egui::Role::Button,
				true,
				format!(
					"{} {}",
					crate::i18n::translate("account-menu-account-switcher-row-forget"),
					account.label()
				),
			)
		});
		let forget = forget.on_hover_text(crate::i18n::translate(
			"account-menu-account-switcher-row-forget-this-account-on-this-device",
		));
		if forget.clicked() {
			self.forget_account_requested = Some(account.id);
			self.account_menu.close = true;
		} else if row.clicked() {
			self.switch_account_requested = Some(account.id);
			self.account_menu.close = true;
		}
	}

	/// Name, handle and current custom status, grouped on the sunken card Discord uses.
	fn account_identity_card(
		&mut self,
		ui: &mut egui::Ui,
		state: &mut State,
		commands: &mut Vec<Command>,
		theme: &profiles::Theme,
	) {
		let colors = design::palette(ui);
		egui::Frame::new()
			.fill(theme.panel)
			.corner_radius(8)
			.inner_margin(12)
			.show(ui, |ui| {
				ui.set_width(ui.available_width());
				ui.spacing_mut().item_spacing.y = 2.0;
				let profile = account_profile(state);
				let name = profile
					.and_then(|p| p.global_name.as_deref())
					.or_else(|| state.user.as_ref().map(|u| u.name.as_str()))
					.map(str::to_owned)
					.unwrap_or_else(|| crate::i18n::translate("your-account"));
				crate::name_styles::label(
					ui,
					profile
						.map(|profile| &profile.user)
						.or(state.user.as_ref())
						.and_then(crate::name_styles::style),
					&name,
					egui::FontId::new(20.0, design::semibold_family(ui.ctx())),
					None,
					theme.card,
					theme.text,
					egui::Sense::hover(),
					true,
				);
				if let Some(profile) = profile {
					ui.add(
						egui::Label::new(
							design::medium(ui, &profile.username, 14.0).color(theme.muted),
						)
						.wrap(),
					);
					if !profile.pronouns.is_empty() {
						ui.add(
							egui::Label::new(
								RichText::new(&profile.pronouns)
									.size(12.0)
									.color(theme.muted),
							)
							.wrap(),
						);
					}
					if let Some(tag) = profile
						.clan
						.as_ref()
						.or(profile.user.primary_guild.as_deref())
						.filter(|_| !self.avatars.cosmetics.hide_clan_tags)
					{
						profiles::server_tag(ui, tag, &mut self.avatars, state.demo);
					}
					let catalog = profile.badge_catalog.as_deref();
					let badges = model::badges::displayed_badges(&profile.badges, catalog);
					if !badges.is_empty() {
						ui.add_space(4.0);
						ui.horizontal_wrapped(|ui| {
							for badge in &badges {
								if badge.icon_key().is_some() {
									// The compact menu offers hover details; badge editing remains
									// in the full profile's existing management surface.
									crate::badges::badge_icon(
										ui,
										&mut self.avatars,
										badge,
										None,
										22.0,
										state.demo,
									);
								} else {
									ui.label(
										RichText::new(&badge.description)
											.size(11.0)
											.color(theme.muted),
									);
								}
							}
						});
					}
					if !profile.bio.trim().is_empty() {
						ui.add_space(10.0);
						ui.label(design::eyebrow(
							ui,
							crate::i18n::translate("profiles-show-about-me"),
							theme.muted,
						));
						ui.scope(|ui| {
							ui.visuals_mut().override_text_color = Some(theme.text);
							ui.visuals_mut().hyperlink_color = theme.link;
							if let Some(user) = profiles::biography(
								ui,
								profile.user.id,
								&profile.bio,
								state,
								&mut self.avatars,
								&mut self.timeline.opening,
								&mut self.profile_formatted,
								&mut self.account_menu.profile_session,
							) {
								self.profile.command_open(user);
								self.account_menu.close = true;
							}
						});
					}
				} else if account_profile_progress(state).0 {
					// The username line every loaded profile adds under the name. Only a first
					// load is sketched: a reload keeps the shown profile silently (owner decision).
					let font = egui::FontId::new(14.0, design::medium_family(ui.ctx()));
					let height = ui.fonts_mut(|fonts| fonts.row_height(&font).round());
					crate::skeleton::custom(
						ui,
						&crate::i18n::translate(
							"account-menu-account-identity-card-loading-profile",
						),
						vec2(ui.available_width(), height),
						|painter, rect, fill| {
							crate::skeleton::bar(
								painter,
								fill,
								egui::pos2(rect.left(), rect.center().y - 5.0),
								vec2(96.0_f32.min(rect.width()), 10.0),
							);
						},
					);
				}
				if !self.own_presence.custom_status.is_empty() {
					ui.add_space(10.0);
					let (line, _) = ui
						.allocate_exact_size(vec2(ui.available_width(), 1.0), egui::Sense::hover());
					ui.painter().rect_filled(line, 0, colors.border);
					ui.add_space(10.0);
					ui.add(
						egui::Label::new(
							RichText::new(&self.own_presence.custom_status)
								.size(14.0)
								.color(theme.text),
						)
						.wrap(),
					);
				}
				if let Some(error) = account_profile_progress(state).1 {
					ui.add_space(6.0);
					ui.add(
						egui::Label::new(RichText::new(error).size(12.0).color(colors.danger))
							.wrap(),
					);
					if ui
						.button(crate::i18n::translate(
							"account-menu-account-identity-card-reload-profile",
						))
						.clicked() && let Some(own) = state.user.as_ref().map(|user| user.id)
						&& let Some(command) = state.request_profile(own, None)
					{
						commands.push(command);
					}
				}
			});
	}

	/// Presence row: status dot, label and a chevron opening the status menu.
	fn account_status_row(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		let status = self.own_presence.status;
		let label = design::medium(
			ui,
			crate::i18n::translate_if_key(presence_status_key(status)),
			14.0,
		)
		.color(colors.text_strong);
		let response = ui
			.scope(|ui| {
				let width = ui.available_width();
				ui.spacing_mut().button_padding = vec2(34.0, 10.0);
				egui::containers::menu::MenuButton::from_button(
					egui::Button::new(())
						.left_text(label)
						.frame_when_inactive(false)
						.corner_radius(6)
						.min_size(vec2(width, 40.0)),
				)
				.ui(ui, |ui| self.presence_menu(ui))
				.0
			})
			.inner;
		let rect = response.rect;
		let background = if response.hovered() || response.has_focus() {
			ui.visuals().widgets.hovered.weak_bg_fill
		} else {
			colors.raised
		};
		presence_dot(
			ui.painter(),
			egui::pos2(rect.left() + 17.0, rect.center().y),
			5.0,
			status,
			background,
		);
		icons::paint(
			ui.painter(),
			icons::Icon::ChevronRight,
			egui::Rect::from_center_size(
				egui::pos2(rect.right() - 18.0, rect.center().y),
				egui::Vec2::splat(16.0),
			),
			colors.muted,
		);
	}

	fn presence_menu(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		ui.set_width(260.0_f32.min(ui.ctx().content_rect().width() - 48.0));
		for status in PresenceStatus::ALL {
			let label = crate::i18n::translate_if_key(presence_status_key(status));
			let description = crate::i18n::translate_if_key(match status {
				PresenceStatus::DoNotDisturb => {
					"account-menu-presence-menu-you-will-not-receive-desktop-notifications"
				}
				PresenceStatus::Invisible => "account-menu-presence-menu-you-will-appear-offline",
				_ => "",
			});
			let height = if description.is_empty() { 40.0 } else { 62.0 };
			let response = ui.add_sized(
				[ui.available_width(), height],
				egui::Button::new("")
					.frame_when_inactive(false)
					.corner_radius(6),
			);
			response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &label));
			let x = response.rect.left() + 34.0;
			let y = response.rect.top() + if description.is_empty() { 11.0 } else { 10.0 };
			let background = if response.hovered() || response.has_focus() {
				ui.visuals().widgets.hovered.weak_bg_fill
			} else {
				ui.visuals().window_fill
			};
			presence_dot(
				ui.painter(),
				egui::pos2(response.rect.left() + 17.0, response.rect.center().y),
				5.0,
				status,
				background,
			);
			ui.painter().text(
				egui::pos2(x, y),
				egui::Align2::LEFT_TOP,
				&label,
				egui::FontId::new(14.0, design::medium_family(ui.ctx())),
				colors.text_strong,
			);
			if !description.is_empty() {
				let galley = ui.painter().layout(
					description,
					egui::FontId::proportional(12.0),
					colors.muted,
					(response.rect.width() - 46.0).max(80.0),
				);
				ui.painter()
					.galley(egui::pos2(x, y + 22.0), galley, colors.muted);
			}
			if response.clicked() {
				if self.own_presence.status != status {
					self.own_presence.status = status;
					self.own_presence_changed = true;
				}
				ui.close();
			}
			if status == PresenceStatus::Online {
				ui.separator();
			}
		}
	}

	/// Custom status row: smiley to write one, pencil once a status is set.
	fn account_custom_status_row(&mut self, ui: &mut egui::Ui) {
		let colors = design::palette(ui);
		let set = !self.own_presence.custom_status.is_empty();
		let label = if set {
			"account-menu-custom-status-edit"
		} else {
			"account-menu-custom-status-set"
		};
		let text = design::medium(ui, crate::i18n::translate_if_key(label), 14.0)
			.color(colors.text_strong);
		let response = ui
			.scope(|ui| {
				let width = ui.available_width();
				ui.spacing_mut().button_padding = vec2(34.0, 10.0);
				ui.add(
					egui::Button::new(())
						.left_text(text)
						.frame_when_inactive(false)
						.corner_radius(6)
						.min_size(vec2(width, 40.0)),
				)
			})
			.inner;
		icons::paint(
			ui.painter(),
			if set {
				icons::Icon::Pencil
			} else {
				icons::Icon::Smile
			},
			egui::Rect::from_center_size(
				egui::pos2(response.rect.left() + 17.0, response.rect.center().y),
				egui::Vec2::splat(17.0),
			),
			colors.muted,
		);
		if response.clicked() {
			self.open_custom_status_editor();
		}
	}
	/// Opens the custom status dialog on the current status, as its pencil does.
	pub(super) fn open_custom_status_editor(&mut self) {
		self.account_menu
			.draft
			.clone_from(&self.own_presence.custom_status);
		self.account_menu.clear_after = ClearAfter::nearest(self.own_presence_expires);
		self.account_menu.custom_open = true;
	}
	/// Clears the custom status and its deadline; the presence update follows.
	pub(super) fn clear_custom_status(&mut self) {
		self.account_menu.draft.clear();
		self.account_menu.clear_after = ClearAfter::Never;
		self.own_presence_expires = None;
		self.own_presence.expires_at_ms = None;
		if !self.own_presence.custom_status.is_empty() {
			self.own_presence.custom_status.clear();
			self.own_presence_changed = true;
		}
	}
	/// "Clear after" dropdown, matching the presence rows: value on the left, chevron right.
	fn clear_after_row(&mut self, ui: &mut egui::Ui) -> egui::Response {
		let colors = design::palette(ui);
		let chosen = self.account_menu.clear_after;
		let label = design::medium(ui, chosen.label(), 14.0).color(colors.text_strong);
		let response = ui
			.scope(|ui| {
				let width = ui.available_width();
				ui.spacing_mut().button_padding = vec2(14.0, 10.0);
				egui::containers::menu::MenuButton::from_button(
					egui::Button::new(())
						.left_text(label)
						.frame_when_inactive(false)
						.corner_radius(8)
						.min_size(vec2(width, 44.0)),
				)
				.ui(ui, |ui| {
					ui.set_width(220.0_f32.min(ui.ctx().content_rect().width() - 48.0));
					for choice in ClearAfter::ALL {
						let picked = choice == chosen;
						let response = ui.add_sized(
							[ui.available_width(), 36.0],
							egui::Button::new(())
								.left_text(
									design::medium(ui, choice.label(), 14.0)
										.color(colors.text_strong),
								)
								.frame_when_inactive(picked)
								.corner_radius(6),
						);
						if response.clicked() {
							self.account_menu.clear_after = choice;
							ui.close();
						}
					}
				})
				.0
			})
			.inner;
		let rect = response.rect;
		ui.painter().rect_stroke(
			rect,
			8,
			egui::Stroke::new(1.0, colors.border),
			egui::StrokeKind::Inside,
		);
		icons::paint(
			ui.painter(),
			icons::Icon::ChevronDown,
			egui::Rect::from_center_size(
				egui::pos2(rect.right() - 18.0, rect.center().y),
				egui::Vec2::splat(14.0),
			),
			colors.muted,
		);
		response
	}

	/// Live preview, bounded field and footer actions, styled like Discord's dialog.
	fn custom_status_editor(&mut self, ui: &mut egui::Ui, state: &State) {
		let colors = design::palette(ui);
		let draft = self.account_menu.draft.trim().to_owned();
		egui::Frame::new()
			.fill(colors.base)
			.corner_radius(8)
			.inner_margin(12)
			.show(ui, |ui| {
				ui.set_width(ui.available_width());
				ui.horizontal(|ui| {
					ui.spacing_mut().item_spacing.x = 10.0;
					let name = if let Some(user) = &state.user {
						self.avatars.show(ui, user, 40.0, state.demo);
						state
							.own_profile
							.data
							.as_ref()
							.and_then(|p| p.global_name.as_deref())
							.unwrap_or(user.name.as_str())
							.to_owned()
					} else {
						design::avatar(ui, &crate::i18n::translate("pending-show-you"), 40.0);
						crate::i18n::translate("your-account")
					};
					let width = ui.available_width();
					ui.vertical(|ui| {
						ui.set_width(width);
						ui.spacing_mut().item_spacing.y = 2.0;
						ui.add(
							egui::Label::new(
								design::semibold(ui, &name, 15.0).color(colors.text_strong),
							)
							.truncate(),
						);
						let (status, color) = if draft.is_empty() {
							(
								crate::i18n::translate(
									"account-menu-custom-status-editor-no-custom-status",
								),
								colors.muted,
							)
						} else {
							(draft.clone(), colors.text)
						};
						ui.add(
							egui::Label::new(RichText::new(status).size(13.0).color(color))
								.truncate(),
						);
					});
				});
			});
		ui.add_space(16.0);
		let label = dialog::label(ui, "account-menu-custom-status-editor-status-text");
		dialog::input(
			ui,
			egui::TextEdit::singleline(&mut self.account_menu.draft)
				.align(egui::Align2::LEFT_CENTER)
				.id_salt(("account-custom-status", state.generation))
				.hint_text(crate::i18n::translate(
					"account-menu-custom-status-editor-what-s-on-your-mind",
				))
				.char_limit(128),
		)
		.labelled_by(label.id);
		let (_, valid, _) = self.custom_status_draft();
		ui.add_space(4.0);
		// A bounded row: a bare right-to-left layout here takes the dialog's whole remaining
		// height and the size never settles.
		ui.horizontal(|ui| {
			ui.set_height(14.0);
			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				let used = self.account_menu.draft.chars().count();
				ui.label(
					RichText::new(format!("{used}/128"))
						.size(12.0)
						.color(if used > 112 {
							colors.danger
						} else {
							colors.muted
						}),
				);
			});
		});
		ui.add_space(12.0);
		let label = dialog::label(ui, "account-menu-custom-status-editor-clear-after");
		self.clear_after_row(ui).labelled_by(label.id);
		// The deadline is local to this client, so name the moment rather than implying
		// Discord will clear it for you.
		if let Some(clears) = self.account_menu.clear_after.clears_at() {
			dialog::hint(
				ui,
				&format!(
					"{} {clears}.",
					crate::i18n::translate("account-menu-custom-status-editor-serein-clears-it")
				),
			);
		}
		if !valid {
			ui.add_space(8.0);
			dialog::notice(
				ui,
				dialog::Level::Error,
				"account-menu-custom-status-editor-use-up-to-128-characters-without-control-characters",
			);
		}
		if !self.own_presence_status.is_empty() {
			dialog::hint(ui, self.own_presence_status);
		}
	}

	/// Trimmed draft, whether it is publishable, and whether it differs from what is live.
	fn custom_status_draft(&self) -> (String, bool, bool) {
		let draft = self.account_menu.draft.trim().to_owned();
		let valid = model::OwnPresence {
			status: self.own_presence.status,
			custom_status: draft.clone(),
			expires_at_ms: None,
		}
		.valid();
		let changed = draft != self.own_presence.custom_status
			|| (!draft.is_empty()
				&& ClearAfter::nearest(self.own_presence_expires) != self.account_menu.clear_after);
		(draft, valid, changed)
	}

	/// Footer actions: Apply publishes the draft, Clear removes the live status.
	fn custom_status_actions(&mut self, ui: &mut egui::Ui) {
		let (draft, valid, changed) = self.custom_status_draft();
		if ui
			.add_enabled_ui(valid && changed, |ui| {
				dialog::action(
					ui,
					"account-menu-custom-status-actions-apply",
					dialog::Action::Primary,
				)
			})
			.inner
			.clicked()
		{
			self.own_presence.custom_status = draft.clone();
			self.account_menu
				.draft
				.clone_from(&self.own_presence.custom_status);
			self.own_presence_expires = (!draft.is_empty())
				.then(|| self.account_menu.clear_after.deadline())
				.flatten();
			self.own_presence.expires_at_ms = self.own_presence_expires;
			self.own_presence_changed = true;
		}
		let clearable =
			!self.own_presence.custom_status.is_empty() || !self.account_menu.draft.is_empty();
		if ui
			.add_enabled_ui(clearable, |ui| {
				dialog::action(
					ui,
					"account-menu-custom-status-actions-clear",
					dialog::Action::Outline,
				)
			})
			.inner
			.clicked()
		{
			self.clear_custom_status();
		}
	}
}

pub(super) fn presence_status_key(status: PresenceStatus) -> &'static str {
	match status {
		PresenceStatus::Online => "status-online",
		PresenceStatus::Idle => "status-idle",
		PresenceStatus::DoNotDisturb => "status-dnd",
		PresenceStatus::Invisible => "account-menu-status-invisible",
	}
}

/// Presence dot with Discord's status glyphs punched out in the surface colour.
fn presence_dot(
	painter: &egui::Painter,
	center: egui::Pos2,
	radius: f32,
	status: PresenceStatus,
	background: egui::Color32,
) {
	profiles::paint_status(
		painter,
		center,
		radius,
		status.wire(),
		false,
		(background, 0.0),
	);
}

#[cfg(test)]
mod tests {
	use super::*;
	use egui::{Event, Pos2, Rect};

	#[test]
	fn account_details_follow_the_signed_in_identity_and_ignore_foreign_views() {
		let mut state = test_support::demo_state();
		let own = state.user.as_ref().unwrap().clone();
		let mut another = own.clone();
		another.id = model::Id(424_242);
		state.own_profile.data = Some(profiles::synthetic(&own, None));
		state.profile = Some(client_core::profile::ProfileView {
			user: another.id,
			guild: None,
			request: 1,
			loading: false,
			error: None,
			data: Some(profiles::synthetic(&another, None)),
		});
		assert_eq!(account_profile(&state).unwrap().user.id, own.id);
		state.own_profile.error = Some("old editor failure");
		state.profile.as_mut().unwrap().error = Some("another account failure");
		assert_eq!(account_profile_progress(&state), (false, None));
		// Even an inconsistent stale view cannot mask the valid global account metadata.
		state.profile.as_mut().unwrap().user = own.id;
		state.profile.as_mut().unwrap().loading = true;
		assert_eq!(account_profile_progress(&state), (true, None));
		state.profile.as_mut().unwrap().loading = false;
		state.profile.as_mut().unwrap().error = Some("current global read failure");
		assert_eq!(
			account_profile_progress(&state),
			(false, Some("current global read failure"))
		);
		assert_eq!(account_profile(&state).unwrap().user.id, own.id);
		state.profile.as_mut().unwrap().guild = Some(model::Id(10));
		assert_eq!(account_profile_progress(&state), (false, None));
		state.profile.as_mut().unwrap().guild = None;
		state.profile.as_mut().unwrap().data = Some(profiles::synthetic(&own, Some(model::Id(10))));
		assert!(account_profile(&state).unwrap().guild.is_none());
		state.profile.as_mut().unwrap().user = another.id;
		state.profile.as_mut().unwrap().data = Some(profiles::synthetic(&another, None));
		// Switching identities must replace the whole summary, including banner and bio.
		state.user = Some(another.clone());
		assert_eq!(account_profile(&state).unwrap().user.id, another.id);
		state.profile = None;
		assert!(
			account_profile(&state).is_none(),
			"the previous account's edit data is hidden"
		);
		state.user = None;
		assert!(account_profile(&state).is_none());
	}

	fn render_frame(
		ctx: &egui::Context,
		view: &mut MessagingUi,
		state: &mut State,
		size: egui::Vec2,
		events: Vec<Event>,
	) -> egui::FullOutput {
		ctx.run_ui(
			egui::RawInput {
				screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
				events,
				..Default::default()
			},
			|ui| {
				egui::Panel::bottom("test-account-footer").show(ui, |ui| {
					let mut commands = vec![];
					view.account_card(ui, state, &mut commands);
					assert!(commands.is_empty());
				});
			},
		)
	}
	fn frame(
		ctx: &egui::Context,
		view: &mut MessagingUi,
		state: &mut State,
		size: egui::Vec2,
		events: Vec<Event>,
	) -> Vec<(String, Rect)> {
		let mut text = vec![];
		let output = render_frame(ctx, view, state, size, events);
		fn collect(shape: &egui::Shape, labels: &mut Vec<(String, Rect)>) {
			match shape {
				egui::Shape::Text(t) => labels.push((
					t.galley.job.text.clone(),
					Rect::from_min_size(t.pos, t.galley.size()),
				)),
				egui::Shape::Vec(shapes) => {
					for shape in shapes {
						collect(shape, labels);
					}
				}
				_ => {}
			}
		}
		for shape in &output.shapes {
			collect(&shape.shape, &mut text);
		}
		assert!(output.platform_output.commands.is_empty());
		output.drop_without_applying_deltas();
		text
	}

	#[test]
	fn account_frame_preserves_complete_artwork_outside_scrolling_details() {
		fn meshes<'a>(shape: &'a egui::Shape, found: &mut Vec<&'a egui::epaint::Mesh>) {
			match shape {
				egui::Shape::Mesh(mesh) => found.push(mesh),
				egui::Shape::Vec(shapes) => {
					for shape in shapes {
						meshes(shape, found);
					}
				}
				_ => {}
			}
		}
		for size in [vec2(340.0, 760.0), vec2(760.0, 520.0)] {
			let ctx = egui::Context::default();
			design::apply(&ctx);
			let mut state = test_support::demo_state();
			let own = state.user.as_ref().unwrap().clone();
			let profile = profiles::synthetic(&own, None);
			let mut view = MessagingUi {
				cosmetics: model::CosmeticPreferences::SHOWN,
				..Default::default()
			};
			view.avatars.cosmetics = view.cosmetics;
			let artwork = view.avatars.frame_shapes(
				&ctx,
				profile.frame.as_deref().unwrap(),
				Rect::from_min_size(Pos2::new(100.0, 100.0), vec2(300.0, 300.0)),
				Rect::EVERYTHING,
				true,
				true,
			);
			let mut source = Vec::new();
			for shape in &artwork {
				meshes(shape, &mut source);
			}
			let texture_ids: std::collections::HashSet<_> =
				source.iter().map(|mesh| mesh.texture_id).collect();
			assert!(!texture_ids.is_empty());
			state.own_profile.data = Some(profile);
			view.preview_account_menu(state.generation);
			for _ in 0..3 {
				frame(&ctx, &mut view, &mut state, size, vec![]);
			}
			let output = render_frame(&ctx, &mut view, &mut state, size, vec![]);
			let viewport = Rect::from_min_size(Pos2::ZERO, size);
			let mut drawn = std::collections::HashSet::new();
			for shape in &output.shapes {
				let mut found = Vec::new();
				meshes(&shape.shape, &mut found);
				for mesh in found
					.iter()
					.filter(|mesh| texture_ids.contains(&mesh.texture_id))
				{
					drawn.insert(mesh.texture_id);
					let uv = mesh
						.vertices
						.iter()
						.fold(Rect::NOTHING, |mut bounds, vertex| {
							bounds.extend_with(vertex.uv);
							assert!(viewport.contains(vertex.pos));
							assert!(shape.clip_rect.contains(vertex.pos));
							bounds
						});
					// Cropping the frame to the detail ScrollArea removes these outside UVs.
					assert!(uv.min.distance(Pos2::ZERO) < 0.001, "{uv:?}");
					assert!(uv.max.distance(Pos2::new(1.0, 1.0)) < 0.001, "{uv:?}");
				}
			}
			assert_eq!(drawn, texture_ids);
			output.drop_without_applying_deltas();
			view.cosmetics.frames = false;
			view.avatars.cosmetics = view.cosmetics;
			for _ in 0..3 {
				frame(&ctx, &mut view, &mut state, size, vec![]);
			}
			let output = render_frame(&ctx, &mut view, &mut state, size, vec![]);
			for shape in &output.shapes {
				let mut found = Vec::new();
				meshes(&shape.shape, &mut found);
				assert!(
					found
						.iter()
						.all(|mesh| !texture_ids.contains(&mesh.texture_id))
				);
			}
			output.drop_without_applying_deltas();
		}
	}
	fn click(
		ctx: &egui::Context,
		view: &mut MessagingUi,
		state: &mut State,
		size: egui::Vec2,
		position: Pos2,
	) {
		for pressed in [true, false] {
			frame(
				ctx,
				view,
				state,
				size,
				vec![
					Event::PointerMoved(position),
					Event::PointerButton {
						pos: position,
						button: egui::PointerButton::Primary,
						pressed,
						modifiers: egui::Modifiers::NONE,
					},
				],
			);
		}
	}
	fn locate(text: &[(String, Rect)], label: &str) -> Pos2 {
		text.iter()
			.find(|(text, _)| text == label)
			.unwrap_or_else(|| panic!("Missing {label}: {text:?}"))
			.1
			.center()
	}
	fn alt_account() -> model::SavedAccount {
		model::SavedAccount {
			id: model::Id(424_242),
			name: "synthetic-alt".into(),
			display: Some("Synthetic Alt".into()),
			avatar: None,
			discriminator: 0,
			has_token: true,
		}
	}

	#[test]
	fn switcher_lists_other_accounts_without_the_signed_in_one() {
		let size = vec2(340.0, 900.0);
		for demo in [true, false] {
			let ctx = egui::Context::default();
			design::apply(&ctx);
			let mut state = test_support::demo_state();
			state.demo = demo;
			let own = state.user.as_ref().unwrap().clone();
			state.own_profile.data = Some(profiles::synthetic(&own, None));
			// The signed-in account is remembered too, and never offered as a switch target.
			let mut view = MessagingUi {
				accounts: vec![
					model::SavedAccount {
						id: own.id,
						name: own.name.clone(),
						display: Some("Signed in already".into()),
						avatar: None,
						discriminator: own.discriminator,
						has_token: true,
					},
					alt_account(),
					// Recorded before the handle was kept: the display name must not repeat.
					model::SavedAccount {
						id: model::Id(424_243),
						name: "Synthetic Stale".into(),
						display: Some("Synthetic Stale".into()),
						..alt_account()
					},
				],
				..Default::default()
			};
			view.preview_account_menu(state.generation);
			for _ in 0..3 {
				frame(&ctx, &mut view, &mut state, size, vec![]);
			}
			let text = frame(&ctx, &mut view, &mut state, size, vec![]);
			let listed = |label: &str| text.iter().any(|(value, _)| value == label);
			assert!(listed("SWITCH ACCOUNTS"));
			assert!(listed("Synthetic Alt"));
			assert!(listed("synthetic-alt"));
			assert!(listed("Add an account"));
			assert_eq!(
				text.iter()
					.filter(|(value, _)| value == "Synthetic Stale")
					.count(),
				1,
				"an entry without a known handle must not name the account twice"
			);
			// The signed-in account is remembered but never listed as a switch target.
			assert!(!listed("Signed in already"));
			click(
				&ctx,
				&mut view,
				&mut state,
				size,
				locate(&text, "Add an account"),
			);
			assert!(view.add_account_requested);
			assert!(!view.account_menu.open);
			assert_eq!(view.switch_account_requested, None);
		}
	}

	#[test]
	fn switcher_row_separates_switching_from_forgetting() {
		let account = alt_account();
		let row = |view: &mut MessagingUi, ctx: &egui::Context, events: Vec<Event>| {
			let output = ctx.run_ui(
				egui::RawInput {
					screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(300.0, 120.0))),
					events,
					..Default::default()
				},
				|ui| {
					ui.scope_builder(
						egui::UiBuilder::new()
							.max_rect(Rect::from_min_size(Pos2::ZERO, vec2(300.0, 44.0))),
						|ui| view.account_switcher_row(ui, &account, false),
					);
				},
			);
			output.drop_without_applying_deltas();
		};
		// The row spans the full width; the trailing bin owns only its own corner.
		for (position, switches) in [
			(Pos2::new(120.0, 22.0), true),
			(Pos2::new(280.0, 22.0), false),
		] {
			let ctx = egui::Context::default();
			design::apply(&ctx);
			let mut view = MessagingUi::default();
			// The pointer lands on widgets registered by an earlier frame.
			row(&mut view, &ctx, vec![]);
			for pressed in [true, false] {
				row(
					&mut view,
					&ctx,
					vec![
						Event::PointerMoved(position),
						Event::PointerButton {
							pos: position,
							button: egui::PointerButton::Primary,
							pressed,
							modifiers: egui::Modifiers::NONE,
						},
					],
				);
			}
			assert_eq!(
				view.switch_account_requested,
				switches.then_some(account.id)
			);
			assert_eq!(
				view.forget_account_requested,
				(!switches).then_some(account.id)
			);
			// The popout owns `open`; rows only ask it to close.
			assert!(view.account_menu.close);
		}
	}

	#[test]
	fn account_menu_opens_applies_clears_and_closes_across_themes_and_sizes() {
		for light in [false, true] {
			for size in [vec2(340.0, 760.0), vec2(760.0, 520.0)] {
				let ctx = egui::Context::default();
				ctx.set_theme(if light {
					egui::ThemePreference::Light
				} else {
					egui::ThemePreference::Dark
				});
				design::apply(&ctx);
				let mut state = test_support::demo_state();
				let own = state.user.as_ref().unwrap().clone();
				state.own_profile.data = Some(profiles::synthetic(&own, None));
				let mut view = MessagingUi::default();
				frame(&ctx, &mut view, &mut state, size, vec![]);
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				if light {
					for key in [egui::Key::Tab, egui::Key::Enter] {
						frame(
							&ctx,
							&mut view,
							&mut state,
							size,
							vec![Event::Key {
								key,
								physical_key: None,
								pressed: true,
								repeat: false,
								modifiers: egui::Modifiers::NONE,
							}],
						);
					}
				} else {
					click(&ctx, &mut view, &mut state, size, locate(&text, &own.name));
				}
				for _ in 0..3 {
					frame(&ctx, &mut view, &mut state, size, vec![]);
				}
				assert!(view.account_menu.open);
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				click(&ctx, &mut view, &mut state, size, locate(&text, "Online"));
				for _ in 0..3 {
					frame(&ctx, &mut view, &mut state, size, vec![]);
				}
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				for status in PresenceStatus::ALL {
					let p = locate(&text, status.label());
					assert!(Rect::from_min_size(Pos2::ZERO, size).contains(p));
				}
				click(
					&ctx,
					&mut view,
					&mut state,
					size,
					locate(&text, "Invisible"),
				);
				assert_eq!(view.own_presence.status, PresenceStatus::Invisible);
				assert!(std::mem::take(&mut view.own_presence_changed));
				view.account_menu.open = true;
				for _ in 0..3 {
					frame(&ctx, &mut view, &mut state, size, vec![]);
				}
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				click(
					&ctx,
					&mut view,
					&mut state,
					size,
					locate(&text, "Set a custom status"),
				);
				assert!(view.account_menu.custom_open);
				// A bounded draft is separate from the value published by the host.
				view.account_menu.draft = "  Synthetic status 🌙  ".into();
				assert!(view.own_presence.custom_status.is_empty());
				// Tall viewport keeps the editor in view; narrow/short mode additionally checks scrolling above.
				let size = vec2(size.x, 900.0);
				for _ in 0..3 {
					frame(&ctx, &mut view, &mut state, size, vec![]);
				}
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				click(&ctx, &mut view, &mut state, size, locate(&text, "Apply"));
				assert_eq!(view.own_presence.custom_status, "Synthetic status 🌙");
				assert!(std::mem::take(&mut view.own_presence_changed));
				let text = frame(&ctx, &mut view, &mut state, size, vec![]);
				click(&ctx, &mut view, &mut state, size, locate(&text, "Clear"));
				assert!(view.own_presence.custom_status.is_empty());
				assert!(view.own_presence_changed);
				frame(
					&ctx,
					&mut view,
					&mut state,
					size,
					vec![Event::Key {
						key: egui::Key::Escape,
						physical_key: None,
						pressed: true,
						repeat: false,
						modifiers: egui::Modifiers::NONE,
					}],
				);
				assert!(!view.account_menu.custom_open);
				view.account_menu.open = true;
				view.account_menu.draft = "Old account draft".into();
				state.generation += 1;
				frame(&ctx, &mut view, &mut state, size, vec![]);
				assert!(!view.account_menu.open && view.account_menu.draft.is_empty());
				assert!(view.take_avatar_requests().is_empty());
			}
		}
	}
}
