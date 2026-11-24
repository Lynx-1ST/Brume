//! Desktop ownership for a bounded set of explicitly selected screen shares. The choice is an
//! explicit click; frames are decoded off the render thread and never written to disk.
use crate::gpu_video::Wake;
use client_core::{Command, Event, State, screen, voice};
use discord_voice::{Identity, Status};
use eframe::egui;
use model::Id;
use std::{
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};
use tokio::{runtime::Runtime, sync::watch, task::JoinHandle};
use zeroize::Zeroizing;

const SIGNAL_TIMEOUT: Duration = Duration::from_secs(30);
// Eight UHD YUV pictures plus the displayed picture fit without increasing the item bound.
const DECODED_BUDGET: usize = 128 * 1024 * 1024;

/// Latest stream picture and whether the UI still has to upload it.
type Slot = Option<(platform::video::Picture, bool)>;

/// Replace the latest picture in place: a plain copy of the decoder's planes into one reused
/// allocation. Colour conversion happens on the GPU when the UI uploads it.
fn store_frame(slot: &mut Slot, frame: discord_voice::RemoteFrame<'_>) -> bool {
	let picture = frame.picture;
	if !picture.is_valid() {
		return false;
	}
	let len = picture.data.len();
	let mut data = slot.take().map(|(old, _)| old.data).unwrap_or_default();
	if data.capacity() > 1024 * 1024 && data.capacity() > len.saturating_mul(4) {
		// Release a previous large resolution while keeping ordinary resize reuse.
		data = Vec::new();
	}
	data.clear();
	// Exact growth: never double a previous resolution's allocation beyond the frame.
	data.reserve_exact(len);
	data.extend_from_slice(picture.data);
	*slot = Some((picture.with_data(data), true));
	true
}

// Decode early, display on the audio/device clock. Only decoded pictures may be
// skipped here: the H.264 decoder has already consumed their reference state.
#[derive(Default)]
struct Scheduled {
	latest: Slot,
	pending: std::collections::VecDeque<(platform::video::Picture, discord_voice::Presentation)>,
	bytes: usize,
}
impl Scheduled {
	fn store(&mut self, frame: discord_voice::RemoteFrame<'_>) -> bool {
		let Some(presentation) = frame.presentation.clone() else {
			return store_frame(&mut self.latest, frame);
		};
		if !presentation.is_current() || !frame.picture.is_valid() {
			return false;
		}
		let length = frame.picture.data.len();
		if length > DECODED_BUDGET {
			return false;
		}
		let old = if self.latest.as_ref().is_some_and(|(_, fresh)| !*fresh) {
			self.latest.take()
		} else {
			None
		};
		let mut data = old.map(|(picture, _)| picture.data).unwrap_or_default();
		if data.capacity() > length.saturating_mul(4) {
			data = Vec::new();
		}
		let required = data.capacity().max(length);
		while self.pending.len() >= 8
			|| self.bytes
				+ required + self
				.latest
				.as_ref()
				.map_or(0, |(picture, _)| picture.data.capacity())
				> DECODED_BUDGET
		{
			let Some((old, _)) = self.pending.pop_front() else {
				return false;
			};
			self.bytes -= old.data.capacity();
		}
		data.clear();
		data.reserve_exact(length);
		data.extend_from_slice(frame.picture.data);
		self.bytes += data.capacity();
		self.pending
			.push_back((frame.picture.with_data(data), presentation));
		true
	}
	/// When the ui next has something to show: `None` for now, else the deadline of the
	/// earliest queued picture.
	fn due(&self, now: Instant) -> Option<Instant> {
		if self.latest.as_ref().is_some_and(|(_, fresh)| *fresh) {
			return None;
		}
		let (_, stamp) = self.pending.front()?;
		let deadline = stamp.deadline(now);
		(stamp.is_current() && deadline > now).then_some(deadline)
	}
	fn advance(&mut self, now: Instant) -> Option<Instant> {
		while let Some((_, stamp)) = self.pending.front() {
			if stamp.is_current() && stamp.deadline(now) > now {
				return Some(stamp.deadline(now));
			}
			let (picture, stamp) = self.pending.pop_front().expect("front exists");
			self.bytes -= picture.data.capacity();
			if stamp.is_current() {
				self.latest = Some((picture, true));
			}
		}
		None
	}
}

/// Decoder output: keep the watched user's pictures and wake the UI when one is due.
///
/// Pictures are decoded ahead of their presentation time. Waking the UI on decode as well
/// ran a full UI pass that had nothing new to show, doubling the passes per stream frame.
fn frame_sink(
	slot: Arc<Mutex<Scheduled>>,
	wake: egui::Context,
	streamer: u64,
) -> discord_voice::VideoSink {
	frame_sink_at(slot, wake, streamer, Instant::now)
}

fn frame_sink_at(
	slot: Arc<Mutex<Scheduled>>,
	wake: egui::Context,
	streamer: u64,
	now: impl Fn() -> Instant + Send + Sync + 'static,
) -> discord_voice::VideoSink {
	Arc::new(move |frame: discord_voice::RemoteFrame| {
		if frame.user != streamer && frame.user != 0 {
			return;
		}
		let now = now();
		let due = frame
			.presentation
			.as_ref()
			.map(|presentation| presentation.deadline(now).saturating_duration_since(now));
		if slot.lock().is_ok_and(|mut slot| slot.store(frame)) {
			match due {
				Some(delay) => wake.request_repaint_after(delay),
				None => wake.request_repaint(),
			}
		}
	})
}

