#!/usr/bin/env python3
# Copyright 2026 The Binius Developers
"""
Benchmark iterated_f circuit with various configurations and generate LaTeX table.

This script runs the iterated_f benchmark with different combinations of parallel
lanes and iterations, then outputs the results as a LaTeX table.

Usage:
    python tools/bench_iterated_f.py [--output results.tex]
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple


class BenchConfig(NamedTuple):
    """Configuration for a single benchmark run."""
    lanes: int
    iterations: int


class BenchResult(NamedTuple):
    """Results from a benchmark run."""
    config: BenchConfig
    witness_time_ms: float
    proof_time_ms: float
    verify_time_ms: float


# Benchmark configurations to run
BENCH_CONFIGS = [
    # 1 parallel f, iterations from 2^13 to 2^17
    BenchConfig(lanes=1, iterations=2**13),
    BenchConfig(lanes=1, iterations=2**14),
    BenchConfig(lanes=1, iterations=2**15),
    BenchConfig(lanes=1, iterations=2**16),
    BenchConfig(lanes=1, iterations=2**17),
    # 4 parallel f, iterations from 2^11 to 2^15
    BenchConfig(lanes=4, iterations=2**11),
    BenchConfig(lanes=4, iterations=2**12),
    BenchConfig(lanes=4, iterations=2**13),
    BenchConfig(lanes=4, iterations=2**14),
    BenchConfig(lanes=4, iterations=2**15),
    # 16 parallel f, iterations from 2^9 to 2^13
    BenchConfig(lanes=16, iterations=2**9),
    BenchConfig(lanes=16, iterations=2**10),
    BenchConfig(lanes=16, iterations=2**11),
    BenchConfig(lanes=16, iterations=2**12),
    BenchConfig(lanes=16, iterations=2**13),
]


def get_workspace_root() -> Path:
    """Get the workspace root directory."""
    script_dir = Path(__file__).parent
    return script_dir.parent


def run_benchmark(config: BenchConfig) -> bool:
    """Run a single benchmark configuration."""
    workspace = get_workspace_root()
    env = os.environ.copy()
    env["ITERATIONS"] = str(config.iterations)
    env["LANES"] = str(config.lanes)
    env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -C target-cpu=native"
    
    print(f"Running benchmark: lanes={config.lanes}, iterations={config.iterations}")
    
    result = subprocess.run(
        ["cargo", "bench", "-p", "binius-examples", "--bench", "iterated_f", "--", "--noplot"],
        cwd=workspace,
        env=env,
        capture_output=True,
        text=True,
    )
    
    if result.returncode != 0:
        print(f"Benchmark failed: {result.stderr}", file=sys.stderr)
        return False
    
    return True


def find_benchmark_dir(workspace: Path, phase: str, config: BenchConfig) -> Path | None:
    """Find the criterion benchmark directory for a given configuration."""
    criterion_dir = workspace / "target" / "criterion" / f"iterated_f_{phase}"
    
    if not criterion_dir.exists():
        return None
    
    # Look for matching benchmark directory
    if config.lanes == 1:
        prefix = f"iterations_{config.iterations}_"
    else:
        prefix = f"iterations_{config.iterations}_lanes_{config.lanes}_"
    
    for entry in criterion_dir.iterdir():
        if entry.is_dir() and entry.name.startswith(prefix):
            return entry
    
    return None


def read_benchmark_time(bench_dir: Path) -> float | None:
    """Read the mean time in milliseconds from a benchmark directory."""
    estimates_file = bench_dir / "new" / "estimates.json"
    
    if not estimates_file.exists():
        return None
    
    try:
        with open(estimates_file) as f:
            data = json.load(f)
        # Time is in nanoseconds, convert to milliseconds
        return data["mean"]["point_estimate"] / 1_000_000
    except (json.JSONDecodeError, KeyError) as e:
        print(f"Error reading {estimates_file}: {e}", file=sys.stderr)
        return None


def collect_results(workspace: Path, config: BenchConfig) -> BenchResult | None:
    """Collect benchmark results for a configuration."""
    phases = ["witness_generation", "proof_generation", "proof_verification"]
    times = []
    
    for phase in phases:
        bench_dir = find_benchmark_dir(workspace, phase, config)
        if bench_dir is None:
            print(f"Warning: No benchmark dir found for {phase} with config {config}")
            return None
        
        time_ms = read_benchmark_time(bench_dir)
        if time_ms is None:
            return None
        times.append(time_ms)
    
    return BenchResult(
        config=config,
        witness_time_ms=times[0],
        proof_time_ms=times[1],
        verify_time_ms=times[2],
    )


def format_time(time_ms: float) -> str:
    """Format time in appropriate units."""
    if time_ms < 1:
        return f"{time_ms * 1000:.1f} µs"
    elif time_ms < 1000:
        return f"{time_ms:.1f} ms"
    else:
        return f"{time_ms / 1000:.2f} s"


def format_power_of_two(n: int) -> str:
    """Format a number as a power of two if possible."""
    import math
    if n > 0 and (n & (n - 1)) == 0:
        exp = int(math.log2(n))
        return f"$2^{{{exp}}}$"
    return str(n)


def generate_latex_table(results: list[BenchResult]) -> str:
    """Generate a LaTeX table from benchmark results."""
    lines = [
        r"\begin{table}[htbp]",
        r"\centering",
        r"\caption{Iterated f benchmark results}",
        r"\label{tab:iterated-f-benchmark}",
        r"\begin{tabular}{rrrrrr}",
        r"\toprule",
        r"Lanes & Iterations & Total Ops & Witness Gen & Proof Gen & Verification \\",
        r"\midrule",
    ]
    
    current_lanes = None
    for result in results:
        lanes = result.config.lanes
        iters = result.config.iterations
        total_ops = lanes * iters
        
        # Add horizontal line between lane groups
        if current_lanes is not None and lanes != current_lanes:
            lines.append(r"\midrule")
        current_lanes = lanes
        
        lines.append(
            f"{lanes} & {format_power_of_two(iters)} & {format_power_of_two(total_ops)} & "
            f"{format_time(result.witness_time_ms)} & "
            f"{format_time(result.proof_time_ms)} & "
            f"{format_time(result.verify_time_ms)} \\\\"
        )
    
    lines.extend([
        r"\bottomrule",
        r"\end{tabular}",
        r"\end{table}",
    ])
    
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark iterated_f circuit and generate LaTeX table"
    )
    parser.add_argument(
        "--output", "-o",
        type=Path,
        default=None,
        help="Output file for LaTeX table (default: stdout)",
    )
    parser.add_argument(
        "--skip-run",
        action="store_true",
        help="Skip running benchmarks, only collect existing results",
    )
    args = parser.parse_args()
    
    workspace = get_workspace_root()
    
    # Run benchmarks (unless skipped)
    if not args.skip_run:
        print("Running benchmarks...")
        for config in BENCH_CONFIGS:
            if not run_benchmark(config):
                print(f"Failed to run benchmark for {config}", file=sys.stderr)
                return 1
    
    # Collect results
    print("\nCollecting results...")
    results = []
    for config in BENCH_CONFIGS:
        result = collect_results(workspace, config)
        if result:
            results.append(result)
            print(f"  {config.lanes} lanes, {config.iterations} iters: "
                  f"witness={format_time(result.witness_time_ms)}, "
                  f"proof={format_time(result.proof_time_ms)}, "
                  f"verify={format_time(result.verify_time_ms)}")
        else:
            print(f"  {config.lanes} lanes, {config.iterations} iters: NO DATA")
    
    if not results:
        print("No benchmark results found!", file=sys.stderr)
        return 1
    
    # Generate LaTeX table
    latex = generate_latex_table(results)
    
    if args.output:
        args.output.write_text(latex)
        print(f"\nLaTeX table written to {args.output}")
    else:
        print("\n" + "=" * 60)
        print("LaTeX Table:")
        print("=" * 60)
        print(latex)
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
