$ErrorActionPreference = "Stop"
$PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "=== Format check ==="
cargo fmt -- --check

Write-Host "=== Clippy ==="
cargo clippy --all-targets -- -D warnings

Write-Host "=== Build ==="
cargo build --bin kubo-rs

Write-Host "=== Build examples ==="
cargo build --examples

Write-Host "=== Test ==="
cargo test

Write-Host "=== Doc check ==="
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --no-deps --document-private-items

Write-Host "=== All checks passed ==="
Write-Host ""
Write-Host "For cross-language FFI alignment tests, run:"
Write-Host "  make test-ffi"
Write-Host "  ./scripts/cross-test.sh"
