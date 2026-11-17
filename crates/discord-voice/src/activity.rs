//! Display-only activity; never gates or changes transmitted audio.
// ponytail: a -45 dBFS level threshold detects sound, not speech; use VAD if noise lights it up.
pub(crate) fn hold(energy: f32, previous: u8) -> u8 {
	if energy.is_finite() && energy > 960.0 * 0.000_031_623 {
		10 // 200 ms at the transport's 20 ms cadence.
	} else {
		previous.saturating_sub(1)
	}
}

/// Finite RMS level for the local preview meter, clamped to its display range.
pub(crate) fn level_db(frame: &[f32; 960]) -> f32 {
	let energy: f32 = frame
		.iter()
		.filter(|s| s.is_finite())
		.map(|s| s.clamp(-1.0, 1.0).powi(2))
		.sum();
	(10.0 * (energy / 960.0).max(1e-10).log10()).clamp(-100.0, 0.0)
}

/// Manual sensitivity with 3 dB hysteresis, 200 ms release and a 5 ms click-free ramp.
pub(crate) struct InputGate {
	remaining: u8,
	gain: f32,
	noise_floor: f32,
	window_minimum: f32,
	window_frames: u8,
}
impl Default for InputGate {
	fn default() -> Self {
		Self {
			remaining: 0,
			gain: 0.0,
			noise_floor: -65.0,
			window_minimum: f32::INFINITY,
			window_frames: 0,
		}
	}
}
impl InputGate {
	pub fn apply(&mut self, frame: &mut [f32; 960], threshold: Option<i16>) -> bool {
		self.apply_detected(frame, threshold.map(f32::from), true)
	}

	pub fn apply_settings(
		&mut self,
		frame: &mut [f32; 960],
		settings: model::voice_settings::Processing,
		speech_probability: f32,
	) -> bool {
		if !settings.automatic_sensitivity && !settings.advanced_voice_activity {
			return self.apply(frame, settings.sensitivity_db);
		}
		let level = level_db(frame);
		let speech = !settings.advanced_voice_activity
			|| (speech_probability.is_finite()
				&& speech_probability >= if self.remaining > 0 { 0.35 } else { 0.6 });
		let threshold = if settings.automatic_sensitivity {
			// A two-second minimum follows background changes without learning individual
			// syllables as noise. With VAD, exclude speech from the estimator altogether.
			let has_detector = settings.advanced_voice_activity
				|| settings.suppression == model::voice_settings::NoiseSuppression::RnNoise;
			let recognized_speech = speech_probability.is_finite() && speech_probability >= 0.35;
			if !has_detector || !recognized_speech {
				self.window_minimum = self.window_minimum.min(level);
			}
			self.window_frames += 1;
			if self.window_frames == 100 {
				if self.window_minimum.is_finite() {
					self.noise_floor = self.window_minimum.clamp(-80.0, -32.0);
				}
				self.window_frames = 0;
				self.window_minimum = f32::INFINITY;
			}
			Some((self.noise_floor + 12.0).clamp(-65.0, -20.0))
		} else {
			settings.sensitivity_db.map(f32::from)
		};
		// VAD still provides a gate when the manual level gate is disabled.
		self.apply_detected(frame, threshold.or(Some(-80.0)), speech)
	}

	fn apply_detected(
		&mut self,
		frame: &mut [f32; 960],
		threshold: Option<f32>,
		detected: bool,
	) -> bool {
		let Some(threshold) = threshold else {
			self.gain = 1.0;
			self.remaining = 0;
			return true;
		};
		let closing = self.remaining > 0;
		let threshold = threshold.clamp(-80.0, 0.0) - if closing { 3.0 } else { 0.0 };
		if detected && level_db(frame) > threshold {
			self.remaining = 10;
		} else {
			self.remaining = self.remaining.saturating_sub(1);
		}
		let open = self.remaining > 0;
		let audible = open || self.gain > 0.0;
		for sample in frame {
			self.gain = (self.gain + if open { 1.0 / 240.0 } else { -1.0 / 240.0 }).clamp(0.0, 1.0);
			*sample *= self.gain;
		}
		audible
	}
}

pub(crate) fn hold_at(energy: f32, previous: u8, threshold: i16) -> u8 {
	if energy.is_finite()
		&& energy > 960.0 * 10.0_f32.powf(f32::from(threshold.clamp(-80, 0)) / 10.0)
	{
		10
	} else {
		previous.saturating_sub(1)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use model::voice_settings::Processing;

	#[test]
	fn automatic_gate_learns_background_without_losing_manual_setting_or_speech() {
		let settings = Processing {
			automatic_sensitivity: true,
			sensitivity_db: Some(-25),
			..Processing::studio()
		};
		let mut gate = InputGate::default();
		// A steady -40 dBFS background becomes the floor after a calibration window.
		for _ in 0..112 {
			gate.apply_settings(&mut [0.01; 960], settings, 0.0);
		}
		assert!(!gate.apply_settings(&mut [0.01; 960], settings, 0.0));
		assert!(gate.apply_settings(&mut [0.1; 960], settings, 1.0));
		// Disabling auto restores -25 dBFS rather than the learned threshold.
		let manual = Processing {
			automatic_sensitivity: false,
			..settings
		};
		for _ in 0..12 {
			gate.apply_settings(&mut [0.01; 960], manual, 0.0);
		}
		assert!(!gate.apply_settings(&mut [0.01; 960], manual, 1.0));
		let detected = Processing {
			suppression: model::voice_settings::NoiseSuppression::RnNoise,
			..settings
		};
		// Several windows of recognized speech must not raise the learned noise floor.
		for _ in 0..220 {
			assert!(gate.apply_settings(&mut [0.05; 960], detected, 0.9));
		}
	}

	#[test]
	fn speech_gate_rejects_loud_noise_and_nonfinite_scores_then_holds_word_endings() {
		let settings = Processing {
			advanced_voice_activity: true,
			..Processing::studio()
		};
		let mut gate = InputGate::default();
		assert!(!gate.apply_settings(&mut [0.2; 960], settings, 0.1));
		assert!(!gate.apply_settings(&mut [0.2; 960], settings, f32::NAN));
		assert!(gate.apply_settings(&mut [0.1; 960], settings, 0.8));
		assert!(gate.apply_settings(&mut [0.001; 960], settings, 0.4));
		for _ in 0..12 {
			gate.apply_settings(&mut [0.2; 960], settings, 0.0);
		}
		assert!(!gate.apply_settings(&mut [0.2; 960], settings, 0.0));
		assert!(gate.apply_settings(&mut [0.2; 960], Processing::studio(), 0.0));
	}
}
