// Copyright 2026 The Binius Developers

mod utils;

use std::alloc::System;

use binius_examples::circuits::ecdsa_verify::{EcdsaVerifyExample, Instance, Params};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use peakmem_alloc::PeakAlloc;
use utils::{ExampleBenchmark, SignBenchConfig, print_benchmark_header, run_cs_benchmark};

// Global allocator that tracks peak memory usage
#[global_allocator]
static ECDSA_VERIFY_PEAK_ALLOC: PeakAlloc<System> = PeakAlloc::new(System);

struct EcdsaVerifyBenchmark {
	config: SignBenchConfig,
}

impl EcdsaVerifyBenchmark {
	fn new() -> Self {
		let config = SignBenchConfig::from_env(1);
		Self { config }
	}
}

impl ExampleBenchmark for EcdsaVerifyBenchmark {
	type Params = Params;
	type Instance = Instance;
	type Example = EcdsaVerifyExample;

	fn create_params(&self) -> Self::Params {
		Params {
			n_signatures: self.config.n_signatures,
		}
	}

	fn create_instance(&self) -> Self::Instance {
		Instance {}
	}

	fn bench_name(&self) -> String {
		format!("sig_{}", self.config.n_signatures)
	}

	fn throughput(&self) -> Throughput {
		Throughput::Elements(self.config.n_signatures as u64)
	}

	fn proof_description(&self) -> String {
		format!("{} signatures", self.config.n_signatures)
	}

	fn log_inv_rate(&self) -> usize {
		self.config.log_inv_rate
	}

	fn print_params(&self) {
		let params_list = vec![
			("Signatures".to_string(), self.config.n_signatures.to_string()),
			("Message size".to_string(), "32 bytes (fixed)".to_string()),
			("Log inverse rate".to_string(), self.config.log_inv_rate.to_string()),
		];
		print_benchmark_header("ECDSA Verify", &params_list);
	}
}

fn bench_ecdsa_verify(c: &mut Criterion) {
	let benchmark = EcdsaVerifyBenchmark::new();
	run_cs_benchmark(c, benchmark, "ecdsa_verify", &ECDSA_VERIFY_PEAK_ALLOC);
}

criterion_group!(ecdsa_verify, bench_ecdsa_verify);
criterion_main!(ecdsa_verify);
