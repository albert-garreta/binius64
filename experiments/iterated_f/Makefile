# Iterated f Benchmark Experiment
# Copyright 2026 The Binius Developers

REPO_ROOT := $(shell cd ../.. && pwd)
RESULTS_DIR := $(REPO_ROOT)/target/criterion

.PHONY: all quick full single clean help

# Default: show help
help:
	@echo "Iterated f Benchmark Experiment"
	@echo ""
	@echo "Targets:"
	@echo "  make quick              Run quick subset of benchmarks"
	@echo "  make full               Run complete benchmark matrix"
	@echo "  make full OUTPUT=f.tex  Run benchmarks and save to file"
	@echo "  make single L=4 I=8192  Run single config (4 lanes, 8192 iters)"
	@echo "  make collect            Collect existing results only"
	@echo "  make clean              Remove benchmark results"
	@echo "  make help               Show this help"
	@echo ""
	@echo "Environment variables:"
	@echo "  RUSTFLAGS    Additional Rust compiler flags"
	@echo "  LOG_INV_RATE Log inverse rate for FRI (default: 1)"

# Quick benchmark (subset)
quick:
	@./run_experiment.sh --quick

# Full benchmark matrix
full:
ifdef OUTPUT
	@./run_experiment.sh --output $(OUTPUT)
else
	@./run_experiment.sh
endif

# Single configuration
single:
ifndef L
	$(error L (lanes) is required. Usage: make single L=4 I=8192)
endif
ifndef I
	$(error I (iterations) is required. Usage: make single L=4 I=8192)
endif
	@./run_experiment.sh --single $(L) $(I)

# Collect existing results without running benchmarks
collect:
ifdef OUTPUT
	@./run_experiment.sh --skip-run --output $(OUTPUT)
else
	@./run_experiment.sh --skip-run
endif

# Clean benchmark results
clean:
	rm -rf $(RESULTS_DIR)/iterated_f_*
	@echo "Removed iterated_f benchmark results"
