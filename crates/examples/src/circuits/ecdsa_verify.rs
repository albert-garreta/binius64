// Copyright 2026 The Binius Developers

use std::array;

use anyhow::Result;
use binius_circuits::{
	bignum::BigUint,
	ecdsa::bitcoin_verify,
	secp256k1::Secp256k1Affine,
};
use binius_core::word::Word;
use binius_frontend::{
	CircuitBuilder, Wire, WitnessFiller,
	util::pack_bytes_into_wires_le,
};
use clap::Args;
use ethsign::SecretKey;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::ExampleCircuit;

struct SignatureInputs {
	z: [Wire; 4],
	r: [Wire; 4],
	s: [Wire; 4],
	pk_x: [Wire; 4],
	pk_y: [Wire; 4],
}

/// Example circuit that verifies ECDSA signatures without hashing.
pub struct EcdsaVerifyExample {
	signatures: Vec<SignatureInputs>,
}

#[derive(Args, Debug, Clone)]
pub struct Params {
	/// Number of ECDSA signatures to verify
	#[arg(short = 'n', long, default_value_t = 1)]
	pub n_signatures: usize,
}

#[derive(Args, Debug, Clone)]
pub struct Instance {}

impl ExampleCircuit for EcdsaVerifyExample {
	type Params = Params;
	type Instance = Instance;

	fn build(params: Params, builder: &mut CircuitBuilder) -> Result<Self> {
		let signatures = (0..params.n_signatures)
			.map(|_| {
				let z = array::from_fn(|_| builder.add_inout());
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

				let z_big = BigUint { limbs: z.to_vec() };
				let r_big = BigUint { limbs: r.to_vec() };
				let s_big = BigUint { limbs: s.to_vec() };

				let signature_valid = bitcoin_verify(builder, pk, &z_big, &r_big, &s_big);
				let signature_valid_msb = builder.shr(signature_valid, 63);
				builder.assert_eq(
					"ecdsa_signature_valid",
					signature_valid_msb,
					builder.add_constant(Word::ONE),
				);

				SignatureInputs { z, r, s, pk_x, pk_y }
			})
			.collect();

		Ok(Self { signatures })
	}

	fn populate_witness(&self, _instance: Instance, w: &mut WitnessFiller) -> Result<()> {
		let mut rng = StdRng::seed_from_u64(42);

		for SignatureInputs { z, r, s, pk_x, pk_y } in &self.signatures {
			let sk_bytes: [u8; 32] = rng.random();
			let secret_key = SecretKey::from_raw(&sk_bytes)?;

			let z_bytes: [u8; 32] = rng.random();
			let mut signature = secret_key.sign(&z_bytes)?;
			let public = secret_key.public();

			let mut z_le = z_bytes;
			z_le.reverse();
			pack_bytes_into_wires_le(w, z, &z_le);

			signature.r.reverse();
			pack_bytes_into_wires_le(w, r, &signature.r);

			signature.s.reverse();
			pack_bytes_into_wires_le(w, s, &signature.s);

			let pk_bytes = public.bytes();
			let (pk_x_be, pk_y_be) = pk_bytes.split_at(32);

			let mut pk_x_le = pk_x_be.to_vec();
			pk_x_le.reverse();
			pack_bytes_into_wires_le(w, pk_x, &pk_x_le);

			let mut pk_y_le = pk_y_be.to_vec();
			pk_y_le.reverse();
			pack_bytes_into_wires_le(w, pk_y, &pk_y_le);
		}

		Ok(())
	}

	fn param_summary(params: &Self::Params) -> Option<String> {
		Some(format!("{}s", params.n_signatures))
	}
}
