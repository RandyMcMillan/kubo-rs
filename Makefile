.DEFAULT_GOAL := help

.PHONY: all build build-bin build-go build-ffi test test-cli test-ffi test-ffi-c test-ffi-rust test-all bench fmt clippy clean clean-all check example p2p scripts cross-test run-wasm-dashboard build-wasm-dashboard-release

all: fmt clippy test

# Rust builds
build:
	cargo build

build-bin:
	cargo build --bin kubo-rs

build-release:
	cargo build --release

# Go builds (delegates to go/kubo-sys Makefile)
build-go:
	$(MAKE) -C go/kubo-sys build

# FFI archive build
build-ffi:
	cd go/ffi && go build -buildmode=c-archive -o ./tmp/libkubo_ffi.a .

# Testing
test:
	cargo test

test-cli:
	cargo test --test cli

test-ffi-c: build-ffi
	cd go/ffi && \
	if [ "$$(uname)" = "Darwin" ]; then \
		cc -o cmd/testffi/testffi cmd/testffi/main.c -I./tmp ./tmp/libkubo_ffi.a -lpthread -ldl -framework Security -framework CoreFoundation -lresolv; \
	else \
		cc -o cmd/testffi/testffi cmd/testffi/main.c -I./tmp ./tmp/libkubo_ffi.a -lpthread -ldl; \
	fi && \
	./cmd/testffi/testffi

test-ffi-rust: build-ffi
	cd go/ffi && \
	if [ "$$(uname)" = "Darwin" ]; then \
		rustc cmd/testrust/main.rs -L ./tmp -lkubo_ffi -o cmd/testrust/testrust \
			-C link-arg="-framework" -C link-arg="Security" \
			-C link-arg="-framework" -C link-arg="CoreFoundation" \
			-lresolv -lpthread -ldl; \
	else \
		rustc cmd/testrust/main.rs -L ./tmp -lkubo_ffi -o cmd/testrust/testrust -lpthread -ldl; \
	fi && \
	./cmd/testrust/testrust

test-ffi: test-ffi-c test-ffi-rust

test-all: test test-ffi

# Benchmarks
bench:
	cargo bench

# Formatting and linting
fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

check: fmt clippy test-all

# Cleanup
clean:
	cargo clean
	cd go/ffi && go clean
	cd go/ffi && rm -f cmd/testffi/testffi cmd/testrust/testrust
	cd go/ffi && rm -rf ./tmp

clean-all: clean
	$(MAKE) -C go/kubo-sys clean

# Examples
example:
	cargo run --example basic

p2p:
	cargo run --example p2p

dashboard:
	cargo run --example dashboard

wasm-dashboard:
	./scripts/wasm-dashboard.sh --build-only 2>/dev/null || ./scripts/wasm-dashboard.sh

run-wasm-dashboard:
	pkill -f "trunk serve" 2>/dev/null || true
	./scripts/wasm-dashboard.sh

build-wasm-dashboard-release:
	@rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
	@which trunk >/dev/null 2>&1 || cargo install trunk
	pkill -f "trunk serve" 2>/dev/null || true
	cd examples/wasm-dashboard && env -u NO_COLOR trunk build --public-url /kubo-rs/

# Cross-testing
scripts:
	@echo "Run one of:"
	@echo "  ./scripts/test.sh         (Unix/macOS)"
	@echo "  ./scripts/test.ps1        (Windows PowerShell)"
	@echo "  python3 ./scripts/test.py (cross-platform)"
	@echo "  ./scripts/cross-test.sh   (Rust + FFI alignment tests)"

cross-test:
	./scripts/cross-test.sh

# Help
help:
	@echo "Available targets:"
	@echo "  build          - Build the Rust library"
	@echo "  build-bin      - Build the kubo-rs CLI binary"
	@echo "  build-go       - Build the Go ipfs binary (via go/kubo-sys/Makefile)"
	@echo "  build-ffi      - Build the FFI C archive"
	@echo "  test           - Run all Rust tests"
	@echo "  test-cli       - Run CLI integration tests"
	@echo "  test-ffi-c     - Build and run C FFI tests"
	@echo "  test-ffi-rust  - Build and run Rust raw-FFI tests"
	@echo "  test-ffi       - Run both C and Rust FFI tests"
	@echo "  test-all       - Run Rust tests + FFI tests"
	@echo "  bench          - Run Criterion benchmarks"
	@echo "  fmt            - Format Rust code"
	@echo "  clippy         - Run Clippy lints"
	@echo "  check          - Run fmt + clippy + test-all"
	@echo "  clean          - Clean Rust and FFI build artifacts"
	@echo "  clean-all      - Clean everything including Go build"
	@echo "  example        - Run the basic example"
	@echo "  p2p            - Run the p2p example"
	@echo "  dashboard      - Run the ratatui TUI dashboard example"
	@echo "  wasm-dashboard            - Build the WASM dashboard example"
	@echo "  run-wasm-dashboard        - Build and serve the WASM dashboard (opens http://localhost:8080)"
	@echo "  build-wasm-dashboard-release - Build WASM dashboard for GitHub Pages deployment"
	@echo "  cross-test     - Run cross-language alignment tests"
	@echo "  scripts        - Show available test scripts"
