.PHONY: all build test bench fmt clippy clean check example p2p scripts

all: fmt clippy test

build:
	cargo build

build-bin:
	cargo build --bin kubo-rs

test:
	cargo test

bench:
	cargo bench

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

check: fmt clippy test

clean:
	cargo clean
	cd kubo-sys/ffi && go clean

example:
	cargo run --example basic

p2p:
	cargo run --example p2p

scripts:
	@echo "Run one of:"
	@echo "  ./scripts/test.sh      (Unix/macOS)"
	@echo "  ./scripts/test.ps1     (Windows PowerShell)"
	@echo "  python3 ./scripts/test.py  (cross-platform)"
