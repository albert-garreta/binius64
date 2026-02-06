#!/usr/bin/env bash
# Copyright 2026 The Binius Developers
# 
# Run the iterated_f benchmark experiment.
# This script is a convenience wrapper for running the benchmark suite.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default configuration
export RUSTFLAGS="${RUSTFLAGS:-} -C target-cpu=native"

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Run the iterated_f benchmark experiment for Binius64.

Options:
    -h, --help          Show this help message
    -q, --quick         Run a quick subset of benchmarks
    -o, --output FILE   Save LaTeX table to FILE
    --skip-run          Only collect existing results
    --single LANES ITERS Run a single configuration

Examples:
    $(basename "$0")                      # Run full benchmark suite
    $(basename "$0") --quick              # Run quick subset
    $(basename "$0") --single 4 8192      # Run 4 lanes, 8192 iterations
    $(basename "$0") -o results.tex       # Save results to file

EOF
}

run_single() {
    local lanes=$1
    local iterations=$2
    echo "Running: lanes=$lanes, iterations=$iterations"
    LANES=$lanes ITERATIONS=$iterations cargo bench -p binius-examples --bench iterated_f -- --noplot
}

run_quick() {
    echo "Running quick benchmark subset..."
    run_single 1 8192
    run_single 4 2048
    run_single 16 512
}

run_full() {
    echo "Running full benchmark suite..."
    python3 "$REPO_ROOT/tools/bench_iterated_f.py" "$@"
}

main() {
    cd "$REPO_ROOT"
    
    local output_args=()
    local skip_run=false
    local quick=false
    local single=false
    local single_lanes=""
    local single_iters=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -q|--quick)
                quick=true
                shift
                ;;
            -o|--output)
                output_args+=("--output" "$2")
                shift 2
                ;;
            --skip-run)
                skip_run=true
                shift
                ;;
            --single)
                single=true
                single_lanes=$2
                single_iters=$3
                shift 3
                ;;
            *)
                echo "Unknown option: $1" >&2
                usage
                exit 1
                ;;
        esac
    done
    
    echo "============================================"
    echo "Iterated f Benchmark Experiment"
    echo "============================================"
    echo "Repository: $REPO_ROOT"
    echo "RUSTFLAGS: $RUSTFLAGS"
    echo ""
    
    if $single; then
        run_single "$single_lanes" "$single_iters"
    elif $quick; then
        run_quick
    else
        if $skip_run; then
            run_full --skip-run "${output_args[@]}"
        else
            run_full "${output_args[@]}"
        fi
    fi
    
    echo ""
    echo "Done. Results are in target/criterion/iterated_f_*/"
}

main "$@"
