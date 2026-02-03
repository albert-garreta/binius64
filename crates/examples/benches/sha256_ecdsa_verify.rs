// Copyright 2026 The Binius Developers
//! SHA-256 (2 KiB) + ECDSA verify benchmark

mod utils;

use std::alloc::System;

use binius_examples::circuits::sha256_ecdsa_verify::{Instance, Params, Sha256EcdsaVerifyExample};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use peakmem_alloc::PeakAlloc;
use utils::{ExampleBenchmark, SignBenchConfig, print_benchmark_header, run_cs_benchmark};

// Global allocator that tracks peak memory usage
#[global_allocator]
static SHA256_ECDSA_VERIFY_PEAK_ALLOC: PeakAlloc<System> = PeakAlloc::new(System);

const DEFAULT_MESSAGE_LEN_BYTES: usize = 2048;

struct Sha256EcdsaVerifyBenchmark {
	message_len_bytes: usize,
	log_inv_rate: usize,
}

impl Sha256EcdsaVerifyBenchmark {
	fn new() -> Self {
		let sign_config = SignBenchConfig::from_env(1);
		Self {
			message_len_bytes: DEFAULT_MESSAGE_LEN_BYTES,
			log_inv_rate: sign_config.log_inv_rate,
		}
	}
}

impl ExampleBenchmark for Sha256EcdsaVerifyBenchmark {
	type Params = Params;
	type Instance = Instance;
	type Example = Sha256EcdsaVerifyExample;

	fn create_params(&self) -> Self::Params {
		Params {
			message_len_bytes: self.message_len_bytes,
		}
	}

	fn create_instance(&self) -> Self::Instance {
		Instance {}
	}

	fn bench_name(&self) -> String {
		format!("message_bytes_{}", self.message_len_bytes)
	}

	fn throughput(&self) -> Throughput {
		Throughput::Bytes(self.message_len_bytes as u64)
	}

	fn proof_description(&self) -> String {
		format!("{} bytes message", self.message_len_bytes)
	}

	fn log_inv_rate(&self) -> usize {
		self.log_inv_rate
	}

	fn print_params(&self) {
		let params_list = vec![
			("Message size".to_string(), format!("{} bytes", self.message_len_bytes)),
			("Digest".to_string(), "SHA-256".to_string()),
			("Signature".to_string(), "ECDSA/secp256k1".to_string()),
			("Log inverse rate".to_string(), self.log_inv_rate.to_string()),
		];
		print_benchmark_header("SHA-256 + ECDSA Verify", &params_list);
	}
}

fn bench_sha256_ecdsa_verify(c: &mut Criterion) {
	let benchmark = Sha256EcdsaVerifyBenchmark::new();
	run_cs_benchmark(c, benchmark, "sha256_ecdsa_verify", &SHA256_ECDSA_VERIFY_PEAK_ALLOC);
}

criterion_group!(sha256_ecdsa_verify, bench_sha256_ecdsa_verify);
criterion_main!(sha256_ecdsa_verify);
