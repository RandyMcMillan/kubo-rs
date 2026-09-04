# kubo-rs

Rust bindings for [Kubo](https://github.com/ipfs/kubo), the reference implementation of IPFS in Go.

## Status

Early development. The crate currently provides a minimal but functional FFI bridge that lets Rust code:

- Initialize IPFS repositories
- Start and stop Kubo nodes (online or offline)
- Retrieve node identity (peer ID, listening addresses)
- Add and retrieve UnixFS content
- Connect to peers on the libp2p network

## Prerequisites

- **Rust** — latest stable toolchain (edition 2024)
- **Go** — 1.26.5 or later (the build script auto-downloads the correct toolchain via `GOTOOLCHAIN`)
- **Git** — for submodule management

## Building

Clone with submodules:

```bash
git clone --recursive https://github.com/RandyMcMillan/kubo-rs.git
# or, if already cloned:
git submodule update --init --recursive
```

Build the crate:

```bash
cargo build
```

The first build compiles the Kubo Go codebase into a static C archive via CGO. This is slow (minutes) but cached afterwards.

## Usage

```rust
use kubo_rs::{init_repo, Node};

// Initialize a repo
init_repo("/tmp/ipfs-repo")?;

// Start an offline node
let node = Node::start("/tmp/ipfs-repo", false)?;
println!("peer id: {}", node.peer_id()?);

// Add some data
let cid = node.add_bytes(b"hello world")?;
println!("CID: {}", cid);

// Retrieve it
let data = node.cat(&cid)?;
assert_eq!(data, b"hello world");

// Shut down
node.stop()?;
```

See [`FFI.md`](FFI.md) for architecture details and the full API surface.

## Testing

```bash
cargo test
```

Tests cover version retrieval, repo initialization, node lifecycle, UnixFS add/cat roundtrips, Block API, drop behavior, and peer-to-peer data exchange between two online nodes.

## CLI Binary

A `kubo-rs` CLI binary is included:

```bash
# Build the binary
cargo build --bin kubo-rs

# Initialize a repo
cargo run --bin kubo-rs -- init ~/.ipfs

# Get peer ID
cargo run --bin kubo-rs -- peer-id ~/.ipfs

# Add a file
cargo run --bin kubo-rs -- add --repo ~/.ipfs ./file.txt

# Retrieve by CID
cargo run --bin kubo-rs -- cat --repo ~/.ipfs Qm...

# Block API
cargo run --bin kubo-rs -- block-put --repo ~/.ipfs ./raw.bin
cargo run --bin kubo-rs -- block-stat --repo ~/.ipfs Qm...
```

## Benchmarks

```bash
cargo bench
```

Includes Criterion benchmarks for `add_bytes` with 1 MiB and 11 B payloads.

## Cross-platform Testing

Use the provided scripts for a full check:

```bash
# Unix / macOS
./scripts/test.sh

# Windows PowerShell
.\scripts\test.ps1

# Python (cross-platform)
python3 ./scripts/test.py
```

Or run via Make:

```bash
make check        # fmt + clippy + test
make build-bin    # build CLI binary
make test-cli     # run CLI integration tests
make bench        # run benchmarks
```

## CI

GitHub Actions builds and tests on:

- Ubuntu Latest
- macOS Latest
- Windows Latest

## Project Structure

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Safe public API (`Node`, `init_repo`, `version`) |
| `src/main.rs` | `kubo-rs` CLI binary |
| `src/ffi.rs` | Unsafe `extern "C"` bindings |
| `src/error.rs` | `Error` enum |
| `build.rs` | Compiles `go/kubo-sys/ffi/` via CGO |
| `go/kubo-sys/` | Kubo git submodule (Go) |
| `go/kubo-sys/ffi/` | CGo library exporting C symbols |
| `scripts/` | Cross-platform test scripts |
| `examples/` | Usage examples |
| `benches/` | Criterion benchmarks |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
