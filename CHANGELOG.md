# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial Rust FFI bridge to Kubo (Go IPFS implementation).
- `kubo-sys/ffi/` — CGo library exporting C symbols for:
  - Repo initialization (`kubo_init_repo`)
  - Node lifecycle (`kubo_node_start`, `kubo_node_stop`)
  - Peer identity (`kubo_node_peer_id`, `kubo_node_listening_addrs`)
  - Swarm connectivity (`kubo_node_connect`)
  - UnixFS add/cat (`kubo_unixfs_add_bytes`, `kubo_unixfs_cat`)
  - Block API (`kubo_block_put`, `kubo_block_get`, `kubo_block_stat`)
- `build.rs` — compiles Go FFI to static archive with:
  - `GOTOOLCHAIN` pinning from `kubo-sys/go.mod`
  - Platform-specific linking (Unix, macOS, Windows)
- Safe Rust API in `src/lib.rs`:
  - `init_repo`, `version`
  - `Node` with `start`, `stop`, `peer_id`, `listening_addrs`, `connect`
  - `Node` with `add_bytes`, `cat`
  - `Node` with `block_put`, `block_get`, `block_stat`
- `kubo_rs::Error` enum implementing `std::error::Error`.
- Comprehensive test suite covering:
  - Version retrieval
  - Repo init and node start/stop
  - UnixFS add/cat roundtrip
  - Empty content roundtrip
  - Drop behavior
  - Online node listening addresses
  - Invalid path rejection
  - Peer-to-peer data exchange between two nodes
  - Block API put/get/stat roundtrip
- `examples/basic.rs` — demonstrates repo init, node start, add/cat.
- `README.md` and `FFI.md` documentation.
- GitHub Actions CI with OS matrix (Ubuntu, macOS, Windows).
