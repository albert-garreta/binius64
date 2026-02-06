// Copyright 2026 The Binius Developers
//! Iterated g32 benchmark

// cargo bench -p binius-examples --bench iterated_g32

mod utils;

use std::alloc::System;

use binius_examples::circuits::iterated_g32::{
	DEFAULT_ITERATIONS, Instance, IteratedG32Example, Params,
};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use peakmem_alloc::PeakAlloc;
use utils::{ExampleBenchmark, print_benchmark_header, run_cs_benchmark};

// Global allocator that tracks peak memory usage
#[global_allocator]
static ITERATED_G32_PEAK_ALLOC: PeakAlloc<System> = PeakAlloc::new(System);

struct IteratedG32Benchmark {
	log_inv_rate: usize,
	iterations: usize,
}

impl IteratedG32Benchmark {
	fn new() -> Self {
		let log_inv_rate = std::env::var("LOG_INV_RATE")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(1);
		let iterations = std::env::var("ITERATIONS")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(DEFAULT_ITERATIONS);
		Self {
			log_inv_rate,
			iterations,
		}
	}
}

impl ExampleBenchmark for IteratedG32Benchmark {
	type Params = Params;
	type Instance = Instance;
	type Example = IteratedG32Example;

	fn create_params(&self) -> Self::Params {
		Params {
			iterations: self.iterations,
		}
	}

	fn create_instance(&self) -> Self::Instance {
		Instance { x0: Some(0x1234_5678) }
	}

	fn bench_name(&self) -> String {
		format!("iterations_{}", self.iterations)
	}

	fn throughput(&self) -> Throughput {
		Throughput::Elements(self.iterations as u64)
	}

	fn proof_description(&self) -> String {
		format!("{} iterations", self.iterations)
	}

	fn log_inv_rate(&self) -> usize {
		self.log_inv_rate
	}

	fn print_params(&self) {
		let params_list = vec![
			("Iterations".to_string(), self.iterations.to_string()),
			("x0".to_string(), format!("0x{:08x}", 0x1234_5678u32)),
			("Log inverse rate".to_string(), self.log_inv_rate.to_string()),
		];
		print_benchmark_header("Iterated g32", &params_list);
	}
}

fn bench_iterated_g32(c: &mut Criterion) {
	let benchmark = IteratedG32Benchmark::new();
	run_cs_benchmark(c, benchmark, "iterated_g32", &ITERATED_G32_PEAK_ALLOC);
}

criterion_group!(benches, bench_iterated_g32);
criterion_main!(benches);