/// Moves due pictures forward and uploads the newest one; true when a new picture is shown.
/// Uploading copies the planes into GPU staging, so it runs under the slot lock and the
/// decoder keeps refilling the same allocation.
fn present(
	frames: &Mutex<Scheduled>,
	texture: &mut crate::gpu_video::VideoTexture,
	view: &mut ui::StreamView,
	ctx: &egui::Context,
	wake: &Wake,
) -> bool {
	let Ok(mut slot) = frames.try_lock() else {
		// The decoder is storing a picture; look again shortly instead of waiting a frame.
		wake.picture_after(Duration::from_millis(5));
		return false;
	};
	let now = Instant::now();
	if let Some(deadline) = slot.advance(now) {
		wake.picture_after(deadline.saturating_duration_since(now));
	}
	if let Some((picture, fresh)) = slot.latest.as_mut()
		&& std::mem::take(fresh)
		&& let Some(shown) = texture.upload(
			ctx,
			&format!("remote-stream-{}", view.user),
			picture.borrow(),
		) {
		view.texture = Some(shown);
		return true;
	}
	false
}

/// `--demo-stream-file=REC.au` (a `screen_bench --out` lab recording) replays an encoded
/// stream through the platform decoder, the watch slot and the GPU upload, paced at
/// `--demo-stream-fps` (default 30), and logs decode, upload and UI pass counts. Offline.
#[cfg(feature = "demo")]
struct Replay {
	streamer: Id,
	texture: crate::gpu_video::VideoTexture,
	frames: Arc<Mutex<Scheduled>>,
	stop: Arc<std::sync::atomic::AtomicBool>,
	decoded: Arc<std::sync::atomic::AtomicU64>,
	uploads: u64,
	report: (Instant, u64, u64, u64),
}

#[cfg(feature = "demo")]
impl Drop for Replay {
	fn drop(&mut self) {
		self.stop.store(true, std::sync::atomic::Ordering::Release);
	}
}

