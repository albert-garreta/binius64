// Copyright 2025 Irreducible Inc.
//! Shared utilities for benchmarks

pub mod config;
pub mod phase_timing;
pub mod reporting;
pub mod runner;

// Re-export commonly used items
#[allow(unused_imports)]
pub use config::{HashBenchConfig, SignBenchConfig};
pub use phase_timing::{PhaseTimingCollector, PhaseTimingLayer, PhaseTimings};
pub use reporting::print_benchmark_header;
pub use runner::{ExampleBenchmark, run_cs_benchmark};
