#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Format check ==="
cargo fmt -- --check

echo "=== Clippy ==="
cargo clippy --all-targets -- -D warnings

echo "=== Build ==="
cargo build --bin kubo-rs

echo "=== Build examples ==="
cargo build --examples

echo "=== Test ==="
cargo test

echo "=== Doc check ==="
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

echo "=== All checks passed ==="
echo
echo "For cross-language FFI alignment tests, run:"
echo "  make test-ffi"
echo "  ./scripts/cross-test.sh"
