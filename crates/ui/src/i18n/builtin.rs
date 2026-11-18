//! Explicit keys for built-in labels and status text owned by the application.
//! This is never applied to user, message, server or remote service content.

pub(super) fn key(text: &str) -> Option<&'static str> {
	match text {
		"Stream PiP" => Some("voice-pip-title"),
		"Open Picture-in-Picture" => Some("voice-pip-open"),
		"Close Picture-in-Picture" => Some("voice-pip-close"),
		"Return to voice channel" => Some("voice-pip-return"),
		"Your screen" => Some("voice-pip-own"),
		"Choose stream" => Some("voice-pip-source"),
		"Mute stream audio" => Some("voice-pip-mute"),
		"Unmute stream audio" => Some("voice-pip-unmute"),
		"Waiting for stream video…" => Some("voice-pip-waiting"),
		"Stop watching" => Some("voice-pip-stop"),
		"Open Chat" => Some("voice-open-chat"),
		"Voice Connected" => Some("ui-builtin-voice-connected"),
		"Voice connected" => Some("ui-builtin-voice-connected-a81da2"),
		"Voice preview" => Some("ui-builtin-voice-preview"),
		"Call failed" => Some("ui-builtin-call-failed"),
		"Connecting…" => Some("ui-builtin-connecting"),
		"Share a screen or window" => Some("ui-builtin-share-a-screen-or-window"),
		"Share your screen" => Some("screen-show-share-your-screen"),
		"Stop sharing" => Some("ui-builtin-stop-sharing"),
		"Stop sharing your screen" => Some("ui-builtin-stop-sharing-your-screen"),
		"Exit fullscreen" => Some("ui-builtin-exit-fullscreen"),
		"Exit fullscreen (Esc)" => Some("ui-builtin-exit-fullscreen-esc"),
		"Streaming Now" => Some("ui-builtin-streaming-now"),
		"This stream is no longer live" => Some("ui-builtin-this-stream-is-no-longer-live"),
		"Could not join this stream's voice channel" => {
			Some("ui-builtin-could-not-join-this-stream-s-voice-channel")
		}
		"Leave the current call before watching this stream" => {
			Some("ui-builtin-leave-the-current-call-before-watching-this-stream")
		}
		"The four-stream viewing limit is reached" => {
			Some("ui-builtin-the-four-stream-viewing-limit-is-reached")
		}
		"Up to four streams can be watched at once" => {
			Some("ui-builtin-up-to-four-streams-can-be-watched-at-once")
		}
		"This stream is not available in the connected voice channel" => {
			Some("ui-builtin-this-stream-is-not-available-in-the-connected-voice-channel")
		}
		"The stream is not available" => Some("ui-builtin-the-stream-is-not-available"),
		"Your screen · local preview" => Some("ui-builtin-your-screen-local-preview"),
		"Screen share you are watching" => Some("ui-builtin-screen-share-you-are-watching"),
		"Watch Stream" => Some("ui-builtin-watch-stream"),
		"Click or press Escape to return to the grid" => {
			Some("ui-builtin-click-or-press-escape-to-return-to-the-grid")
		}
		"Click to enlarge" => Some("ui-builtin-click-to-enlarge"),
		"Connecting to the stream…" => Some("ui-builtin-connecting-to-the-stream"),
		"Stop receiving this screen share" => Some("ui-builtin-stop-receiving-this-screen-share"),
		"Answer call" => Some("ui-builtin-answer-call"),
		"Start voice call" => Some("ui-builtin-start-voice-call"),
		"Join audio. Your microphone starts after the call is secured." => {
			Some("ui-builtin-join-audio-your-microphone-starts-after-the-call-is-secured")
		}
		"Join to listen. Speaking is unavailable in this channel." => {
			Some("ui-builtin-join-to-listen-speaking-is-unavailable-in-this-channel")
		}
		"Voice Isolation" => Some("ui-builtin-voice-isolation"),
		"Studio" => Some("ui-builtin-studio"),
		"Custom" => Some("ui-builtin-custom"),
		"RNNoise suppression, echo cancellation and automatic gain for speech." => {
			Some("ui-builtin-rnnoise-suppression-echo-cancellation-and-automatic-gain-for-speech")
		}
		"Open microphone without suppression, echo cancellation or automatic gain." => Some(
			"ui-builtin-open-microphone-without-suppression-echo-cancellation-or-automatic-gain",
		),
		"Turn on microphone" => Some("ui-builtin-turn-on-microphone"),
		"Turn off microphone" => Some("ui-builtin-turn-off-microphone"),
		"Voice settings" => Some("ui-builtin-voice-settings"),
		"Microphone and speaker settings" => Some("ui-builtin-microphone-and-speaker-settings"),
		"Turn off camera" => Some("ui-builtin-turn-off-camera"),
		"Turn on camera" => Some("ui-builtin-turn-on-camera"),
		"Stop sharing your camera" => Some("ui-builtin-stop-sharing-your-camera"),
		"Camera is off in the offline preview" => {
			Some("ui-builtin-camera-is-off-in-the-offline-preview")
		}
		"Camera capture is unavailable on this platform" => {
			Some("ui-builtin-camera-capture-is-unavailable-on-this-platform")
		}
		"Choose a camera" => Some("ui-builtin-choose-a-camera"),
		"More options" => Some("ui-builtin-more-options"),
		"More call options" => Some("ui-builtin-more-call-options"),
		"Hide participants" => Some("ui-builtin-hide-participants"),
		"Show participants" => Some("ui-builtin-show-participants"),
		"Hide the participant strip under the enlarged video" => {
			Some("ui-builtin-hide-the-participant-strip-under-the-enlarged-video")
		}
		"Show the other participants under the enlarged video" => {
			Some("ui-builtin-show-the-other-participants-under-the-enlarged-video")
		}
		"Screen sharing requires a connected call and video permission on a supported desktop." => {
			Some(
				"ui-builtin-screen-sharing-requires-a-connected-call-and-video-permission-on-a-supported-desktop",
			)
		}
		"Public" => Some("ui-builtin-public"),
		"Joined private" => Some("ui-builtin-joined-private"),
		"Private" => Some("ui-builtin-private"),
		"Enlarge image" => Some("ui-builtin-enlarge-image"),
		"Loading full quality" => Some("ui-builtin-loading-full-quality"),
		"Loading image" => Some("ui-builtin-loading-image"),
		"Full quality unavailable" => Some("ui-builtin-full-quality-unavailable"),
		"Close (Esc)" => Some("ui-builtin-close-esc"),
		"Previous (←)" => Some("ui-builtin-previous"),
		"Next (→)" => Some("ui-builtin-next"),
		"Close" => Some("archives-show-close"),
		"Restore" => Some("settings-activity-restore"),
		"Maximize" => Some("ui-builtin-maximize"),
		"Minimize" => Some("app-menu-minimize"),
		"Next game" => Some("ui-builtin-next-game"),
		"Select emoji" => Some("ui-builtin-select-emoji"),
		"Skip this question" => Some("ui-builtin-skip-this-question"),
		"Press keys…" => Some("ui-builtin-press-keys"),
		"Control your microphone and incoming audio during a connected call." => {
			Some("ui-builtin-control-your-microphone-and-incoming-audio-during-a-connected-call")
		}
		"Proxy authentication" => Some("ui-builtin-proxy-authentication"),
		"Credentials stay in Serein's OS credential store and are never shared with the plugin. Apply the proxy URL before saving credentials." => {
			Some(
				"ui-builtin-credentials-stay-in-serein-s-os-credential-store-and-are-never-shared-with-the-plugin-apply-the-prox",
			)
		}
		"HTTP proxy authentication is unencrypted: anyone observing the connection to the proxy can recover these credentials. Use an HTTPS proxy for encrypted authentication." => {
			Some(
				"ui-builtin-http-proxy-authentication-is-unencrypted-anyone-observing-the-connection-to-the-proxy-can-recover-th",
			)
		}
		"Save credentials" => Some("ui-builtin-save-credentials"),
		"Remove saved credentials" => Some("ui-builtin-remove-saved-credentials"),
		"Choose URL mode to configure proxy authentication." => {
			Some("ui-builtin-choose-url-mode-to-configure-proxy-authentication")
		}
		"PROFILE PREVIEW" => Some("ui-builtin-profile-preview"),
		"Your activity goes here" => Some("ui-builtin-your-activity-goes-here"),
		"Enter an application ID and activity name, then choose Preview." => {
			Some("ui-builtin-enter-an-application-id-and-activity-name-then-choose-preview")
		}
		"Artwork resolves when applied. This draft preview uses placeholders." => {
			Some("ui-builtin-artwork-resolves-when-applied-this-draft-preview-uses-placeholders")
		}
		"Activity sharing is off. Enable it in Settings > Registered Games to publish your presence." => {
			Some(
				"ui-builtin-activity-sharing-is-off-enable-it-in-settings-registered-games-to-publish-your-presence",
			)
		}
		"Offline demo. Nothing is published." => {
			Some("ui-builtin-offline-demo-nothing-is-published")
		}
		"Offline. Your saved activity will wait for a connection." => {
			Some("ui-builtin-offline-your-saved-activity-will-wait-for-a-connection")
		}
		"Enable activity sharing in Settings > Registered Games. Invisible hides your activity." => {
			Some(
				"ui-builtin-enable-activity-sharing-in-settings-registered-games-invisible-hides-your-activity",
			)
		}
		"Elapsed since starting; applying edits keeps the timer." => {
			Some("ui-builtin-elapsed-since-starting-applying-edits-keeps-the-timer")
		}
		"Timer starts at local midnight." => Some("ui-builtin-timer-starts-at-local-midnight"),
		"1 Hour" => Some("ui-builtin-1-hour"),
		"24 Hours" => Some("ui-builtin-24-hours"),
		"3 Days" => Some("ui-builtin-3-days"),
		"1 Week" => Some("ui-builtin-1-week"),
		"Recommended" => Some("ui-builtin-recommended"),
		"New Message in the channel I'm currently reading" => {
			Some("ui-builtin-new-message-in-the-channel-i-m-currently-reading")
		}
		"Incoming Ring" => Some("ui-builtin-incoming-ring"),
		"Outgoing Ring" => Some("ui-builtin-outgoing-ring"),
		"Voice" => Some("ui-builtin-voice"),
		"Stream audio" => Some("voice-stream-tile-stream-audio"),
		"Join Voice" => Some("invites-show-join-voice"),
		"Join call" => Some("voice-call-bar-join-call"),
		"Unmute" => Some("voice-voice-participant-menu-unmute"),
		"Mute" => Some("voice-voice-participant-menu-mute"),
		"Speaking is unavailable in this channel." => {
			Some("voice-mute-toggle-speaking-is-unavailable-in-this-channel")
		}
		"Camera settings" => Some("voice-voice-card-section-camera-settings"),
		"Undeafen" => Some("voice-mute-toggle-undeafen"),
		"Deafen" => Some("voice-mute-toggle-deafen"),
		"Stream volume" => Some("voice-stream-audio-controls-stream-volume"),
		"Switch to this voice channel" => Some("voice-built-in-switch-to-this-voice-channel"),
		"Camera requires H264 support from the voice server" => {
			Some("voice-built-in-camera-requires-h264-support-from-the-voice-server")
		}
		"Camera is unavailable with current channel permissions" => {
			Some("voice-built-in-camera-is-unavailable-with-current-channel-permissions")
		}
		"Turn off noise suppression" => Some("voice-built-in-turn-off-noise-suppression"),
		"Turn on noise suppression" => Some("voice-built-in-turn-on-noise-suppression"),
		"Noise suppression reduces keyboard noise, breathing and fans locally." => Some(
			"voice-built-in-noise-suppression-reduces-keyboard-noise-breathing-and-fans-locally",
		),
		"Noise suppression is unavailable in this build or preview." => {
			Some("voice-built-in-noise-suppression-is-unavailable-in-this-build-or-preview")
		}
		"Fullscreen" => Some("voice-built-in-fullscreen"),
		"View this screen share in fullscreen" => {
			Some("voice-built-in-view-this-screen-share-in-fullscreen")
		}
		"Microphone gain" => Some("voice-voice-popup-content-microphone-gain"),
		"Speaker volume" => Some("voice-voice-popup-content-speaker-volume"),
		"Call switch cancelled: previous call did not finish disconnecting. Try joining again." => {
			Some(
				"voice-built-in-call-switch-cancelled-previous-call-did-not-finish-disconnecting-try-joining-again",
			)
		}
		"Unable to create this server right now." => {
			Some("ui-owned-unable-to-create-this-server-right-now")
		}
		"Enter a valid Discord invite link or invite code." => {
			Some("ui-owned-enter-a-valid-discord-invite-link-or-invite-code")
		}
		"Unable to proceed. Check your connection or try again after the current request." => Some(
			"ui-owned-unable-to-proceed-check-your-connection-or-try-again-after-the-current-request",
		),
		"Prepared icon is too large" => Some("ui-owned-prepared-icon-is-too-large"),
		"Application data directory is unavailable." => {
			Some("status-owned-application-data-directory-is-unavailable")
		}
		"Result discarded because the conversation or draft changed." => {
			Some("status-owned-result-discarded-because-the-conversation-or-draft-changed")
		}
		"Package inspected. Review its source and capabilities before enabling." => Some(
			"status-owned-package-inspected-review-its-source-and-capabilities-before-enabling",
		),
		"Extension enabled." => Some("status-owned-extension-enabled"),
		"Theme exported." => Some("status-owned-theme-exported"),
		"Extension file selection ended." => Some("status-owned-extension-file-selection-ended"),
		"Refresh the catalog or import this package again." => {
			Some("status-owned-refresh-the-catalog-or-import-this-package-again")
		}
		"Some plugin events were skipped because the event queue was full." => {
			Some("status-owned-some-plugin-events-were-skipped-because-the-event-queue-was-full")
		}
		"Disconnected" => Some("status-owned-disconnected"),
		"Preview only; this font is not saved." => {
			Some("status-owned-preview-only-this-font-is-not-saved")
		}
		"Loading font…" => Some("status-owned-loading-font"),
		"Voice calls are unavailable in the offline preview" => {
			Some("status-owned-voice-calls-are-unavailable-in-the-offline-preview")
		}
		"Local storage worker stopped. Restart Serein to save fonts." => {
			Some("status-owned-local-storage-worker-stopped-restart-serein-to-save-fonts")
		}
		"No saved login for that account on this device. Use another account to sign in again, or forget it with ×." => {
			Some(
				"status-owned-no-saved-login-for-that-account-on-this-device-use-another-account-to-sign-in-again-or-for",
			)
		}
		"Could not save this account for the switcher; sign in again to retry" => {
			Some("status-owned-could-not-save-this-account-for-the-switcher-sign-in-again-to-retry")
		}
		"Could not save login; this session will not restore automatically" => {
			Some("status-owned-could-not-save-login-this-session-will-not-restore-automatically")
		}
		"Could not queue saved login; session only" => {
			Some("status-owned-could-not-queue-saved-login-session-only")
		}
		"Reconnecting to Discord…" => Some("status-owned-reconnecting-to-discord"),
		"Proxy credentials loaded. Future API requests use this setting." => {
			Some("status-owned-proxy-credentials-loaded-future-api-requests-use-this-setting")
		}
		"Proxy credential operation ended; retry saving or removing credentials." => Some(
			"status-owned-proxy-credential-operation-ended-retry-saving-or-removing-credentials",
		),
		"Invalid proxy URL" => Some("status-owned-invalid-proxy-url"),
		"Demo mode does not read or save proxy credentials." => {
			Some("status-owned-demo-mode-does-not-read-or-save-proxy-credentials")
		}
		"Wait for the current credential operation." => {
			Some("status-owned-wait-for-the-current-credential-operation")
		}
		"Apply this URL in the plugin before saving credentials." => {
			Some("status-owned-apply-this-url-in-the-plugin-before-saving-credentials")
		}
		"Use a username of 1-256 bytes without a colon, and a password up to 1024 bytes; control characters are unsupported." => {
			Some(
				"status-owned-use-a-username-of-1-256-bytes-without-a-colon-and-a-password-up-to-1024-bytes-control-char",
			)
		}
		"Enter a valid HTTP/HTTPS proxy URL first." => {
			Some("status-owned-enter-a-valid-http-https-proxy-url-first")
		}
		"Screen sharing stopped" => Some("status-owned-screen-sharing-stopped"),
		"Looking for screens and windows…" => {
			Some("status-owned-looking-for-screens-and-windows")
		}
		"Could not start screen source discovery" => {
			Some("status-owned-could-not-start-screen-source-discovery")
		}
		"Screen sharing is unavailable on this platform" => {
			Some("status-owned-screen-sharing-is-unavailable-on-this-platform")
		}
		"Screen source discovery stopped" => Some("status-owned-screen-source-discovery-stopped"),
		"Previous screen share is still closing" => {
			Some("status-owned-previous-screen-share-is-still-closing")
		}
		"Requesting screen-share connection…" => {
			Some("status-owned-requesting-screen-share-connection")
		}
		"Screen sharing requires the current connected call" => {
			Some("status-owned-screen-sharing-requires-the-current-connected-call")
		}
		"Discord did not acknowledge stopping screen share; leave the call before sharing again" => {
			Some(
				"status-owned-discord-did-not-acknowledge-stopping-screen-share-leave-the-call-before-sharing-again",
			)
		}
		"Connecting screen-share transport…" => {
			Some("status-owned-connecting-screen-share-transport")
		}
		"Connecting to Discord…" => Some("status-owned-connecting-to-discord"),
		"Signed out of this account; its saved login is kept" => {
			Some("status-owned-signed-out-of-this-account-its-saved-login-is-kept")
		}
		"Saved account removed from this device" => {
			Some("status-owned-saved-account-removed-from-this-device")
		}
		"Saved-login restore cancelled" => Some("status-owned-saved-login-restore-cancelled"),
		"Sign in through Discord; saved-login lookup stopped" => {
			Some("status-owned-sign-in-through-discord-saved-login-lookup-stopped")
		}
		"Waiting for Discord login" => Some("status-owned-waiting-for-discord-login"),
		"Updates have not been checked on this channel yet." => {
			Some("status-owned-updates-have-not-been-checked-on-this-channel-yet")
		}
		"Update downloaded and verified. Restart when you are ready." => {
			Some("status-owned-update-downloaded-and-verified-restart-when-you-are-ready")
		}
		"Update ready. Close Serein to install and restart." => {
			Some("status-owned-update-ready-close-serein-to-install-and-restart")
		}
		"The update worker stopped. Please try again." => {
			Some("status-owned-the-update-worker-stopped-please-try-again")
		}
		"Preparing to restart…" => Some("status-owned-preparing-to-restart"),
		"Please wait one minute between update checks." => {
			Some("status-owned-please-wait-one-minute-between-update-checks")
		}
		"Checking for updates…" => Some("status-owned-checking-for-updates"),
		"Stopped watching" => Some("status-owned-stopped-watching"),
		"Connect the call before watching a stream" => {
			Some("status-owned-connect-the-call-before-watching-a-stream")
		}
		"Requesting the stream…" => Some("status-owned-requesting-the-stream"),
		"Watching the stream" => Some("status-owned-watching-the-stream"),
		"Forward unavailable: check destinations, connection and message limits" => Some(
			"status-owned-forward-unavailable-check-destinations-connection-and-message-limits",
		),
		"Verify to join this server" => Some("status-owned-verify-to-join-this-server"),
		"Wait for the previous message edit to finish" => {
			Some("status-owned-wait-for-the-previous-message-edit-to-finish")
		}
		"Edit could not be applied; reload this conversation" => {
			Some("status-owned-edit-could-not-be-applied-reload-this-conversation")
		}
		"Message edit failed; your edit was kept for retry" => {
			Some("status-owned-message-edit-failed-your-edit-was-kept-for-retry")
		}
		"Wait for the previous pin action to finish" => {
			Some("status-owned-wait-for-the-previous-pin-action-to-finish")
		}
		"Pin action failed; the previous state was restored" => {
			Some("status-owned-pin-action-failed-the-previous-state-was-restored")
		}
		"You're in · welcome to the server" => {
			Some("status-owned-you-re-in-welcome-to-the-server")
		}
		"Wait for the current poll action" => Some("status-owned-wait-for-the-current-poll-action"),
		"Poll action is unavailable with the current permissions or poll state" => Some(
			"status-owned-poll-action-is-unavailable-with-the-current-permissions-or-poll-state",
		),
		"Profile updated; use Reload history to refresh this conversation" => {
			Some("status-owned-profile-updated-use-reload-history-to-refresh-this-conversation")
		}
		"Reaction limit reached" => Some("status-owned-reaction-limit-reached"),
		"Voice moderation request accepted" => {
			Some("status-owned-voice-moderation-request-accepted")
		}
		"Enter a Discord username of at most 32 characters." => {
			Some("status-owned-enter-a-discord-username-of-at-most-32-characters")
		}
		"Enter a Discord username: 2–32 letters, numbers, underscores or periods." => Some(
			"status-owned-enter-a-discord-username-2-32-letters-numbers-underscores-or-periods",
		),
		"Already friends or a request is pending. Check the Pending tab." => {
			Some("status-owned-already-friends-or-a-request-is-pending-check-the-pending-tab")
		}
		"Choose between two and nine friends for a group message" => {
			Some("status-owned-choose-between-two-and-nine-friends-for-a-group-message")
		}
		"Creating group message…" => Some("status-owned-creating-group-message"),
		"Opening direct message…" => Some("profiles-message-sending"),
		"Finish pending messages and leave the call before closing this DM" => {
			Some("status-owned-finish-pending-messages-and-leave-the-call-before-closing-this-dm")
		}
		"User actions unavailable while disconnected" => {
			Some("status-owned-user-actions-unavailable-while-disconnected")
		}
		"Outcome unknown · check the official client before retrying" => {
			Some("status-owned-outcome-unknown-check-the-official-client-before-retrying")
		}
		"Direct message opened" => Some("status-owned-direct-message-opened"),
		"Group message opened" => Some("status-owned-group-message-opened"),
		"Verify to send this message" => Some("status-owned-verify-to-send-this-message"),
		"Discord requires verification for this message; retry it after the current check" => Some(
			"status-owned-discord-requires-verification-for-this-message-retry-it-after-the-current-check",
		),
		"This device's call session was replaced" => Some("voice-call-moved-to-another-client"),
		"Voice channel exceeds the 64 participant limit" => {
			Some("status-owned-voice-channel-exceeds-the-64-participant-limit")
		}
		"Voice roster exceeds safe capacity; reconnect to refresh" => {
			Some("status-owned-voice-roster-exceeds-safe-capacity-reconnect-to-refresh")
		}
		"Session expired · reconnect explicitly; drafts remain in RAM" => {
			Some("status-owned-session-expired-reconnect-explicitly-drafts-remain-in-ram")
		}
		"Security challenge · unsupported here; connection stopped" => {
			Some("status-owned-security-challenge-unsupported-here-connection-stopped")
		}
		"Permission denied" => Some("status-owned-permission-denied"),
		"Discord requires account attention · connection stopped; check the official client" => {
			Some(
				"status-owned-discord-requires-account-attention-connection-stopped-check-the-official-client",
			)
		}
		"Discord blocked this request · connection stopped; check the official client" => Some(
			"status-owned-discord-blocked-this-request-connection-stopped-check-the-official-client",
		),
		"Rate limited · wait before retrying" => {
			Some("status-owned-rate-limited-wait-before-retrying")
		}
		"Connection failed" => Some("status-owned-connection-failed"),
		"Unsupported service response" => Some("status-owned-unsupported-service-response"),
		"Safe capacity exceeded · connection stopped" => {
			Some("status-owned-safe-capacity-exceeded-connection-stopped")
		}
		"Invalid session input or bot account rejected" => {
			Some("status-owned-invalid-session-input-or-bot-account-rejected")
		}
		"Connecting to voice server..." => Some("status-owned-connecting-to-voice-server"),
		"Checking voice network..." => Some("status-owned-checking-voice-network"),
		"Opening audio devices..." => Some("status-owned-opening-audio-devices"),
		"Connecting call…" => Some("status-owned-connecting-call"),
		"Ringing…" => Some("voice-recipient-ringing"),
		"Securing audio…" => Some("status-owned-securing-audio"),
		"Connected · waiting for others" => Some("status-owned-connected-waiting-for-others"),
		"Keep Message" => Some("message-delete-keep"),
		"Deleting this message is unavailable." => Some("message-delete-unavailable"),
		"Deafened by the server" => Some("voice-notice-deafened-by-the-server"),
		"Muted by the server" => Some("voice-notice-muted-by-the-server"),
		"Speaking is unavailable in this channel. You can still listen." => {
			Some("voice-notice-speaking-is-unavailable-in-this-channel-you-can-still-listen")
		}
		"Push-to-talk is required to speak here. Enable it in Voice settings." => {
			Some("voice-notice-push-to-talk-is-required-to-speak-here-enable-it-in-voice-settings")
		}
		"Synthetic participants · microphone and speakers are off." => {
			Some("voice-notice-synthetic-participants-microphone-and-speakers-are-off")
		}
		"Cached history · awaiting sync" => Some("ui-history-sync"),
		"Conversation unavailable" => Some("ui-history-unavailable"),
		"Opened from archive" => Some("ui-history-archive"),
		"Stream preview" => Some("voice-preview-name"),
		"Preview hidden or unavailable" => Some("voice-preview-unavailable"),
		"Pasted text exceeds the draft limit" => Some("status-draft-paste-limit"),
		"Cancel the current reply before restoring this message" => {
			Some("status-draft-cancel-reply")
		}
		"Keep or clear the existing draft before restoring pending text" => {
			Some("status-draft-restore-pending")
		}
		"Sending is unavailable with the current connection or permissions" => {
			Some("status-send-unavailable")
		}
		"Read status could not be confirmed · unread markers may be out of date" => {
			Some("status-read-confirm")
		}
		"Read status could not sync · unread markers may be out of date" => {
			Some("status-read-sync")
		}
		_ => None,
	}
}
