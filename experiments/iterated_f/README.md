# Iterated f Benchmark Experiment

This document describes the `iterated_f` benchmark circuit used for evaluating the performance of the Binius64 zero-knowledge proof system.

## Citation

If you use this benchmark in academic work, please use:

```bibtex
@misc{binius64-iterated-f,
  author = {{The Binius Developers}},
  title = {Iterated f Benchmark for Binius64},
  year = {2026},
  url = {https://github.com/binius-zk/binius64},
  note = {Accessed: YYYY-MM-DD}
}
```

For the Binius protocol itself, see the references at [binius.xyz](https://www.binius.xyz).

## Function Definition

The circuit computes the iterated application of the function:

$$
f(x) = \left(x^2 \bmod 2^{32}\right) \oplus \operatorname{ROTR}^{14}(x)
$$

where:
- $x$ is a 32-bit unsigned integer
- $x^2 \bmod 2^{32}$ computes the square, keeping only the low 32 bits
- $\operatorname{ROTR}^{14}(x)$ rotates $x$ right by 14 bits (in 32-bit space)
- $\oplus$ denotes bitwise XOR

Starting from an initial value $x_0$, the circuit proves knowledge of the sequence:

$$
x_1 = f(x_0), \quad x_2 = f(x_1), \quad \ldots, \quad x_n = f(x_{n-1})
$$

Both $x_0$ (input) and $x_n$ (output) are public values.

### Circuit Source

The circuit implementation is at:
- **Source**: [`crates/examples/src/circuits/iterated_f.rs`](crates/examples/src/circuits/iterated_f.rs)
- **Benchmark**: [`crates/examples/benches/iterated_f.rs`](crates/examples/benches/iterated_f.rs)

## Parameters

| Parameter    | Description                                      | Default  |
|--------------|--------------------------------------------------|----------|
| `iterations` | Number of times to apply $f$ per lane            | $2^{13}$ |
| `lanes`      | Number of parallel, independent $f$ chains       | 1        |
| `x0`         | Initial value(s), comma-separated (32-bit each)  | Random   |

The total number of function applications is `lanes × iterations`.

## Dependencies

- [Rust](https://www.rust-lang.org/) (see `rust-toolchain.toml` for version)
- [Cargo](https://doc.rust-lang.org/cargo/) (included with Rust)
- Python 3.10+ (for automated benchmark script)

## Running the Benchmark

### Prerequisites

```bash
# Clone the repository
git clone https://github.com/binius-zk/binius64.git
cd binius64

# Enable native CPU optimizations (recommended)
export RUSTFLAGS="-C target-cpu=native"
```

### Single Configuration

Run with default parameters:
```bash
cargo bench -p binius-examples --bench iterated_f
```

Run with custom parameters via environment variables:
```bash
ITERATIONS=32768 LANES=4 cargo bench -p binius-examples --bench iterated_f
```

### Proof Generation and Verification

You can also run the circuit through the prover/verifier CLI:

```bash
# Generate a proof
cargo run --release -p binius-examples --bin prover -- iterated-f --iterations 8192 --lanes 4

# Verify a proof (from a saved proof file)
cargo run --release -p binius-examples --bin verifier -- proof.bin
```

### Automated Benchmark Suite

A Python script is provided to run the full benchmark matrix and generate a LaTeX table:

```bash
# Run all benchmarks and output LaTeX table
python3 tools/bench_iterated_f.py

# Save LaTeX table to file
python3 tools/bench_iterated_f.py --output results.tex

# Only collect existing results (skip running benchmarks)
python3 tools/bench_iterated_f.py --skip-run
```

The script runs the following configurations:

| Lanes | Iterations Range           | Total Operations Range     |
|-------|----------------------------|----------------------------|
| 1     | $2^{13}$ to $2^{17}$       | $2^{13}$ to $2^{17}$       |
| 4     | $2^{11}$ to $2^{15}$       | $2^{13}$ to $2^{17}$       |
| 16    | $2^{9}$ to $2^{13}$        | $2^{13}$ to $2^{17}$       |

## Benchmark Phases

The benchmark measures three phases:

1. **Witness Generation**: Time to compute the intermediate values and populate circuit wires
2. **Proof Generation**: Time to generate the zero-knowledge proof
3. **Proof Verification**: Time to verify the proof

Results are saved to `target/criterion/iterated_f_*/` in JSON format.

## Circuit Complexity

Per iteration, the circuit uses:
- 1 multiplication (`imul` for squaring)
- 3 bitwise AND operations (`band` for masking)
- 1 32-bit right rotation (`rotr_32`)
- 1 XOR operation (`bxor`)

Per lane, the circuit additionally enforces:
- 1 range check (32-bit constraint on input)
- 1 equality assertion (output matches computed result)

## Hardware Requirements

- **Minimum**: 4 GB RAM, any 64-bit CPU
- **Recommended**: 16+ GB RAM, CPU with AVX2/NEON support

The Binius64 prover automatically detects and uses available SIMD instructions.

## Output Format

The benchmark script generates a LaTeX table with the following columns:

| Column        | Description                                  |
|---------------|----------------------------------------------|
| Lanes         | Number of parallel lanes                     |
| Iterations    | Iterations per lane                          |
| Total Ops     | Total function applications (lanes × iters)  |
| Witness Gen   | Witness generation time                      |
| Proof Gen     | Proof generation time                        |
| Verification  | Proof verification time                      |

## Reproducibility Notes

- **Deterministic inputs**: When `x0` is not specified, the circuit uses a seeded PRNG (seed=42) for reproducible benchmarks
- **Compiler flags**: Use `RUSTFLAGS="-C target-cpu=native"` for fair performance comparison
- **Warm-up**: Criterion.rs handles warm-up automatically; results reflect steady-state performance
- **Multiple samples**: Each measurement is the mean of multiple iterations with confidence intervals

## License

This code is dual-licensed under Apache 2.0 and MIT licenses. See [LICENSE-Apache-2.0.txt](LICENSE-Apache-2.0.txt) and [LICENSE-MIT.txt](LICENSE-MIT.txt).

## Related Documentation

- [Binius64 Protocol Blueprint](https://www.binius.xyz/blueprint) — Cryptographic protocol specification
- [Building with Binius](https://www.binius.xyz/building) — Developer guides
- [API Documentation](https://docs.binius.xyz) — Rust API reference
