.DEFAULT_GOAL := help

.PHONY: all build build-bin build-go build-ffi test test-cli test-ffi test-ffi-c test-ffi-rust test-all bench fmt clippy clean clean-all check example p2p scripts cross-test run-wasm-dashboard build-wasm-dashboard-release website run-website build-website-release wasm-p2p run-wasm-p2p build-wasm-p2p-release wasm-hybrid-wasi run-wasm-hybrid-wasi-cli

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
	cargo clippy --all-targets

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

fix-wasm-bindgen:
	@rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
	@which trunk >/dev/null 2>&1 || cargo install trunk
	@# Ensure wasm-bindgen-cli matches the crate version (0.2.128)
	@cargo install -f wasm-bindgen-cli --version 0.2.128 2>/dev/null || true
	@# Clear stale trunk cache if binary version mismatches
	@rm -rf ~/Library/Caches/dev.trunkrs.trunk/wasm-bindgen-* 2>/dev/null || true
	@echo "wasm-bindgen-cli 0.2.128 ready"

wasm-dashboard: fix-wasm-bindgen
	./scripts/wasm-dashboard.sh --build-only 2>/dev/null || ./scripts/wasm-dashboard.sh

run-wasm-dashboard: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	./scripts/wasm-dashboard.sh

build-wasm-dashboard-release: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	cd examples/wasm-dashboard && env -u NO_COLOR trunk build --public-url /kubo-rs/

website: fix-wasm-bindgen
	./scripts/website.sh --build-only 2>/dev/null || ./scripts/website.sh

run-website: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	./scripts/website.sh

build-website-release: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	cd examples/website && env -u NO_COLOR trunk build --public-url /kubo-rs/

wasm-p2p: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	cd examples/wasm-p2p && env -u NO_COLOR trunk serve --port 8084

run-wasm-p2p: wasm-p2p

build-wasm-p2p-release: fix-wasm-bindgen
	pkill -f "trunk serve" 2>/dev/null || true
	cd examples/wasm-p2p && env -u NO_COLOR trunk build --public-url /kubo-rs/

install-wasi-sdk:
	@WSI_HOME="$(HOME)/.wasi-sdk"; \
	WSI="$${WASI_SDK_PATH:-$$WSI_HOME}"; \
	if [ ! -f "$$WSI/bin/clang" ]; then \
	  mkdir -p "$$WSI_HOME"; \
	  ARCH="$$(uname -m)"; \
	  case "$$(uname -s)" in \
	    Darwin) OS=macos ;; \
	    Linux)  OS=linux ;; \
	    MINGW*|MSYS*|CYGWIN*) OS=windows ;; \
	    *) OS=linux ;; \
	  esac; \
	  if [ "$$ARCH" = "arm64" ]; then ARCH=arm64; fi; \
	  if [ "$$ARCH" = "x86_64" ]; then ARCH=x86_64; fi; \
	  URL="https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-34/wasi-sdk-34.0-$$ARCH-$$OS.tar.gz"; \
	  echo "Downloading wasi-sdk for $$ARCH-$$OS → $$WSI_HOME ..."; \
	  curl -fsSL -o /tmp/wasi-sdk.tar.gz "$$URL" || { echo "Download failed. Install manually from https://github.com/WebAssembly/wasi-sdk/releases"; exit 1; }; \
	  echo "Extracting wasi-sdk ..."; \
	  tar -xzf /tmp/wasi-sdk.tar.gz -C "$$WSI_HOME" --strip-components=1; \
	  rm -f /tmp/wasi-sdk.tar.gz; \
	  echo "wasi-sdk installed to $$WSI_HOME"; \
	  echo "Add to your shell profile: export WASI_SDK_PATH=$$WSI_HOME"; \
	fi

wasm-hybrid-wasi: install-wasi-sdk
	@WSI="$${WASI_SDK_PATH:-$(HOME)/.wasi-sdk}"; \
	if [ ! -f "$$WSI/bin/clang" ]; then \
	  echo "ERROR: wasi-sdk not found at $$WSI"; exit 1; \
	fi; \
	rustup target list --installed | grep -q wasm32-wasip1 || rustup target add wasm32-wasip1; \
	cd examples/wasm-hybrid-wasi && \
	  CC="$$WSI/bin/clang" \
	  cargo build --target wasm32-wasip1 --release

install-wasmtime:
	@which wasmtime >/dev/null 2>&1 || { \
	  echo "Installing wasmtime ..."; \
	  curl https://wasmtime.dev/install.sh -sSf | bash; \
	}
	@export PATH="$(HOME)/.wasmtime/bin:$$PATH"; \
	which wasmtime >/dev/null 2>&1 || { echo "ERROR: wasmtime install failed"; exit 1; }

run-wasm-hybrid-wasi-cli: wasm-hybrid-wasi install-wasmtime
	@export PATH="$(HOME)/.wasmtime/bin:$$PATH"; \
	TGT="$$(cd examples/wasm-hybrid-wasi && cargo metadata --format-version 1 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"; \
	wasmtime "$${TGT}/wasm32-wasip1/release/wasm-hybrid-wasi.wasm"

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
	@echo "  fix-wasm-bindgen          - Install matching wasm-bindgen-cli and clear stale trunk cache"
	@echo "  wasm-dashboard            - Build the WASM dashboard example"
	@echo "  run-wasm-dashboard        - Build and serve the WASM dashboard (auto-picks free port from 8080)"
	@echo "  build-wasm-dashboard-release - Build WASM dashboard for GitHub Pages deployment"
	@echo "  website                   - Build the website example"
	@echo "  run-website               - Build and serve the website (auto-picks free port from 8082)"
	@echo "  build-website-release     - Build website for GitHub Pages deployment"
	@echo "  wasm-p2p                  - Build and serve the WASM libp2p WebRTC example (port 8084)"
	@echo "  run-wasm-p2p              - Alias for wasm-p2p"
	@echo "  build-wasm-p2p-release    - Build WASM libp2p example for GitHub Pages deployment"
	@echo "  wasm-hybrid-wasi          - Build wasm-hybrid-wasi (requires wasi-sdk)"
	@echo "  run-wasm-hybrid-wasi-cli  - Run wasm-hybrid-wasi with wasmtime"
	@echo "  cross-test     - Run cross-language alignment tests"
	@echo "  scripts        - Show available test scripts"
