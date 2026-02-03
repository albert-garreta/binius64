// Copyright 2026 The Binius Developers

use std::array;

use anyhow::{Result, ensure};
use binius_circuits::{
	bignum::BigUint,
	ecdsa::bitcoin_verify,
	secp256k1::Secp256k1Affine,
	sha256::Sha256,
};
use binius_core::word::Word;
use binius_frontend::{CircuitBuilder, Wire, WitnessFiller, util::pack_bytes_into_wires_le};
use clap::Args;
use ethsign::SecretKey;
use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};
use sha2::Digest;

use crate::ExampleCircuit;

struct SignatureInputs {
	r: [Wire; 4],
	s: [Wire; 4],
	pk_x: [Wire; 4],
	pk_y: [Wire; 4],
}

/// Example circuit that hashes a message with SHA-256 and verifies an ECDSA signature.
pub struct Sha256EcdsaVerifyExample {
	sha256: Sha256,
	message_len_bytes: usize,
	signature: SignatureInputs,
}

#[derive(Args, Debug, Clone)]
pub struct Params {
	/// Message length in bytes (fixed at circuit build time).
	#[arg(long, default_value_t = 2048)]
	pub message_len_bytes: usize,
}

#[derive(Args, Debug, Clone)]
pub struct Instance {}

impl ExampleCircuit for Sha256EcdsaVerifyExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		ensure!(params.message_len_bytes > 0, "message_len_bytes must be positive");
		let message_len_words = params.message_len_bytes.div_ceil(8);
		let len_bytes = builder.add_constant_64(params.message_len_bytes as u64);

		let digest: [Wire; 4] = array::from_fn(|_| builder.add_witness());
		let z_big = BigUint {
			limbs: digest.iter().rev().copied().collect(),
		};
		let message = (0..message_len_words)
			.map(|_| builder.add_witness())
			.collect();
		let sha256 = Sha256::new(builder, len_bytes, digest, message);

		let r = array::from_fn(|_| builder.add_inout());
		let s = array::from_fn(|_| builder.add_inout());
		let pk_x = array::from_fn(|_| builder.add_inout());
		let pk_y = array::from_fn(|_| builder.add_inout());

		let pk = Secp256k1Affine {
			x: BigUint {
				limbs: pk_x.to_vec(),
			},
			y: BigUint {
				limbs: pk_y.to_vec(),
			},
			is_point_at_infinity: builder.add_constant(Word::ZERO),
		};

		let r_big = BigUint { limbs: r.to_vec() };
		let s_big = BigUint { limbs: s.to_vec() };

		let signature_valid = bitcoin_verify(builder, pk, &z_big, &r_big, &s_big);
		let signature_valid_msb = builder.shr(signature_valid, 63);
		builder.assert_eq(
			"ecdsa_signature_valid",
			signature_valid_msb,
			builder.add_constant(Word::ONE),
		);

		Ok(Self {
			sha256,
			message_len_bytes: params.message_len_bytes,
			signature: SignatureInputs { r, s, pk_x, pk_y },
		})
	}

	fn populate_witness(&self, _instance: Instance, w: &mut WitnessFiller) -> Result<()> {
		let mut rng = StdRng::seed_from_u64(42);

		let mut message_bytes = vec![0u8; self.message_len_bytes];
		rng.fill_bytes(&mut message_bytes);

		let digest = sha2::Sha256::digest(&message_bytes);
		let digest_bytes: [u8; 32] = digest.into();

		self.sha256
			.populate_len_bytes(w, self.message_len_bytes);
		self.sha256.populate_message(w, &message_bytes);
		self.sha256.populate_digest(w, digest_bytes);

		let sk_bytes: [u8; 32] = rng.random();
		let secret_key = SecretKey::from_raw(&sk_bytes)?;
		let mut signature = secret_key.sign(&digest_bytes)?;
		let public = secret_key.public();

		signature.r.reverse();
		pack_bytes_into_wires_le(w, &self.signature.r, &signature.r);

		signature.s.reverse();
		pack_bytes_into_wires_le(w, &self.signature.s, &signature.s);

		let pk_bytes = public.bytes();
		let (pk_x_be, pk_y_be) = pk_bytes.split_at(32);

		let mut pk_x_le = pk_x_be.to_vec();
		pk_x_le.reverse();
		pack_bytes_into_wires_le(w, &self.signature.pk_x, &pk_x_le);

		let mut pk_y_le = pk_y_be.to_vec();
		pk_y_le.reverse();
		pack_bytes_into_wires_le(w, &self.signature.pk_y, &pk_y_le);

		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}b", params.message_len_bytes))
	}
}
