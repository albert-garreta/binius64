// Copyright 2026 The Binius Developers
//! Iterated f benchmark

// cargo bench -p binius-examples --bench iterated_f

mod utils;

use std::alloc::System;

use binius_examples::circuits::iterated_f::{
	DEFAULT_ITERATIONS, DEFAULT_LANES, Instance, IteratedFExample, Params,
};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use peakmem_alloc::PeakAlloc;
use utils::{ExampleBenchmark, print_benchmark_header, run_cs_benchmark};

// Global allocator that tracks peak memory usage
#[global_allocator]
static ITERATED_F_PEAK_ALLOC: PeakAlloc<System> = PeakAlloc::new(System);

struct IteratedFBenchmark {
	log_inv_rate: usize,
	iterations: usize,
	lanes: usize,
}

impl IteratedFBenchmark {
	fn new() -> Self {
		let log_inv_rate = std::env::var("LOG_INV_RATE")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(1);
		let iterations = std::env::var("ITERATIONS")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(DEFAULT_ITERATIONS);
		let lanes = std::env::var("LANES")
			.ok()
			.and_then(|s| s.parse::<usize>().ok())
			.unwrap_or(DEFAULT_LANES);
		Self {
			log_inv_rate,
			iterations,
			lanes,
		}
	}
}

impl ExampleBenchmark for IteratedFBenchmark {
	type Params = Params;
	type Instance = Instance;
	type Example = IteratedFExample;

	fn create_params(&self) -> Self::Params {
		Params {
			iterations: self.iterations,
			lanes: self.lanes,
		}
	}

	fn create_instance(&self) -> Self::Instance {
		// Provide x0 values for all lanes (using the same pattern)
		let x0_values: Vec<u32> = (0..self.lanes)
			.map(|i| 0x1234_5678u32.wrapping_add(i as u32))
			.collect();
		Instance { x0: Some(x0_values) }
	}

	fn bench_name(&self) -> String {
		if self.lanes == 1 {
			format!("iterations_{}", self.iterations)
		} else {
			format!("iterations_{}_lanes_{}", self.iterations, self.lanes)
		}
	}

	fn throughput(&self) -> Throughput {
		Throughput::Elements((self.iterations * self.lanes) as u64)
	}

	fn proof_description(&self) -> String {
		if self.lanes == 1 {
			format!("{} iterations", self.iterations)
		} else {
			format!("{} iterations x {} lanes", self.iterations, self.lanes)
		}
	}

	fn log_inv_rate(&self) -> usize {
		self.log_inv_rate
	}

	fn print_params(&self) {
		let params_list = vec![
			("Iterations".to_string(), self.iterations.to_string()),
			("Lanes".to_string(), self.lanes.to_string()),
			("x0[0]".to_string(), format!("0x{:08x}", 0x1234_5678u32)),
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