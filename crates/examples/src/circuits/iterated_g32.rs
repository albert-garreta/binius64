// Copyright 2026 The Binius Developers
//! Iterated g32 circuit: apply $g(x) = f(x)^2 \bmod q$ for a configurable number of steps,
//! where $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^{14}(x)$ and
//! $q = 2^{32} - 5$.
//!
//! This example mirrors the `iterated_g` circuit, but each iteration squares the
//! 32-bit `f(x)` output in a small 32-bit prime field.
//!
//! Note: Benchmark is in crates/examples/benches/iterated_g32.rs.
use std::iter;

use anyhow::Result;
use binius_circuits::bignum::{BigUint, PseudoMersennePrimeField, assert_eq};
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller, util::num_biguint_from_u64_limbs};
use clap::Args;
use num_bigint::BigUint as NumBigUint;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const DEFAULT_ITERATIONS: usize = 1 << 15;
const ROT_RIGHT: u32 = 14;
const DEFAULT_RANDOM_SEED: u64 = 42;
const SMALL_MODULUS_BITS: usize = 64;
const SMALL_MODULUS_SUBTRAHEND: [u64; 1] = [0xFFFF_FFFF_0000_0005];

pub struct IteratedG32Example {
	x0: Wire,
	x_final: BigUint,
	iterations: usize,
}

#[derive(Args, Debug, Clone, Default)]
pub struct Params {
	/// Number of iterations of g to apply.
	#[arg(long, default_value_t = DEFAULT_ITERATIONS)]
	pub iterations: usize,
}

#[derive(Args, Debug, Clone)]
pub struct Instance {
	/// Initial x value (32-bit unsigned). If not provided, a deterministic random value is used.
	#[arg(long)]
	pub x0: Option<u32>,
}

impl ExampleCircuit for IteratedG32Example {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let x0 = builder.add_inout();
		let field =
			PseudoMersennePrimeField::new(builder, SMALL_MODULUS_BITS, &SMALL_MODULUS_SUBTRAHEND);
		let x_final = BigUint::new_inout(builder, field.limbs_len());

		// Ensure the initial witness value is a 32-bit unsigned integer by masking
		// and asserting equality inside the circuit.
		let mask = builder.add_constant_64(0xFFFF_FFFF);
		let x0_masked = builder.band(x0, mask);
		builder.assert_eq("x0_32bit", x0, x0_masked);

		let zero = builder.add_constant(Word::ZERO);
		let mut x = BigUint {
			limbs: iter::once(x0)
				.chain(iter::repeat(zero).take(field.limbs_len() - 1))
				.collect(),
		};

		for _ in 0..params.iterations {
			// Compute f(x) from the low 32 bits.
			let x_low = builder.band(x.limbs[0], mask);
			let (_hi, lo) = builder.imul(x_low, x_low);
			let sq_lo = builder.band(lo, mask);
			let rot = builder.rotr_32(x_low, ROT_RIGHT);
			let f_word = builder.bxor(sq_lo, rot);

			let f_big = BigUint {
				limbs: iter::once(f_word)
					.chain(iter::repeat(zero).take(field.limbs_len() - 1))
					.collect(),
			};
			x = field.square(builder, &f_big);
		}

		assert_eq(builder, "final_matches", &x, &x_final);

		Ok(Self {
			x0,
			x_final,
			iterations: params.iterations,
		})
	}

	fn populate_witness(&self, instance: Instance, filler: &mut WitnessFiller) -> Result<()> {
		let x0_value = match instance.x0 {
			Some(value) => value,
			None => {
				let mut rng = StdRng::seed_from_u64(DEFAULT_RANDOM_SEED);
				rng.random::<u32>()
			}
		};

		let modulus_subtrahend = num_biguint_from_u64_limbs(SMALL_MODULUS_SUBTRAHEND);
		let modulus = NumBigUint::from(2u8).pow(SMALL_MODULUS_BITS as u32) - modulus_subtrahend;

		let mut x = NumBigUint::from(x0_value as u64);
		for _ in 0..self.iterations {
			let x_low = x.iter_u64_digits().next().unwrap_or(0) as u32;
			let f_value = x_low.wrapping_mul(x_low) ^ x_low.rotate_right(ROT_RIGHT);
			let f_big = NumBigUint::from(f_value as u64);
			x = (&f_big * &f_big) % &modulus;
		}

		filler[self.x0] = Word(x0_value as u64);
		let mut limb_values = vec![0u64; self.x_final.limbs.len()];
		for (idx, limb) in x.iter_u64_digits().enumerate() {
			if idx < limb_values.len() {
				limb_values[idx] = limb;
			}
		}
		self.x_final.populate_limbs(filler, &limb_values);

		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}i", params.iterations))
	}
}
