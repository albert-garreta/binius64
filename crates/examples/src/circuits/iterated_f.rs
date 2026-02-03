// Copyright 2026 The Binius Developers
//! Iterated f circuit: apply $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^3(x)$
//! for `ITERATIONS` steps starting from `x0`.
//!
//! Note: Benchmark is in crates/examples/benches/iterated_f.rs.
use anyhow::Result;
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller};
use clap::Args;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const ITERATIONS: usize = 1 << 13;
const ROT_RIGHT: u32 = 3;
const DEFAULT_RANDOM_SEED: u64 = 42;

pub struct IteratedFExample {
	x0: Wire,
	x_final: Wire,
}

#[derive(Args, Debug, Clone, Default)]
pub struct Params {}

#[derive(Args, Debug, Clone)]
pub struct Instance {
	/// Initial x value (32-bit unsigned). If not provided, a deterministic random value is used.
	#[arg(long)]
	pub x0: Option<u32>,
}

impl ExampleCircuit for IteratedFExample {
	type Params = Params;
	type Instance = Instance;

	fn build(_params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let x0 = builder.add_inout();
		let x_final = builder.add_inout();

		let mask = builder.add_constant_64(0xFFFF_FFFF);
		let x0_masked = builder.band(x0, mask);
		builder.assert_eq("x0_32bit", x0, x0_masked);

		let mut x = x0;
		for _ in 0..ITERATIONS {
			let (_hi, lo) = builder.imul(x, x);
			let sq_lo = builder.band(lo, mask);
			let rot = builder.rotr_32(x, ROT_RIGHT);
			x = builder.bxor(sq_lo, rot);
		}

		builder.assert_eq("final_matches", x, x_final);

		Ok(Self { x0, x_final })
	}

	fn populate_witness(&self, instance: Instance, filler: &mut WitnessFiller) -> Result<()> {
		let x0_value = match instance.x0 {
			Some(value) => value,
			None => {
				let mut rng = StdRng::seed_from_u64(DEFAULT_RANDOM_SEED);
				rng.random::<u32>()
			}
		};

		let mut x = x0_value;
		for _ in 0..ITERATIONS {
			x = x.wrapping_mul(x) ^ x.rotate_right(ROT_RIGHT);
		}

		filler[self.x0] = Word(x0_value as u64);
		filler[self.x_final] = Word(x as u64);
		Ok(())
	}

	fn param_summary(_params: &Self::Params) -> Option<String> {
		Some(format!("{}i", ITERATIONS))
	}
}