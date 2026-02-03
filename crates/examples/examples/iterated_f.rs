// Copyright 2026 The Binius Developers
use anyhow::Result;
use binius_examples::{Cli, circuits::iterated_f::IteratedFExample};

fn main() -> Result<()> {
	Cli::<IteratedFExample>::new("iterated_f")
		.about("Iterate f(x) = (x^2 mod 2^32) XOR ROTR^3(x) for 2^10 steps")
		.run()
}