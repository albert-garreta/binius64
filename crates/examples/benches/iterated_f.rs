// Copyright 2026 The Binius Developers
//! Iterated f benchmark

mod utils;

use std::alloc::System;

use binius_examples::circuits::iterated_f::{Instance, IteratedFExample, Params, ITERATIONS};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use peakmem_alloc::PeakAlloc;
use utils::{ExampleBenchmark, print_benchmark_header, run_cs_benchmark};

// Global allocator that tracks peak memory usage
#[global_allocator]
static ITERATED_F_PEAK_ALLOC: PeakAlloc<System> = PeakAlloc::new(System);

struct IteratedFBenchmark {
	log_inv_rate: usize,
}

impl IteratedFBenchmark {
	fn new() -> Self {
		let log_inv_rate = std::env::var("LOG_INV_RATE")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(1);
		Self { log_inv_rate }
	}
}

impl ExampleBenchmark for IteratedFBenchmark {
	type Params = Params;
	type Instance = Instance;
	type Example = IteratedFExample;

	fn create_params(&self) -> Self::Params {
		Params {}
	}

	fn create_instance(&self) -> Self::Instance {
		Instance { x0: Some(0x1234_5678) }
	}

	fn bench_name(&self) -> String {
		format!("iterations_{ITERATIONS}")
	}

	fn throughput(&self) -> Throughput {
		Throughput::Elements(ITERATIONS as u64)
	}

	fn proof_description(&self) -> String {
		format!("{ITERATIONS} iterations")
	}

	fn log_inv_rate(&self) -> usize {
		self.log_inv_rate
	}

	fn print_params(&self) {
		let params_list = vec![
			("Iterations".to_string(), ITERATIONS.to_string()),
			("x0".to_string(), format!("0x{:08x}", 0x1234_5678u32)),
			("Log inverse rate".to_string(), self.log_inv_rate.to_string()),
		];
		print_benchmark_header("Iterated f", &params_list);
	}
}

fn bench_iterated_f(c: &mut Criterion) {
	let benchmark = IteratedFBenchmark::new();
	run_cs_benchmark(c, benchmark, "iterated_f", &ITERATED_F_PEAK_ALLOC);
}

criterion_group!(benches, bench_iterated_f);
criterion_main!(benches);