#[cfg(feature = "demo")]
impl Watch {
	fn replay(&mut self, streamer: Option<Id>, ui: &mut ui::MessagingUi, ctx: &egui::Context) {
		use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
		let Some(streamer) = streamer else {
			self.replay = None;
			return;
		};
		if self
			.replay
			.as_ref()
			.is_some_and(|replay| replay.streamer != streamer)
		{
			self.replay = None;
		}
		let argument =
			|name: &str| std::env::args().find_map(|arg| arg.strip_prefix(name).map(str::to_owned));
		if self.replay.is_none()
			&& let Some(path) = argument("--demo-stream-file=")
		{
			let fps: u32 = argument("--demo-stream-fps=")
				.and_then(|fps| fps.parse().ok())
				.filter(|fps| (1..=120).contains(fps))
				.unwrap_or(30);
			let frames = Arc::new(Mutex::new(Scheduled::default()));
			let sink = frame_sink(frames.clone(), ctx.clone(), streamer.0);
			let (stop, decoded) = (
				Arc::new(AtomicBool::new(false)),
				Arc::new(AtomicU64::new(0)),
			);
			let (thread_stop, counted) = (stop.clone(), decoded.clone());
			let spawned = std::thread::Builder::new()
				.name("demo-stream".into())
				.spawn(move || {
					// Lab recordings are seconds long; refuse anything that is not.
					let Some(bytes) = std::fs::File::open(&path).ok().and_then(|file| {
						use std::io::Read;
						let limit = 256 << 20;
						let mut bytes = Vec::new();
						file.take(limit + 1).read_to_end(&mut bytes).ok()?;
						(bytes.len() <= limit as usize).then_some(bytes)
					}) else {
						eprintln!("[Serein demo stream] cannot read {path} (at most 256 MiB)");
						return;
					};
					let mut units = Vec::new();
					let mut rest = bytes.as_slice();
					while let Some((length, tail)) = rest.split_first_chunk::<4>() {
						let length = u32::from_le_bytes(*length) as usize;
						if length > 4 << 20 || tail.len() < length {
							break;
						}
						if units.len() >= 36_000 {
							break;
						}
						units.push(&tail[..length]);
						rest = &tail[length..];
					}
					let origin = Instant::now();
					// Pictures are shown a little after decoding, as the network playout does.
					let playout = Duration::from_millis(60);
					let deliver: platform::video::TimedLiveSink = Box::new(move |frame, tag| {
						counted.fetch_add(1, Ordering::Relaxed);
						sink(discord_voice::RemoteFrame {
							presentation: Some(discord_voice::Presentation::new(
								origin + Duration::from_millis(tag) + playout,
							)),
							user: streamer.0,
							picture: frame.borrow(),
						});
					});
					let Ok(mut decoder) = platform::video::live::H264Decoder::new_timed(deliver)
					else {
						eprintln!("[Serein demo stream] no platform H.264 decoder");
						return;
					};
					for (index, unit) in units.iter().cycle().enumerate() {
						if thread_stop.load(Ordering::Acquire) {
							break;
						}
						let at = origin + Duration::from_secs_f64(index as f64 / f64::from(fps));
						std::thread::sleep(at.saturating_duration_since(Instant::now()));
						let _ = decoder.decode_timed(unit, (at - origin).as_millis() as u64);
					}
				});
			if spawned.is_ok() {
				self.replay = Some(Replay {
					streamer,
					texture: Default::default(),
					frames,
					stop,
					decoded,
					uploads: 0,
					report: (Instant::now(), 0, 0, ctx.cumulative_pass_nr()),
				});
			}
		}
		let Some(replay) = &mut self.replay else {
			return;
		};
		let index = match ui
			.voice_stream_views
			.iter()
			.position(|view| view.user == streamer)
		{
			Some(index) => index,
			None if ui.voice_stream_views.len() < voice::MAX_WATCHED_STREAMS => {
				ui.voice_stream_views.push(ui::StreamView {
					user: streamer,
					texture: None,
					status: "Offline replay · recorded lab stream",
					viewers: None,
				});
				ui.voice_stream_views.len() - 1
			}
			None => return,
		};
		let view = &mut ui.voice_stream_views[index];
		if present(&replay.frames, &mut replay.texture, view, ctx) {
			replay.uploads += 1;
			view.status = "Offline replay · recorded lab stream";
		}
		let (since, decoded, uploads, passes) = replay.report;
		if since.elapsed() >= Duration::from_secs(5) {
			let now = (
				replay.decoded.load(Ordering::Relaxed),
				replay.uploads,
				ctx.cumulative_pass_nr(),
			);
			let seconds = since.elapsed().as_secs_f64();
			eprintln!(
				"[Serein demo stream] decoded/s={:.1} uploads/s={:.1} ui_passes/s={:.1}",
				(now.0 - decoded) as f64 / seconds,
				(now.1 - uploads) as f64 / seconds,
				(now.2 - passes) as f64 / seconds,
			);
			replay.report = (Instant::now(), now.0, now.1, now.2);
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Context {
	generation: u64,
	channel: Id,
	request: u64,
	stream_request: u64,
	streamer: Id,
}
struct Pending {
	context: Context,
	user: Id,
	peer: Option<Id>,
	session: Zeroizing<String>,
	identity: Arc<Identity>,
	audio: Option<std::sync::mpsc::SyncSender<discord_voice::PlaybackFrame>>,
	rtc: Option<(Id, Id)>,
	server: Option<(voice::Secret, String)>,
	started: Instant,
}
#[derive(Clone, Copy)]
enum Notice {
	Status(&'static str),
	Failed(&'static str),
}
struct Live {
	context: Context,
	task: JoinHandle<()>,
	events: watch::Receiver<Option<Notice>>,
	frames: Arc<Mutex<Scheduled>>,
	wake: Wake,
}

/// Stores the streamer's pictures and asks for a frame when the next one is due. Only the
/// picture changes then, so the request is for a paint-only frame.
fn sink(frames: Arc<Mutex<Scheduled>>, streamer: u64, wake: Wake) -> discord_voice::VideoSink {
	Arc::new(move |frame: discord_voice::RemoteFrame| {
		if frame.user != streamer {
			return;
		}
		let now = Instant::now();
		let due = frames
			.lock()
			.ok()
			.and_then(|mut slot| slot.store(frame).then(|| slot.due(now)));
		match due {
			Some(Some(deadline)) => wake.picture_after(deadline.saturating_duration_since(now)),
			Some(None) => wake.picture(),
			None => {}
		}
	})
}

/// `--demo-call-stream-live`: the offline stream stage receives generated pictures through
/// the same queue, deadlines and wake-ups as a real stream, so playback can be checked and
/// measured without a call.
#[cfg(feature = "demo")]
fn synthetic_stream() -> bool {
	static LIVE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
	*LIVE.get_or_init(|| std::env::args().any(|arg| arg == "--demo-call-stream-live"))
}
#[cfg(not(feature = "demo"))]
fn synthetic_stream() -> bool {
	false
}

#[derive(Default)]
pub(super) struct Watch {
	#[cfg(feature = "demo")]
	replay: Option<Replay>,
	streams: Vec<Stream>,
	sequence: u64,
	presenter: Option<eframe::PaintOnlyRequester>,
}
impl Watch {
	/// Hands over the renderer's paint-only handle once it exists; `None` keeps full repaints.
	pub fn set_presenter(&mut self, presenter: Option<eframe::PaintOnlyRequester>) {
		if self.presenter.is_none() {
			self.presenter = presenter;
		}
	}
	/// A stream is being received; its pictures are presented by [`Self::paint_only`].
	pub fn live(&self) -> bool {
		self.streams.iter().any(|s| s.live())
	}
	/// Shows the stream's due picture in the texture the last ui pass drew, without a ui
	/// pass. `false` when the ui has to run instead: a first picture or another size.
	pub fn paint_only(&mut self, ctx: &egui::Context) -> bool {
		let mut ok = true;
		for stream in &mut self.streams {
			ok &= stream.paint_only(ctx);
		}
		ok
	}
	pub fn stop(&mut self) {
		#[cfg(feature = "demo")]
		{
			self.replay = None;
		}
		self.streams.clear();
	}
	pub fn observe(&mut self, state: &State, event: &mut Event) {
		for stream in &mut self.streams {
			stream.observe(state, event);
		}
	}
	pub fn poll(
		&mut self,
		runtime: &Runtime,
		state: &mut State,
		ui: &mut ui::MessagingUi,
		ctx: &egui::Context,
		call: Option<super::screen::Call<'_>>,
		audio: Option<std::sync::mpsc::SyncSender<discord_voice::PlaybackFrame>>,
	) -> Option<Command> {
		let selected = state
			.voice
			.active
			.as_ref()
			.map(|call| call.watching.as_slice())
			.unwrap_or_default();
		if state.demo {
			self.streams.clear();
			ui.voice_stream_views
				.retain(|view| selected.contains(&view.user));
			#[cfg(feature = "demo")]
			self.replay(selected.first().copied(), ui, ctx);
			return None;
		}
		self.streams
			.retain(|stream| stream.context().is_some() || selected.contains(&stream.view.user));
		for user in selected.iter().take(voice::MAX_WATCHED_STREAMS) {
			if !self.streams.iter().any(|stream| stream.view.user == *user)
				&& self.streams.len() < voice::MAX_WATCHED_STREAMS
			{
				self.streams.push(Stream::new(*user));
			}
		}
		// The existing desktop dispatcher accepts one command per poll. Other streams
		// remain pending and request another frame instead of losing their commands.
		for stream in &mut self.streams {
			stream.sequence = self.sequence;
			let command = stream.poll(
				runtime,
				state,
				ui,
				ctx,
				call.clone(),
				audio.clone(),
				self.presenter.clone(),
			);
			self.sequence = stream.sequence;
			if command.is_some() {
				ui.voice_stream_views.clear();
				ui.voice_stream_views
					.extend(self.streams.iter().map(|stream| stream.view.clone()));
				ctx.request_repaint();
				return command;
			}
		}
		self.streams.retain(|stream| {
			stream.context().is_some()
				|| state
					.voice
					.active
					.as_ref()
					.is_some_and(|call| call.watching.contains(&stream.view.user))
		});
		ui.voice_stream_views.clear();
		ui.voice_stream_views
			.extend(self.streams.iter().map(|stream| stream.view.clone()));
		None
	}
}

struct Stream {
	view: ui::StreamView,
	pending: Option<Pending>,
	live: Option<Live>,
	/// The streamer stopped or Discord failed the view; the state choice is cleared next poll.
	ended: Option<(ui::design::Level, &'static str)>,
	sequence: u64,
	/// GPU texture belonging to this stream alone.
	texture: crate::gpu_video::VideoTexture,
	/// Discord's latest viewer roster for this view, published with its `ui::StreamView`.
	viewers: Option<(Context, Arc<screen::Viewers>)>,
}
impl Stream {
	fn new(streamer: Id) -> Self {
		Self {
			view: ui::StreamView {
				user: streamer,
				texture: None,
				status: "Connecting to the stream…",
				viewers: None,
			},
			pending: None,
			live: None,
			ended: None,
			sequence: 0,
			texture: Default::default(),
			viewers: None,
		}
	}
	pub fn live(&self) -> bool {
		self.live.is_some()
	}
	pub fn paint_only(&mut self, ctx: &egui::Context) -> bool {
		let Some(live) = &self.live else {
			return true;
		};
		// The decoder holds the lock only while it copies a picture in; its own wake follows.
		let Ok(mut slot) = live.frames.try_lock() else {
			return true;
		};
		let now = Instant::now();
		if let Some(deadline) = slot.advance(now) {
			live.wake
				.picture_after(deadline.saturating_duration_since(now));
		}
		match slot.latest.as_mut() {
			Some((picture, fresh)) if *fresh => {
				if !self.texture.fits_picture(&picture.borrow()) {
					return false;
				}
				*fresh = false;
				if let Some(texture) = self.texture.upload(
					ctx,
					&format!("remote-stream-{}", self.view.user),
					picture.borrow(),
				) {
					self.view.texture = Some(texture);
					true
				} else {
					false
				}
			}
			_ => true,
		}
	}
	pub fn stop(&mut self) {
		self.pending = None;
		self.ended = None;
		self.viewers = None;
		self.view.viewers = None;
		self.view.texture = None;
		if let Some(live) = self.live.take() {
			live.task.abort();
		}
		// Logout stops polling, so release the GPU picture here; disposal waits a frame.
		self.texture = Default::default();
	}
	fn context(&self) -> Option<Context> {
		self.pending
			.as_ref()
			.map(|pending| pending.context)
			.or_else(|| self.live.as_ref().map(|live| live.context))
	}
	/// Take negotiation secrets before the UI reduces the event. Nothing is persisted.
	pub fn observe(&mut self, state: &State, event: &mut Event) {
		let Event::Voice(voice::Event::Watch {
			channel,
			request,
			stream_request,
			streamer,
			event,
		}) = event
		else {
			return;
		};
		let Some(context) = self.context() else {
			return;
		};
		if context.generation != state.generation
			|| (
				context.channel,
				context.request,
				context.stream_request,
				context.streamer,
			) != (*channel, *request, *stream_request, *streamer)
		{
			return;
		}
		match event {
			screen::Event::Created {
				rtc_server,
				rtc_channel,
			} => {
				if let Some(pending) = &mut self.pending {
					pending.rtc = Some((*rtc_server, *rtc_channel));
				}
			}
			screen::Event::Server { token, endpoint } => {
				if let Some(pending) = &mut self.pending {
					match (token.take(), endpoint.take()) {
						(Some(token), Some(endpoint)) => pending.server = Some((token, endpoint)),
						// A null endpoint means Discord is still allocating the stream server.
						(_, None) => pending.server = None,
						(None, Some(_)) => {
							self.ended = Some((
								ui::design::Level::Warning,
								"Discord omitted the stream connection token",
							));
						}
					}
				}
			}
			screen::Event::Deleted { reason } => {
				self.ended = Some((
					ui::design::Level::Info,
					reason.unwrap_or("The stream ended"),
				));
			}
			screen::Event::Failed(message) => {
				self.ended = Some((ui::design::Level::Warning, message));
			}
			screen::Event::Viewers(viewers) => {
				self.viewers = Some((context, Arc::new(std::mem::take(viewers))));
			}
		}
	}
	pub fn poll(
		&mut self,
		runtime: &Runtime,
		state: &mut State,
		ui: &mut ui::MessagingUi,
		ctx: &egui::Context,
		call: Option<super::screen::Call<'_>>,
		audio: Option<std::sync::mpsc::SyncSender<discord_voice::PlaybackFrame>>,
	) -> Option<Command> {
		let streamer = self.view.user;
		let wanted = state.voice.active.as_ref().and_then(|active| {
			if !active.watching.contains(&streamer) {
				return None;
			}
			// The separate stream transport survives call rekeys and voice resumption.
			(active.phase != voice::Phase::Failed).then_some((
				state.generation,
				active.channel,
				active.request,
				streamer,
			))
		});
		let current = self.context();
		let mut command = None;
		let synthetic = state.demo && synthetic_stream();
		if let Some(context) = current
			&& ((state.demo && !synthetic)
				|| wanted
					!= Some((
						context.generation,
						context.channel,
						context.request,
						context.streamer,
					))) {
			self.stop();
			self.view.status = "Stopped watching";
			if context.generation == state.generation && !state.demo {
				command = Some(Command::Voice(voice::Command::StopWatching {
					channel: context.channel,
					request: context.request,
					stream_request: context.stream_request,
				}));
			}
		}
		if let Some((level, message)) = self.ended.take() {
			let context = self.context();
			self.stop();
			self.view.status = message;
			// Said once; the stage keeps no line for a view that already ended.
			ui.toasts.push(level, message);
			state.stop_watching_stream(streamer);
			self.view.texture = None;
			self.texture = Default::default();
			return command.or_else(|| {
				let context = context.filter(|context| context.generation == state.generation)?;
				Some(Command::Voice(voice::Command::StopWatching {
					channel: context.channel,
					request: context.request,
					stream_request: context.stream_request,
				}))
			});
		}
		if state.demo && !synthetic {
			self.view.status = if wanted.is_some() {
				"Offline preview · no stream is received"
			} else {
				""
			};
			return command;
		}
		#[cfg(feature = "demo")]
		if synthetic
			&& let Some((generation, channel, request, streamer)) = wanted
			&& self.live.is_none()
		{
			self.sequence = self.sequence.wrapping_add(1);
			let context = Context {
				generation,
				channel,
				request,
				stream_request: self.sequence,
				streamer,
			};
			self.start_synthetic(runtime, context, ctx);
		}
		if !synthetic
			&& let Some((generation, channel, request, streamer)) = wanted
			&& self.context().is_none()
			&& command.is_none()
		{
			let Some(call) = call.filter(|call| {
				call.generation == generation && call.channel == channel && call.request == request
			}) else {
				self.view.status = "Connect the call before watching a stream";
				return None;
			};
			self.sequence = self.sequence.wrapping_add(1);
			let context = Context {
				generation,
				channel,
				request,
				stream_request: self.sequence,
				streamer,
			};
			self.pending = Some(Pending {
				context,
				user: call.user,
				peer: call.peer,
				session: Zeroizing::new(call.session.to_owned()),
				identity: call.identity,
				audio,
				rtc: None,
				server: None,
				started: Instant::now(),
			});
			self.view.status = "Requesting the stream…";
			command = Some(Command::Voice(voice::Command::WatchStream {
				channel,
				request,
				stream_request: context.stream_request,
				streamer,
			}));
		}
		if let Some(pending) = &self.pending {
			if pending.started.elapsed() >= SIGNAL_TIMEOUT {
				self.ended = Some((
					ui::design::Level::Warning,
					"Discord did not provide the stream connection; try again",
				));
				ctx.request_repaint();
			} else {
				ctx.request_repaint_after(SIGNAL_TIMEOUT.saturating_sub(pending.started.elapsed()));
			}
		}
		if self
			.pending
			.as_ref()
			.is_some_and(|pending| pending.rtc.is_some() && pending.server.is_some())
		{
			let pending = self.pending.take().expect("complete stream negotiation");
			if let Err(error) = self.start(runtime, pending, ctx, presenter) {
				self.ended = Some((ui::design::Level::Warning, error));
			}
		}
		if let Some(live) = &mut self.live {
			match *live.events.borrow_and_update() {
				Some(Notice::Status(status)) => self.view.status = status,
				Some(Notice::Failed(error)) => {
					self.ended = Some((ui::design::Level::Warning, error));
				}
				None => {}
			}
			if live.task.is_finished() && self.ended.is_none() {
				self.ended = Some((ui::design::Level::Warning, "The stream connection ended"));
			}
			if present(
				&live.frames,
				&mut self.texture,
				&mut self.view,
				ctx,
				&live.wake,
			) {
				self.view.status = "Watching the stream";
			}
		} else {
			self.view.texture = None;
			self.texture = Default::default();
		}
		let current = self.context();
		self.view.viewers = self
			.viewers
			.as_ref()
			.filter(|(context, _)| Some(*context) == current)
			.map(|(_, viewers)| Arc::clone(viewers));
		self.view.status = if wanted.is_some() {
			self.view.status
		} else {
			""
		};
		command
	}
	fn start(
		&mut self,
		runtime: &Runtime,
		mut pending: Pending,
		ctx: &egui::Context,
		presenter: Option<eframe::PaintOnlyRequester>,
	) -> Result<(), &'static str> {
		let (rtc_server, rtc_channel) = pending.rtc.take().ok_or("Missing stream RTC identity")?;
		let (token, endpoint) = pending.server.take().ok_or("Missing stream server")?;
		let session = voice::Secret::new(pending.session.to_string())
			.map_err(|_| "Invalid stream voice session")?;
		let credentials = voice::VoiceConnection {
			channel: rtc_channel,
			guild: Some(rtc_server),
			user: pending.user,
			peer: pending.peer,
			session,
			token,
			endpoint,
			request: pending.context.stream_request,
		};
		let frames = Arc::new(Mutex::new(Scheduled::default()));
		let streamer = pending.context.streamer;
		let pictures = Wake::new(ctx, presenter);
		let sink = sink(frames.clone(), streamer.0, pictures.clone());
		let (send, events) = watch::channel(None);
		let wake = ctx.clone();
		let identity = pending.identity;
		let audio = pending.audio;
		let task = runtime.spawn(async move {
			let (status_send, status_wake) = (send.clone(), wake.clone());
			let result = discord_voice::watch_stream(
				credentials,
				identity,
				Some(streamer),
				sink,
				audio,
				move |event| {
					let status = match event {
						Status::Connecting => "Connecting to the stream…",
						Status::Discovering => "Checking the stream network…",
						Status::TransportReady | Status::Securing => "Securing the stream…",
						Status::WaitingForPeer => "Waiting for the streamer…",
						Status::Ready { .. } => "Stream secured · waiting for video",
						Status::RemoteAudio
						| Status::Speaking(_)
						| Status::CameraAvailable(_)
						| Status::Latency(_) => {
							return Ok(());
						}
					};
					status_send.send_replace(Some(Notice::Status(status)));
					status_wake.request_repaint();
					Ok(())
				},
			)
			.await;
			if let Err(error) = result {
				send.send_replace(Some(Notice::Failed(error)));
			}
			wake.request_repaint();
		});
		self.view.status = "Connecting to the stream…";
		self.live = Some(Live {
			context: pending.context,
			task,
			events,
			frames,
			wake: pictures,
		});
		Ok(())
	}
	/// Generated 1080p60 pictures, stamped 80 ms ahead like a jitter-buffered stream.
	#[cfg(feature = "demo")]
	fn start_synthetic(
		&mut self,
		runtime: &Runtime,
		context: Context,
		ctx: &egui::Context,
		presenter: Option<eframe::PaintOnlyRequester>,
	) {
		use platform::video::{Color, Layout, Picture};
		const SIZE: (u32, u32) = (1920, 1080);
		let frames = Arc::new(Mutex::new(Scheduled::default()));
		let wake = Wake::new(ctx, presenter);
		let sink = sink(frames.clone(), context.streamer.0, wake.clone());
		let user = context.streamer.0;
		let task = runtime.spawn(async move {
			let (width, height) = (SIZE.0 as usize, SIZE.1 as usize);
			let mut data = vec![128_u8; width * height * 3 / 2];
			let mut ticks = tokio::time::interval(Duration::from_micros(16_667));
			let mut frame = 0_usize;
			loop {
				ticks.tick().await;
				frame = frame.wrapping_add(1);
				// A diagonal ramp that moves a little every picture.
				for (y, row) in data[..width * height].chunks_exact_mut(width).enumerate() {
					let start = (y / 4).wrapping_add(frame.wrapping_mul(3)) as u8;
					for (x, luma) in row.iter_mut().enumerate() {
						*luma = start.wrapping_add((x / 8) as u8);
					}
				}
				sink(discord_voice::RemoteFrame {
					presentation: Some(discord_voice::Presentation::new(
						Instant::now() + Duration::from_millis(80),
					)),
					user,
					picture: Picture {
						width: SIZE.0,
						height: SIZE.1,
						layout: Layout::Nv12,
						color: Color::WEBRTC,
						rotation: 0,
						data: data.as_slice(),
					},
				});
			}
		});
		self.view.status = "Watching the stream";
		self.live = Some(Live {
			context,
			task,
			events: watch::channel(None).1,
			frames,
			wake,
		});
	}
}
impl Drop for Stream {
	fn drop(&mut self) {
		self.stop();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn demo_poll_preserves_selected_synthetic_texture_until_stop() {
		let runtime = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap();
		let mut state = test_support::voice_demo_state();
		state.demo = true;
		state.watch_stream(Id(3)).unwrap();
		let texture = egui::load::SizedTexture {
			id: egui::TextureId::User(19),
			size: egui::vec2(640.0, 360.0),
		};
		let mut ui = ui::MessagingUi::default();
		ui.voice_stream_views = vec![ui::StreamView {
			user: Id(3),
			texture: Some(texture),
			status: "Synthetic stream",
			viewers: None,
		}];
		let mut owner = Watch::default();
		let ctx = egui::Context::default();
		assert!(
			owner
				.poll(&runtime, &mut state, &mut ui, &ctx, None, None)
				.is_none()
		);
		assert_eq!(ui.voice_stream_views[0].texture.unwrap().id, texture.id);
		assert!(owner.streams.is_empty());
		state.stop_watching();
		assert!(
			owner
				.poll(&runtime, &mut state, &mut ui, &ctx, None, None)
				.is_none()
		);
		assert!(ui.voice_stream_views.is_empty());
	}

	#[test]
	fn one_stream_failure_and_late_events_leave_another_stream_running() {
		let runtime = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap();
		let mut state = test_support::voice_demo_state();
		state.demo = false;
		let call = state.voice.active.as_mut().unwrap();
		call.watching = vec![Id(3), Id(7)];
		let channel = call.channel;
		let request = call.request;
		let context = |streamer, stream_request| Context {
			generation: state.generation,
			channel,
			request,
			streamer,
			stream_request,
		};
		let mut owner = Watch::default();
		for (user, token) in [(Id(3), 10), (Id(7), 11)] {
			let mut stream = Stream::new(user);
			stream.viewers = Some((
				context(user, token),
				Arc::new(screen::Viewers {
					count: 1,
					ids: vec![Id(token)],
				}),
			));
			stream.live = Some(Live {
				context: context(user, token),
				task: runtime.spawn(std::future::pending()),
				events: watch::channel(None).1,
				frames: Arc::new(Mutex::new(Scheduled::default())),
			});
			owner.streams.push(stream);
		}
		let mut event = Event::Voice(voice::Event::Watch {
			channel,
			request,
			streamer: Id(3),
			stream_request: 9,
			event: screen::Event::Failed("late synthetic failure"),
		});
		owner.observe(&state, &mut event);
		assert!(owner.streams.iter().all(|stream| stream.ended.is_none()));
		let mut event = Event::Voice(voice::Event::Watch {
			channel,
			request,
			streamer: Id(3),
			stream_request: 10,
			event: screen::Event::Failed("synthetic failure"),
		});
		owner.observe(&state, &mut event);
		let mut ui = ui::MessagingUi::default();
		let ctx = egui::Context::default();
		assert!(matches!(
			owner.poll(&runtime, &mut state, &mut ui, &ctx, None, None),
			Some(Command::Voice(voice::Command::StopWatching {
				stream_request: 10,
				..
			}))
		));
		assert_eq!(state.voice.active.as_ref().unwrap().watching, vec![Id(7)]);
		assert!(
			owner
				.poll(&runtime, &mut state, &mut ui, &ctx, None, None)
				.is_none()
		);
		assert_eq!(owner.streams.len(), 1);
		assert!(owner.streams[0].live.is_some());
		assert_eq!(ui.voice_stream_views.len(), 1);
		assert_eq!(ui.voice_stream_views[0].user, Id(7));
		assert_eq!(
			ui.voice_stream_views[0].viewers.as_ref().unwrap().ids,
			vec![Id(11)]
		);
		owner.stop();
		assert!(owner.streams.is_empty());
	}

	#[test]
	fn watch_survives_call_recovery_until_its_selection_ends() {
		let runtime = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap();
		let mut state = test_support::voice_demo_state();
		state.demo = false;
		state.watch_stream(Id(3)).unwrap();
		let call = state.voice.active.as_ref().unwrap();
		let context = Context {
			generation: state.generation,
			channel: call.channel,
			request: call.request,
			stream_request: 2,
			streamer: Id(3),
		};
		let mut ui = ui::MessagingUi::default();
		let ctx = egui::Context::default();
		for failed in [false, true] {
			state.voice.active.as_mut().unwrap().watching = vec![context.streamer];
			let (_send, events) = watch::channel(None);
			let mut watch = Stream {
				view: ui::StreamView {
					user: context.streamer,
					texture: None,
					status: "",
					viewers: None,
				},
				pending: None,
				live: Some(Live {
					context,
					task: runtime.spawn(std::future::pending()),
					events,
					frames: Arc::new(Mutex::new(Scheduled::default())),
					wake: Wake::new(&ctx, None),
				}),
				ended: None,
				sequence: 0,
				texture: Default::default(),
				viewers: None,
			};
			// Only the live view's roster reaches the stage; another attempt's is ignored.
			let roster = |stream_request, ids: Vec<Id>| {
				Event::Voice(voice::Event::Watch {
					channel: context.channel,
					request: context.request,
					stream_request,
					streamer: context.streamer,
					event: screen::Event::Viewers(screen::Viewers {
						count: ids.len() as u16,
						ids,
					}),
				})
			};
			watch.observe(&state, &mut roster(2, vec![Id(1), Id(4)]));
			watch.observe(&state, &mut roster(9, vec![Id(5)]));
			for phase in [
				voice::Phase::Connected,
				voice::Phase::Securing,
				voice::Phase::ConnectingTransport,
				voice::Phase::OpeningAudio,
				voice::Phase::Waiting,
				voice::Phase::Connected,
			] {
				state.voice.active.as_mut().unwrap().phase = phase;
				assert!(
					watch
						.poll(&runtime, &mut state, &mut ui, &ctx, None, None)
						.is_none()
				);
				assert!(watch.live.is_some());
				assert_eq!(
					watch.view.viewers.as_ref().map(|v| v.ids.as_slice()),
					Some([Id(1), Id(4)].as_slice())
				);
			}
			if failed {
				state.voice.active.as_mut().unwrap().phase = voice::Phase::Failed;
			} else {
				state.stop_watching();
			}
			assert!(matches!(
				watch.poll(&runtime, &mut state, &mut ui, &ctx, None, None),
				Some(Command::Voice(voice::Command::StopWatching {
					channel, request, stream_request: 2,
				})) if channel == context.channel && request == context.request
			));
			assert!(watch.live.is_none());
			assert!(
				watch.view.viewers.is_none(),
				"an ended view keeps no viewers"
			);
		}
	}

	fn frame(width: u32, height: u32, data: &[u8]) -> discord_voice::RemoteFrame<'_> {
		discord_voice::RemoteFrame {
			presentation: None,
			user: 7,
			picture: platform::video::Picture {
				width,
				height,
				layout: platform::video::Layout::I420,
				color: platform::video::Color::WEBRTC,
				rotation: 0,
				data,
			},
		}
	}

	fn i420(width: u32, height: u32, value: u8) -> Vec<u8> {
		vec![value; platform::video::Layout::I420.len(width, height).unwrap()]
	}

	/// Decoding runs ahead of presentation; waking the UI on decode too ran a full UI pass
	/// with nothing new to show, doubling the passes per stream frame.
	#[test]
	fn decoded_pictures_wake_the_ui_when_due_not_when_decoded() {
		let ctx = egui::Context::default();
		let delays = Arc::new(Mutex::new(Vec::new()));
		let seen = delays.clone();
		ctx.set_request_repaint_callback(move |info| seen.lock().unwrap().push(info.delay));
		let now = Instant::now();
		let sink = frame_sink_at(Arc::default(), ctx.clone(), 7, move || now);
		let data = i420(4, 4, 9);
		let mut scheduled = frame(4, 4, &data);
		scheduled.presentation = Some(discord_voice::Presentation::new(
			now + Duration::from_millis(80),
		));
		sink(scheduled);
		let mut other = frame(4, 4, &data);
		other.user = 8;
		sink(other);
		let requested = std::mem::take(&mut *delays.lock().unwrap());
		assert_eq!(requested.len(), 1, "{requested:?}");
		assert!(requested[0] > Duration::ZERO, "{requested:?}");
		// Pictures without a presentation time are shown as soon as they arrive.
		sink(frame(4, 4, &data));
		assert_eq!(*delays.lock().unwrap(), [Duration::ZERO]);
	}

	#[test]
	fn watch_reuses_the_frame_buffer_and_flags_only_updates() {
		let mut slot = None;
		assert!(store_frame(&mut slot, frame(2, 2, &i420(2, 2, 1))));
		let planes = slot.as_ref().unwrap().0.data.as_ptr();
		for value in 0..20 {
			slot.as_mut().unwrap().1 = false;
			assert!(store_frame(&mut slot, frame(2, 2, &i420(2, 2, value))));
			let (picture, fresh) = slot.as_ref().unwrap();
			assert!(*fresh);
			assert_eq!(picture.data.as_ptr(), planes);
			assert_eq!(picture.data, i420(2, 2, value));
		}
		// A smaller picture reuses the allocation too.
		assert!(store_frame(&mut slot, frame(1, 1, &i420(1, 1, 9))));
		let (picture, _) = slot.as_ref().unwrap();
		assert_eq!(picture.data.as_ptr(), planes);
		assert_eq!((picture.width, picture.height), (1, 1));
	}

	#[test]
	fn watch_invalid_frames_preserve_the_last_valid_picture() {
		let mut slot = None;
		assert!(store_frame(&mut slot, frame(1, 1, &i420(1, 1, 4))));
		let planes = slot.as_ref().unwrap().0.data.as_ptr();
		for invalid in [
			frame(0, 1, &[]),
			frame(1, 0, &[]),
			frame(1, 1, &[0; 2]),
			frame(u32::MAX, u32::MAX, &[]),
		] {
			assert!(!store_frame(&mut slot, invalid));
			let (picture, _) = slot.as_ref().unwrap();
			assert_eq!(picture.data.as_ptr(), planes);
			assert_eq!((picture.width, picture.height), (1, 1));
		}
	}

	#[test]
	fn watch_releases_large_capacity_after_a_resolution_drop() {
		let mut slot = None;
		assert!(store_frame(
			&mut slot,
			frame(1920, 1080, &i420(1920, 1080, 0))
		));
		assert!(store_frame(&mut slot, frame(16, 16, &i420(16, 16, 0))));
		let (picture, _) = slot.take().unwrap();
		assert_eq!(picture.data.len(), 16 * 16 * 3 / 2);
		assert!(picture.data.capacity() <= 16 * 16 * 6);
	}

	#[test]
	fn watch_resolution_growth_does_not_double_the_capacity() {
		let mut slot = None;
		assert!(store_frame(
			&mut slot,
			frame(1600, 900, &i420(1600, 900, 0))
		));
		assert!(store_frame(
			&mut slot,
			frame(1920, 1080, &i420(1920, 1080, 0))
		));
		let (picture, _) = slot.take().unwrap();
		assert_eq!(picture.data.len(), 1920 * 1080 * 3 / 2);
		assert!(picture.data.capacity() <= 1920 * 1080 * 3 / 2);
	}
	#[test]
	fn watch_accepts_1440p_frame() {
		let mut slot = None;
		assert!(store_frame(
			&mut slot,
			frame(2560, 1440, &i420(2560, 1440, 0))
		));
		let (picture, _) = slot.take().unwrap();
		assert_eq!(picture.data.len(), 2560 * 1440 * 3 / 2);
		assert_eq!((picture.width, picture.height), (2560, 1440));
	}

	#[test]
	fn decoded_stream_pictures_wait_for_presentation_and_pressure_skips_only_decoded_frames() {
		let now = Instant::now();
		let mut queue = Scheduled::default();
		let pixels = i420(3840, 2160, 4);
		for n in 0..16 {
			let mut picture = frame(3840, 2160, &pixels);
			picture.presentation = Some(discord_voice::Presentation::new(
				now + Duration::from_millis(100 + n * 16),
			));
			assert!(queue.store(picture));
		}
		assert_eq!(queue.pending.len(), 8);
		assert!(queue.bytes <= DECODED_BUDGET);
		assert!(queue.latest.is_none());
		assert_eq!(queue.advance(now), Some(now + Duration::from_millis(228)));
		queue.advance(now + Duration::from_millis(260));
		assert_eq!(queue.pending.len(), 5);
		assert!(queue.latest.as_ref().unwrap().1);
		assert!(queue.bytes + queue.latest.as_ref().unwrap().0.data.capacity() <= DECODED_BUDGET);
		drop(queue);
		// A backend's RGBA fallback reaches the byte bound before the eight-picture bound.
		let mut queue = Scheduled::default();
		let pixels = vec![4; 3840 * 2160 * 4];
		for n in 0..16 {
			let mut picture = frame(3840, 2160, &pixels);
			picture.picture.layout = platform::video::Layout::Rgba;
			picture.presentation = Some(discord_voice::Presentation::new(
				now + Duration::from_millis(100 + n * 16),
			));
			assert!(queue.store(picture));
		}
		assert_eq!(queue.pending.len(), 4);
		assert!(queue.bytes <= DECODED_BUDGET);
		queue.advance(now + Duration::from_secs(1));
		assert!(queue.pending.is_empty());
		assert_eq!(queue.bytes, 0);
		assert!(queue.latest.as_ref().unwrap().1);
	}
}
