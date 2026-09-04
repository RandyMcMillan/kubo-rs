.PHONY: all build build-bin build-go test bench fmt clippy clean check example p2p scripts

all: fmt clippy test

# Rust builds
build:
	cargo build

build-bin:
	cargo build --bin kubo-rs

build-release:
	cargo build --release

# Go builds (delegates to kubo-sys Makefile)
build-go:
	$(MAKE) -C kubo-sys build

# Testing
test:
	cargo test

test-cli:
	cargo test --test cli

bench:
	cargo bench

# Formatting and linting
fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

check: fmt clippy test

# Cleanup
clean:
	cargo clean
	cd kubo-sys/ffi && go clean

clean-all: clean
	$(MAKE) -C kubo-sys clean

# Examples
example:
	cargo run --example basic

p2p:
	cargo run --example p2p

# Scripts
scripts:
	@echo "Run one of:"
	@echo "  ./scripts/test.sh      (Unix/macOS)"
	@echo "  ./scripts/test.ps1     (Windows PowerShell)"
	@echo "  python3 ./scripts/test.py  (cross-platform)"

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the Rust library"
	@echo "  build-bin    - Build the kubo-rs CLI binary"
	@echo "  build-go     - Build the Go ipfs binary (via kubo-sys/Makefile)"
	@echo "  test         - Run all Rust tests"
	@echo "  test-cli     - Run CLI integration tests"
	@echo "  bench        - Run Criterion benchmarks"
	@echo "  fmt          - Format Rust code"
	@echo "  clippy       - Run Clippy lints"
	@echo "  check        - Run fmt + clippy + test"
	@echo "  clean        - Clean Rust and FFI build artifacts"
	@echo "  clean-all    - Clean everything including Go build"
	@echo "  example      - Run the basic example"
	@echo "  p2p          - Run the p2p example"
	@echo "  scripts      - Show available test scripts"
