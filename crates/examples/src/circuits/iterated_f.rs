// Copyright 2026 The Binius Developers
//! Iterated f circuit: apply $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^{14}(x)$
//! for a configurable number of steps starting from `x0`.
//!
//! This example demonstrates how to express a simple 32-bit update rule using
//! 64-bit words in the circuit, while enforcing the 32-bit range constraint.
//! The update rule is repeated `iterations` times and the final value is exposed
//! as a public output (`x_final`).
//!
//! The circuit supports parallel lanes, where each lane runs the same number of
//! iterations independently with its own initial and final values.
//!
//! Note: Benchmark is in crates/examples/benches/iterated_f.rs.
use anyhow::Result;
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller};
use clap::Args;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const DEFAULT_ITERATIONS: usize = 1 << 13;
pub const DEFAULT_LANES: usize = 1;
const ROT_RIGHT: u32 = 14;
const DEFAULT_RANDOM_SEED: u64 = 42;

pub struct IteratedFExample {
	/// Initial values for each lane.
	x0: Vec<Wire>,
	/// Final values for each lane.
	x_final: Vec<Wire>,
	iterations: usize,
	lanes: usize,
}

#[derive(Args, Debug, Clone, Default)]
pub struct Params {
	/// Number of iterations of f to apply per lane.
	#[arg(long, default_value_t = DEFAULT_ITERATIONS)]
	pub iterations: usize,

	/// Number of parallel lanes to execute.
	#[arg(long, default_value_t = DEFAULT_LANES)]
	pub lanes: usize,
}

#[derive(Args, Debug, Clone)]
pub struct Instance {
	/// Initial x values (32-bit unsigned) for each lane. If not provided or fewer values
	/// than lanes, deterministic random values are used for the missing lanes.
	#[arg(long, value_delimiter = ',')]
	pub x0: Option<Vec<u32>>,
}

impl ExampleCircuit for IteratedFExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let lanes = params.lanes;

		// Create input/output wires for each lane.
		let mut x0_wires = Vec::with_capacity(lanes);
		let mut x_final_wires = Vec::with_capacity(lanes);
		for _ in 0..lanes {
			x0_wires.push(builder.add_inout());
			x_final_wires.push(builder.add_inout());
		}

		// Shared mask for 32-bit range constraint.
		let mask = builder.add_constant_64(0xFFFF_FFFF);

		// Process each lane independently.
		for lane in 0..lanes {
			let x0 = x0_wires[lane];
			let x_final = x_final_wires[lane];

			// Ensure the initial witness value is a 32-bit unsigned integer by masking
			// and asserting equality inside the circuit.
			let x0_masked = builder.band(x0, mask);
			builder.assert_eq(format!("x0_32bit_lane{lane}"), x0, x0_masked);

			// Iterate the function in-circuit, keeping values in the low 32 bits.
			let mut x = x0;
			for _ in 0..params.iterations {
				// Compute x^2 as a 128-bit product (hi, lo). Only the low 32 bits are kept.
				let (_hi, lo) = builder.imul(x, x);
				let sq_lo = builder.band(lo, mask);
				// Rotate right in 32-bit space, then XOR with the masked square.
				let rot = builder.rotr_32(x, ROT_RIGHT);
				x = builder.bxor(sq_lo, rot);
			}

			// Public output must match the final iterated value.
			builder.assert_eq(format!("final_matches_lane{lane}"), x, x_final);
		}

		Ok(Self {
			x0: x0_wires,
			x_final: x_final_wires,
			iterations: params.iterations,
			lanes,
		})
	}

	fn populate_witness(&self, instance: Instance, filler: &mut WitnessFiller) -> Result<()> {
		// Get provided x0 values or empty vec.
		let provided_x0 = instance.x0.unwrap_or_default();

		// Use a seeded RNG for deterministic random values.
		let mut rng = StdRng::seed_from_u64(DEFAULT_RANDOM_SEED);

		for lane in 0..self.lanes {
			// Use provided value if available, otherwise generate a deterministic random value.
			let x0_value = if lane < provided_x0.len() {
				provided_x0[lane]
			} else {
				rng.random::<u32>()
			};

			// Mirror the circuit logic in plain Rust to compute the expected output.
			let mut x = x0_value;
			for _ in 0..self.iterations {
				x = x.wrapping_mul(x) ^ x.rotate_right(ROT_RIGHT);
			}

			// Fill witness wires with 64-bit words containing the 32-bit values.
			filler[self.x0[lane]] = Word(x0_value as u64);
			filler[self.x_final[lane]] = Word(x as u64);
		}

		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		if params.lanes == 1 {
			Some(format!("{}i", params.iterations))
		} else {
			Some(format!("{}i_{}l", params.iterations, params.lanes))
		}
	}
}