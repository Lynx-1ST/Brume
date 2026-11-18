use crate::{MessagingUi, design, voice::VoiceRowAction};
use client_core::State;
use egui::{Event, Modifiers, PointerButton, Pos2};
use model::Id;

fn frame(
	ctx: &egui::Context,
	view: &mut MessagingUi,
	state: &State,
	events: Vec<Event>,
) -> (egui::Response, Option<VoiceRowAction>) {
	let mut result = None;
	ctx.run_ui(
		egui::RawInput {
			screen_rect: Some(egui::Rect::from_min_size(
				Pos2::ZERO,
				egui::vec2(300.0, 160.0),
			)),
			events,
			..Default::default()
		},
		|ui| {
			result = Some(view.voice_channel_button(
				ui,
				state,
				state.channel(Id(20)).unwrap(),
				true,
				false,
			));
		},
	)
	.drop_without_applying_deltas();
	result.unwrap()
}

#[test]
fn voice_row_actions_hide_on_idle_and_chat_never_joins_the_call() {
	let ctx = egui::Context::default();
	design::apply(&ctx);
	let mut state = test_support::chat_demo_state();
	state
		.channels
		.iter_mut()
		.find(|channel| channel.id == Id(20))
		.unwrap()
		.kind = 2;
	let mut permissions = test_support::permission_snapshot(&state);
	for guild in &mut permissions.guilds {
		guild.owner = state.user.as_ref().map(|user| user.id);
	}
	state.permissions.replace(permissions).unwrap();
	let mut view = MessagingUi::default();
	let (row, action) = frame(&ctx, &mut view, &state, vec![]);
	assert!(action.is_none());
	let chat = row.id.with("voice-open-chat");
	let invite = row.id.with("channel-menu-context-invite-to-channel");
	assert!(ctx.read_response(chat).is_none());
	row.request_focus();
	frame(&ctx, &mut view, &state, vec![Event::PointerGone]);
	assert!(
		ctx.read_response(chat).is_none(),
		"selection/focus alone must not show actions"
	);
	for _ in 0..2 {
		frame(
			&ctx,
			&mut view,
			&state,
			vec![Event::PointerMoved(row.rect.center())],
		);
	}
	let target = ctx
		.read_response(chat)
		.expect("hover reveals chat")
		.rect
		.center();
	assert!(
		ctx.read_response(invite).is_some(),
		"invite uses the same hover boundary"
	);
	let mut chosen = None;
	for pressed in [true, false] {
		let (_, action) = frame(
			&ctx,
			&mut view,
			&state,
			vec![
				Event::PointerMoved(target),
				Event::PointerButton {
					pos: target,
					button: PointerButton::Primary,
					pressed,
					modifiers: Modifiers::NONE,
				},
			],
		);
		chosen = chosen.or(action);
	}
	assert!(matches!(chosen, Some(VoiceRowAction::Chat)));
	let mut commands = vec![];
	view.open_voice_chat(&mut state, Id(20), &mut commands);
	assert!(view.voice_chat_open && state.selected == Some(Id(20)));
	assert!(state.voice.active.is_none() && view.voice_join.is_none());
	for _ in 0..2 {
		frame(&ctx, &mut view, &state, vec![Event::PointerGone]);
	}
	assert!(ctx.read_response(chat).is_none() && ctx.read_response(invite).is_none());
}
