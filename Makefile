.PHONY: check fmt clippy test build run help

# Default target
help:
	@echo "Available commands:"
	@echo "  make fmt      - Run cargo fmt"
	@echo "  make clippy   - Run cargo clippy"
	@echo "  make test     - Run cargo test"
	@echo "  make check    - Run fmt, clippy, and test (Pre-commit check)"
	@echo "  make build    - Build the project"
	@echo "  make run      - Run the project"

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test

check: fmt clippy test

build:
	cargo build

run:
	cargo run
