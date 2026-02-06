// Copyright 2026 The Binius Developers
//! Iterated g circuit: apply $g(x) = preffix_32((f(x) \cdot (2^{32\cdot 7} - 1))^2 \bmod q)$ for a
//! configurable number of steps, where preffix_32 keeps the first 32 bits of an integer, and where
//! $f(x) = (x^2 \bmod 2^{32}) \oplus \operatorname{ROTR}^{14}(x)$ and
//! $q$ is the secp256k1 scalar field modulus.
//!
//! This example mirrors the `iterated_f` circuit, but each iteration squares the
//! 32-bit `f(x)` output in the ECDSA scalar field.
//!
//! Note: Benchmark is in crates/examples/benches/iterated_g.rs.
use std::iter;

use anyhow::Result;
use binius_circuits::bignum::{BigUint, PseudoMersennePrimeField};
use binius_core::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller, util::num_biguint_from_u64_limbs};
use clap::Args;
use num_bigint::BigUint as NumBigUint;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

pub const DEFAULT_ITERATIONS: usize = 1 << 14;
const ROT_RIGHT: u32 = 14;
const DEFAULT_RANDOM_SEED: u64 = 42;
const SCALAR_MODULUS_SUBTRAHEND: [u64; 3] = [0x402da1732fc9bebf, 0x4551231950b75fc4, 1];
const F_SCALE_LIMBS: [u64; 4] = [
	0xFFFF_FFFF_FFFF_FFFF,
	0xFFFF_FFFF_FFFF_FFFF,
	0xFFFF_FFFF_FFFF_FFFF,
	0x0000_0000_FFFF_FFFF,
];

pub struct IteratedGExample {
	x0: Wire,
	x_final: Wire,
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

impl ExampleCircuit for IteratedGExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let x0 = builder.add_inout();
		let field =
			PseudoMersennePrimeField::new(builder, 256, &SCALAR_MODULUS_SUBTRAHEND);
		let x_final = builder.add_inout();

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
		let f_scale = BigUint::new_constant(builder, &num_biguint_from_u64_limbs(F_SCALE_LIMBS))
			.pad_limbs_to(field.limbs_len(), zero);

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
			let f_scaled = field.mul(builder, &f_big, &f_scale);
			x = field.square(builder, &f_scaled);
		}

		// Extract prefix_32 (first 32 bits) of the final result
		let x_prefix = builder.band(x.limbs[0], mask);
		builder.assert_eq("final_matches", x_prefix, x_final);

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

		let modulus_subtrahend = num_biguint_from_u64_limbs(SCALAR_MODULUS_SUBTRAHEND);
		let modulus = NumBigUint::from(2u8).pow(256u32) - modulus_subtrahend;
		let f_scale = num_biguint_from_u64_limbs(F_SCALE_LIMBS);

		let mut x = NumBigUint::from(x0_value as u64);
		for _ in 0..self.iterations {
			let x_low = x.iter_u64_digits().next().unwrap_or(0) as u32;
			let f_value = x_low.wrapping_mul(x_low) ^ x_low.rotate_right(ROT_RIGHT);
			let f_big = NumBigUint::from(f_value as u64);
			let f_scaled = (&f_big * &f_scale) % &modulus;
			x = (&f_scaled * &f_scaled) % &modulus;
		}

		filler[self.x0] = Word(x0_value as u64);
		// Extract prefix_32 (first 32 bits) of the final result
		let x_prefix = x.iter_u64_digits().next().unwrap_or(0) as u32;
		filler[self.x_final] = Word(x_prefix as u64);

		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}i", params.iterations))
	}
}
