.PHONY: all build test bench fmt clippy clean check example p2p

all: fmt clippy test

build:
	cargo build

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
