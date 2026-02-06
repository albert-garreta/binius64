// Copyright 2026 The Binius Developers
//! Phase timing layer for capturing proof generation phase timings via tracing

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::{Duration, Instant},
};

use tracing::{
	Subscriber,
	span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

/// Collected phase timings from a proof generation run
#[derive(Debug, Clone, Default)]
pub struct PhaseTimings {
	pub witness_commit: Option<Duration>,
	pub intmul_reduction: Option<Duration>,
	pub bitand_reduction: Option<Duration>,
	pub shift_reduction: Option<Duration>,
	pub pcs_opening: Option<Duration>,
	pub total_prove: Option<Duration>,
}

impl PhaseTimings {
	/// Returns witness commit time as a fraction of total prove time
	pub fn witness_commit_fraction(&self) -> Option<f64> {
		match (self.witness_commit, self.total_prove) {
			(Some(wc), Some(total)) => Some(wc.as_secs_f64() / total.as_secs_f64()),
			_ => None,
		}
	}

	/// Returns the time for everything except witness commit
	pub fn non_witness_commit_time(&self) -> Option<Duration> {
		match (self.witness_commit, self.total_prove) {
			(Some(wc), Some(total)) => Some(total.saturating_sub(wc)),
			_ => None,
		}
	}
}

/// Shared state for the timing layer
#[derive(Default)]
struct TimingState {
	timings: PhaseTimings,
	span_starts: HashMap<Id, Instant>,
}

/// A tracing layer that captures phase timings
pub struct PhaseTimingLayer {
	state: Arc<Mutex<TimingState>>,
}

impl PhaseTimingLayer {
	pub fn new() -> (Self, PhaseTimingCollector) {
		let state = Arc::new(Mutex::new(TimingState::default()));
		let layer = Self {
			state: Arc::clone(&state),
		};
		let collector = PhaseTimingCollector { state };
		(layer, collector)
	}
}

/// Handle to collect the phase timings after the proof is complete
pub struct PhaseTimingCollector {
	state: Arc<Mutex<TimingState>>,
}

impl PhaseTimingCollector {
	pub fn collect(self) -> PhaseTimings {
		let state = self.state.lock().unwrap();
		state.timings.clone()
	}
}

impl<S> Layer<S> for PhaseTimingLayer
where
	S: Subscriber + for<'a> LookupSpan<'a>,
{
	fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
		// Check if this is a phase span we care about
		let name = attrs.metadata().name();
		if name.starts_with("[phase]") || name == "Prove" {
			let mut state = self.state.lock().unwrap();
			state.span_starts.insert(id.clone(), Instant::now());
		}
	}

	fn on_close(&self, id: Id, ctx: Context<'_, S>) {
		let span = ctx.span(&id);
		if let Some(span) = span {
			let name = span.name();
			let mut state = self.state.lock().unwrap();

			if let Some(start) = state.span_starts.remove(&id) {
				let duration = start.elapsed();

				match name {
					"[phase] Witness Commit" => {
						state.timings.witness_commit = Some(duration);
					}
					"[phase] IntMul Reduction" => {
						state.timings.intmul_reduction = Some(duration);
					}
					"[phase] BitAnd Reduction" => {
						state.timings.bitand_reduction = Some(duration);
					}
					"[phase] Shift Reduction" => {
						state.timings.shift_reduction = Some(duration);
					}
					"[phase] PCS Opening" => {
						state.timings.pcs_opening = Some(duration);
					}
					"Prove" => {
						state.timings.total_prove = Some(duration);
					}
					_ => {}
				}
			}
		}
	}
}
