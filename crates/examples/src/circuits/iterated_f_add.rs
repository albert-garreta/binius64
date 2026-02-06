// Copyright 2026 The Binius Developers
//! Iterated f add circuit: apply $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^{14}(x)$
//! for a configurable number of steps starting from `x0`, using 32-bit addition
//! to truncate values instead of bitwise AND masking.
//!
//! This example mirrors the `iterated_f` circuit, but uses `iadd_32` to enforce
//! the 32-bit truncation. The function is iterated `iterations` times, and the
//! final value is exposed as a public output (`x_final`).
//!
//! Note: Benchmark is in crates/examples/benches/iterated_f_add.rs.
use anyhow::Result;
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller};
use clap::Args;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const DEFAULT_ITERATIONS: usize = 1 << 16;
const ROT_RIGHT: u32 = 14;
const DEFAULT_RANDOM_SEED: u64 = 42;

/// Circuit container that wires inputs/outputs and iteration count together.
pub struct IteratedFAddExample {
	x0: Wire,
	x_final: Wire,
	iterations: usize,
}

#[derive(Args, Debug, Clone, Default)]
pub struct Params {
	/// Number of iterations of f to apply.
	#[arg(long, default_value_t = DEFAULT_ITERATIONS)]
	pub iterations: usize,
}

#[derive(Args, Debug, Clone)]
pub struct Instance {
	/// Initial x value (32-bit unsigned). If not provided, a deterministic random value is used.
	#[arg(long)]
	pub x0: Option<u32>,
}

impl ExampleCircuit for IteratedFAddExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let x0 = builder.add_inout();
		let x_final = builder.add_inout();

		// Enforce that the input fits into 32 bits by truncating with 32-bit add.
		let zero = builder.add_constant_64(0);
		let x0_trunc = builder.iadd_32(x0, zero);
		builder.assert_eq("x0_32bit", x0, x0_trunc);

		// Iterate the function in-circuit, keeping values in the low 32 bits.
		let mut x = x0;
		for _ in 0..params.iterations {
			// Compute x^2 as a 128-bit product (hi, lo) and truncate via 32-bit add.
			let (_hi, lo) = builder.imul(x, x);
			let sq_lo = builder.iadd_32(lo, zero);
			// Rotate right in 32-bit space, then XOR with the truncated square.
			let rot = builder.rotr_32(x, ROT_RIGHT);
			x = builder.bxor(sq_lo, rot);
		}

		// Public output must match the final iterated value.
		builder.assert_eq("final_matches", x, x_final);

		Ok(Self {
			x0,
			x_final,
			iterations: params.iterations,
		})
	}

	fn populate_witness(&self, instance: Instance, filler: &mut WitnessFiller) -> Result<()> {
		// Use the provided instance value, or generate a deterministic pseudo-random
		// 32-bit value to keep benchmarking reproducible.
		let x0_value = match instance.x0 {
			Some(value) => value,
			None => {
				let mut rng = StdRng::seed_from_u64(DEFAULT_RANDOM_SEED);
				rng.random::<u32>()
			}
		};

		// Mirror the circuit logic in plain Rust to compute the expected output.
		let mut x = x0_value;
		for _ in 0..self.iterations {
			x = x.wrapping_mul(x) ^ x.rotate_right(ROT_RIGHT);
		}

		// Fill witness wires with 64-bit words containing the 32-bit values.
		filler[self.x0] = Word(x0_value as u64);
		filler[self.x_final] = Word(x as u64);
		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}i", params.iterations))
	}
}