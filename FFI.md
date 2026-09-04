# kubo-rs FFI

This document describes the Rust ↔ Go FFI bridge that binds the `kubo-rs` crate to the Kubo IPFS implementation.

## Overview

The FFI layer consists of three parts:

1. **Go CGo library** (`kubo-sys/ffi/`) — exports C symbols and manages Kubo node lifecycle.
2. **Rust build script** (`build.rs`) — compiles the Go code into a static archive and links it.
3. **Rust bindings** (`src/ffi.rs`, `src/lib.rs`) — unsafe FFI declarations and safe public API.

## Architecture

```
Rust consumer
      │
      ▼
  src/lib.rs      (safe API: Node, init_repo, version, Error)
      │
      ▼
  src/ffi.rs      (unsafe extern "C" bindings)
      │
      ▼
  libkubo_ffi.a   (Go static archive built by build.rs)
      │
      ▼
kubo-sys/ffi/ffi.go  (CGO exports, node registry, CoreAPI wrappers)
      │
      ▼
  Kubo Go code    (core.NewNode, coreapi.NewCoreAPI, fsrepo, etc.)
```

## Go CGo Library

Located in `kubo-sys/ffi/`. It is a separate Go module (not `package main`) so it can build as a `c-archive`.

### Why a separate module?

The root `kubo-sys/` module is `package main` (the `ipfs` binary). CGo requires a non-main package to produce a static C archive. The `ffi/` module imports Kubo via `replace github.com/ipfs/kubo => ..`.

### Exported C Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `kubo_version` | `() -> *char` | Returns the Kubo version string |
| `kubo_last_error` | `() -> *char` | Returns the last error message (if any) |
| `kubo_free_string` | `(*char)` | Frees a C string allocated by Go |
| `kubo_init_repo` | `(path) -> int64` | Initializes a new IPFS repo |
| `kubo_node_start` | `(path, online) -> uint64` | Starts a node; returns an opaque handle |
| `kubo_node_stop` | `(handle) -> int64` | Stops the node |
| `kubo_node_peer_id` | `(handle) -> *char` | Returns the node's peer ID |
| `kubo_node_listening_addrs` | `(handle) -> *char` | Returns newline-separated listen addrs |
| `kubo_node_connect` | `(handle, addr) -> int64` | Connects to a peer by multiaddr |
| `kubo_unixfs_add_bytes` | `(handle, data, len) -> *char` | Adds bytes to IPFS; returns CID |
| `kubo_unixfs_cat` | `(handle, cid, out, out_len) -> int64` | Retrieves UnixFS content by CID |
| `kubo_block_put` | `(handle, data, len) -> *char` | Stores raw block; returns CID |
| `kubo_block_get` | `(handle, cid, out, out_len) -> int64` | Retrieves raw block data |
| `kubo_block_stat` | `(handle, cid) -> int64` | Returns block size (-1 on error) |
| `kubo_free_buffer` | `(*uint8_t)` | Frees a buffer allocated by Go |

All functions that can fail return an `int64` error code (`0` = success, `-1` = error). Details are available via `kubo_last_error()`.

### Node Registry

Go pointers cannot cross the C boundary safely (CGO pointer rules). Instead, `kubo_node_start` returns an opaque `uint64` handle that indexes into a global `map[uint64]*nodeHandle` protected by a `sync.RWMutex`. The registry stores:

- `ctx` / `cancel` — the node's lifetime context
- `node` — `*core.IpfsNode`
- `api` — `coreiface.CoreAPI`

### Plugin Loading

Kubo requires plugins to be loaded once before any repo or node operation. The FFI layer uses `sync.Once` to call `loader.NewPluginLoader`, `Initialize`, and `Inject` on first use.

## Rust Build Script

`build.rs` performs the following steps:

