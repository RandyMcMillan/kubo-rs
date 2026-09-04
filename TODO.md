# kubo-rs Development TODO

## Active

- [ ] Add swarm_peers FFI function to ffi.go
- [ ] Add swarm_peers Rust binding and wrapper
- [ ] Rewrite dashboard.rs with rich ratatui UI (inspired by ipfs-desktop)
- [ ] Build and test all changes
- [ ] Commit dashboard enhancements

## Recently Completed

- [x] Fix release.yml invalid action versions (checkout/upload/download)
- [x] Fix cache-factory go-cache /dev/null on Windows
- [x] Fix cargo fmt in src/main.rs
- [x] Add CLI tests for block, config, p2p commands
- [x] Fix FFI.md inaccuracies (package main, cross-compilation)
- [x] Create native ratatui dashboard example
- [x] Create ratzilla WASM dashboard example
- [x] Update Makefile with new example targets
- [x] Make cargo fmt warn-only in all CI workflows

## Alignment Status

| Layer | Status |
|---|---|
| Go FFI exports (15 functions) | Complete |
| Safe Rust wrappers | Complete |
| CLI commands | Complete (ipfs 10, p2p 3, nostr 2) |
| Rust lib tests | 10 tests passing |
| CLI tests | 7 tests passing |
| C FFI tests | 8 tests passing |
| Raw Rust FFI tests | 8 tests passing |
| Native examples | basic, p2p, dashboard |
| WASM example | wasm-dashboard (HTTP API) |
