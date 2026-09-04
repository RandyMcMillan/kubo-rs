.PHONY: all build test fmt clippy clean check

all: fmt clippy test

build:
	cargo build

test:
	cargo test

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