1. Verifies the `kubo-sys` submodule is present.
2. Reads the `go` directive from `kubo-sys/go.mod` and sets `GOTOOLCHAIN` to that version. This ensures reproducible builds even when the host has a newer Go installed.
3. Maps the Rust `TARGET` triple to `GOOS` / `GOARCH`.
4. Runs `go build -buildmode=c-archive` in `kubo-sys/ffi/`.
5. Emits Cargo link instructions for the static archive and required system libraries:
   - Unix: `pthread`, `dl`
   - macOS: `Security`, `CoreFoundation`, `resolv`

### Cross-compilation

The build script currently supports host builds only. CGO cross-compilation requires a matching C cross-toolchain, which is beyond the scope of the current script.

## Rust Safe API

### `kubo_rs::version()`

Returns the Kubo version string (e.g., `"0.44.0-dev"`).

### `kubo_rs::init_repo(path)`

Initializes a new IPFS repository at the given filesystem path.

### `kubo_rs::Error`

```rust
pub enum Error {
    InvalidPath,
    InvalidString,
    InvalidHandle,
    Go(String),
}
```

Implements `std::error::Error` and `Display`.

### `kubo_rs::Node`

An owned handle to a running Kubo node. When dropped, the node is shut down.

```rust
use kubo_rs::{init_repo, Node};

init_repo("/tmp/ipfs-repo")?;
let node = Node::start("/tmp/ipfs-repo", true)?;
let peer_id = node.peer_id()?;
let addrs = node.listening_addrs()?;
node.stop()?;
```

#### Methods

| Method | Description |
|--------|-------------|
| `Node::start(path, online)` | Starts a node. `online` controls libp2p networking. |
| `node.peer_id()` | Returns the node's peer ID. |
| `node.listening_addrs()` | Returns the node's listening multiaddresses. |
| `node.connect(addr)` | Dials a peer. Address must include peer ID: `/ip4/…/p2p/Qm…` |
| `node.add_bytes(data)` | Adds bytes to IPFS; returns the CID. |
| `node.cat(cid)` | Retrieves UnixFS content by CID or `/ipfs/…` path. |
| `node.block_put(data)` | Stores a raw block; returns the CID. |
| `node.block_get(cid)` | Retrieves raw block data by CID. |
| `node.block_stat(cid)` | Returns the size of a block by CID. |
| `node.stop()` | Shuts the node down and consumes the handle. |

## Memory Safety

- All C strings allocated by Go are freed by calling `kubo_free_string` from Rust.
- All byte buffers allocated by `kubo_unixfs_cat`, `kubo_block_get`, etc. are freed by calling `kubo_free_buffer` from Rust.
- The `Node` type implements `Drop` so that the Go node is stopped even if the Rust consumer forgets to call `stop()`.
- Null bytes in paths and strings are rejected early with `Error::InvalidPath` / `Error::InvalidString`.

## Testing

Run the test suite with:

```bash
cargo test
```

Tests cover:

- Version string retrieval
- Repo initialization and node start/stop
- UnixFS add/cat roundtrip
- Empty content roundtrip
- Drop behavior (node stops automatically)
- Listening addresses on online nodes
- Invalid path rejection
- Peer-to-peer data exchange between two online nodes
- Block API put/get/stat roundtrip

## Adding New FFI Functions

1. Add the exported C function in `kubo-sys/ffi/ffi.go`.
2. Run `go mod tidy` in `kubo-sys/ffi/` if new imports are added.
3. Add the `extern "C"` declaration in `src/ffi.rs`.
4. Add a safe wrapper in `src/lib.rs` (or extend `Node`).
5. Add a test in `src/lib.rs` under `#[cfg(test)]`.
6. Run `cargo fmt`, `cargo clippy`, and `cargo test`.

## Security & Stability Notes

The FFI layer is a thin wrapper around Kubo's internal APIs. It does not alter Kubo's default CID recipe, gateway behavior, or RPC API. All Kubo stability and user-agency rules documented in `kubo-sys/AGENTS.md` apply to changes made through this bridge.
