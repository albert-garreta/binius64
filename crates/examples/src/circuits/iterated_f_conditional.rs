// Copyright 2026 The Binius Developers
//! Iterated f conditional circuit: apply $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^{14}(x)$
//! for a configurable number of steps starting from `x0`.
//!
//! This example mirrors the `iterated_f` circuit but with an additional input `y` in
//! $\{1, \dots, N\}$ that controls which iterations apply $f$: at iteration $i \in [1, N]$,
//! if $y + i > N$, then update $x$ to $f(x)$; otherwise keep $x$ unchanged.
//!
//! Note: Benchmark is in crates/examples/benches/iterated_f_conditional.rs.
use anyhow::Result;
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller};
use clap::Args;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const DEFAULT_ITERATIONS: usize = 1 << 13;
const ROT_RIGHT: u32 = 14;
const DEFAULT_RANDOM_SEED: u64 = 42;

pub struct IteratedFConditionalExample {
	x0: Wire,
	y: Wire,
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
	/// Selector y in {1, ..., N} controlling which iterations apply f.
	#[arg(long)]
	pub y: Option<u32>,
}

impl ExampleCircuit for IteratedFConditionalExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let x0 = builder.add_inout();
		let y = builder.add_inout();
		let x_final = builder.add_inout();

		// Ensure the initial witness value is a 32-bit unsigned integer by masking
		// and asserting equality inside the circuit.
		let mask = builder.add_constant_64(0xFFFF_FFFF);
		let x0_masked = builder.band(x0, mask);
		builder.assert_eq("x0_32bit", x0, x0_masked);

		let zero = builder.add_constant(Word::ZERO);
		let all_one = builder.add_constant(Word::ALL_ONE);
		let n_const = builder.add_constant_64(params.iterations as u64);
		let y_gt_zero = builder.icmp_ugt(y, zero);
		let y_le_n = builder.icmp_ule(y, n_const);
		let y_gt_zero_mask = builder.sar(y_gt_zero, 63);
		let y_le_n_mask = builder.sar(y_le_n, 63);
		let y_valid = builder.band(y_gt_zero_mask, y_le_n_mask);
		builder.assert_eq("y_in_range", y_valid, all_one);

		// Iterate the function in-circuit, keeping values in the low 32 bits.
		let mut x = x0;
		for i in 1..=params.iterations {
			// Compute x^2 as a 128-bit product (hi, lo). Only the low 32 bits are kept.
			let (_hi, lo) = builder.imul(x, x);
			let sq_lo = builder.band(lo, mask);
			// Rotate right in 32-bit space, then XOR with the masked square.
			let rot = builder.rotr_32(x, ROT_RIGHT);
			let x_next = builder.bxor(sq_lo, rot);

			let y_plus_i = builder.iadd_32(y, builder.add_constant_64(i as u64));
			let apply = builder.icmp_ugt(y_plus_i, n_const);
			x = builder.select(apply, x_next, x);
		}

		// Public output must match the final iterated value.
		builder.assert_eq("final_matches", x, x_final);

		Ok(Self {
			x0,
			y,
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
		let y_value = match instance.y {
			Some(value) => value,
			None => {
				let mut rng = StdRng::seed_from_u64(DEFAULT_RANDOM_SEED);
				(rng.random::<u32>() % (self.iterations as u32)) + 1
			}
		};

		// Mirror the circuit logic in plain Rust to compute the expected output.
		let mut x = x0_value;
		for i in 1..=self.iterations {
			if (y_value as usize) + i > self.iterations {
				x = x.wrapping_mul(x) ^ x.rotate_right(ROT_RIGHT);
			}
		}

		// Fill witness wires with 64-bit words containing the 32-bit values.
		filler[self.x0] = Word(x0_value as u64);
		filler[self.y] = Word(y_value as u64);
		filler[self.x_final] = Word(x as u64);
		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}i", params.iterations))
	}
}
