# kubo-rs FFI

This document describes the Rust ↔ Go FFI bridge that binds the `kubo-rs` crate to the Kubo IPFS implementation.

## Overview

The FFI layer consists of three parts:

1. **Go CGo library** (`go/ffi/`) — exports C symbols and manages Kubo node lifecycle.
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
go/ffi/kubo.go  (CGO exports, node registry, CoreAPI wrappers)
      │
      ▼
  Kubo Go code    (core.NewNode, coreapi.NewCoreAPI, fsrepo, etc.)
```

## Go CGo Library

Located in `go/ffi/`. It is a separate Go module that uses `package main` with `//export` directives so it can build as a `c-archive`.

### Why a separate module?

The root `go/kubo-sys/` module is `package main` (the `ipfs` binary). The `ffi/` module isolates the CGO exports and imports Kubo via `replace github.com/ipfs/kubo => ..`.

### Exported C Functions

| Group | Exports |
|-------|---------|
| Shared utilities | `kubo_ffi_last_error`, `kubo_ffi_free_string`, `kubo_ffi_free_buffer` |
| Kubo node and content | `kubo_version`, `kubo_init_repo`, `kubo_node_*`, `kubo_unixfs_*`, `kubo_block_*`, `kubo_swarm_*`, `kubo_pin_*`, `kubo_dht_*`, `kubo_name_*`, `kubo_key_*`, `kubo_dag_*`, `kubo_mfs_*` |
| Standalone libp2p | `kubo_libp2p_host_*` |
| Nostr | `kubo_nostr_*` |
| Git | `kubo_git_*` |

The complete Rust declarations are in `src/ffi.rs`; their Go implementations
are the `//export` functions in `go/ffi/`. The public Rust API wraps these
operations in `Node`, `Host`, `Repository`, and top-level Nostr functions.

Functions returning status codes use `0` for success and `-1` for failure;
handle-returning functions use `0` for failure. Details are available via
`kubo_ffi_last_error()`.

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
2. Reads the `go` directive from `go/kubo-sys/go.mod` and sets `GOTOOLCHAIN` to that version. This ensures reproducible builds even when the host has a newer Go installed.
3. Maps the Rust `TARGET` triple to `GOOS` / `GOARCH`.
4. Runs `go build -buildmode=c-archive` in `go/ffi/`.
5. Emits Cargo link instructions for the static archive and required system libraries:
   - Unix: `pthread`, `dl`
   - macOS: `Security`, `CoreFoundation`, `resolv`

### Cross-compilation

The build script supports same-OS cross-compilation (e.g., `x86_64-apple-darwin` → `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` → `x86_64-pc-windows-gnu`). Full cross-OS compilation requires a matching C cross-toolchain and is not supported automatically.

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
    InvalidNostrUrl,
    InvalidNostrEvent,
    NoRelayConnection,
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
| `node.dag_put(data, input_codec, store_codec)` / `node.dag_get(cid, output_codec)` | Writes and reads an IPLD DAG node. |
| `node.mfs_*` | Provides MFS list/read/write/mkdir/remove/flush/stat operations. |
| `node.stop()` | Shuts the node down and consumes the handle. |

`mfs_write` creates a file when the path does not yet exist (its parent
directory must already exist) and replaces the contents when the path is an
existing file.

### `kubo_rs::HybridNode`

A unified handle that coordinates an IPFS `Node` and a libp2p `Host`. This is the recommended entry point for hybrid-protocol apps that need both content-addressing and peer-to-peer messaging.

```rust
use kubo_rs::HybridNode;

let node = HybridNode::start("/tmp/ipfs-repo", true)?;
let cid = node.ipfs.add_bytes(b"hello")?;

// Publish a signed Nostr event to GossipSub
let sk = kubo_rs::nostr_generate_key()?;
let event = kubo_rs::nostr_event_sign(&sk, "hello p2p", 1)?;
node.broadcast_event(&event, None, None)?;

// Drain events from both relay and gossip
let events = node.drain_events(None)?;

// Or publish a file directly: adds to IPFS, builds NIP-94 event, signs, broadcasts
let cid = node.publish_file("./photo.png", &sk, None, None)?;

node.stop()?;
```

| Method | Description |
|--------|-------------|
| `HybridNode::start(path, online)` | Starts both IPFS node and libp2p host. Auto-initialises repo if missing. |
| `node.broadcast_event(event, relay, topic)` | Verifies event, publishes to relay (if given) and GossipSub. |
| `node.drain_events(relay_sub)` | Merges events from relay subscription and GossipSub; deduplicates by event `id`. |
| `node.publish_file(path, sk, relay, topic)` | Adds file to IPFS, constructs NIP-94 event, signs, broadcasts. Returns CID. |
| `node.stop()` | Shuts down both subsystems and consumes the handle. |

## Memory Safety

- All C strings allocated by Go are freed by calling `kubo_ffi_free_string` from Rust.
- All byte buffers allocated by data-returning functions are freed by calling `kubo_ffi_free_buffer` from Rust.
- The `Node` type implements `Drop` so that the Go node is stopped even if the Rust consumer forgets to call `stop()`.
- Null bytes in paths and strings are rejected early with `Error::InvalidPath` / `Error::InvalidString`.

## Testing

### Quick test

```bash
cargo test
```

### Full validation (includes cross-language alignment)

```bash
make check          # fmt + clippy + rust tests + FFI tests
make test-all       # rust tests + C/Rust FFI tests
./scripts/cross-test.sh
```

### Test coverage

- Rust API tests cover node lifecycle, UnixFS and block roundtrips, pinning,
  swarm operations, libp2p lifecycle, Nostr signing, and basic Git repository
  operations. DHT/IPNS and public-relay tests are ignored unless their network
  prerequisites are available.
- C and raw Rust FFI runners exercise matching version/lifecycle, UnixFS,
  block, peer-connect, DAG, MFS create/update, and local Nostr key/event
  roundtrips.
- Coverage is not exhaustive: many individual Git, Nostr relay/NIP, DHT/IPNS,
  and libp2p exports do not yet have dedicated raw-FFI test cases.
- CID alignment: `add_bytes("hello world")` must produce
  `Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD` (CIDv0 dag-pb sha2-256),
  matching Kubo's default profile and `go/kubo-sys/test/cli/add_test.go`.
- **Drop behavior** — node stops automatically when handle is dropped

### Cross-language test runners

The FFI layer is validated from both sides:

| Runner | Language | Location | Purpose |
|--------|----------|----------|---------|
| `cargo test` | Rust (safe API) | `src/lib.rs`, `tests/cli.rs` | Rust consumer-facing tests |
| `testffi` | C (raw FFI) | `go/ffi/cmd/testffi/main.c` | Validate C exports from Go side |
| `testrust` | Rust (raw FFI) | `go/ffi/cmd/testrust/main.rs` | Validate C exports from Rust side |

Both `testffi` and `testrust` exercise matching raw-FFI flows with the same
vectors. This validates those selected exports across languages; it does not
replace per-export coverage for the full ABI.

### Temp directories

All tests use a project-local `./tmp/` directory instead of system temp (`/tmp` or `tempfile`). This avoids approval prompts and makes debugging easier. The directory is gitignored (`tmp/*`).

## CI

GitHub Actions workflows:

- **Rust FFI CI** (`.github/workflows/rust.yml`) — format, clippy, build, test on `ubuntu-latest`, `macos-latest`, `windows-latest`; `cargo-deny`; submodule verification; FFI tests
- **Cache Factory** (`.github/workflows/cache-factory.yml`) — produces shared Rust and Go module caches for all three OSes to speed up PR builds
- **macOS Dispatch** (`.github/workflows/macos-ci.yml`) — `workflow_dispatch` driven CI for various macOS runners (`macos-latest`, `macos-15`, `macos-14`, etc.); large/xlarge variants default to `false`

All workflows fetch submodules at full depth (`fetch-depth: 0`) so that commits on non-default branches are available.

## Adding New FFI Functions

1. Add the exported C function in `go/ffi/ffi.go`.
2. Run `go mod tidy` in `go/ffi/` if new imports are added.
3. Add the `extern "C"` declaration in `src/ffi.rs`.
4. Add a safe wrapper in `src/lib.rs` (or extend `Node`).
5. Add a test in `src/lib.rs` under `#[cfg(test)]`.
6. Mirror the test in `go/ffi/cmd/testffi/main.c` and `go/ffi/cmd/testrust/main.rs`.
7. Run `cargo fmt`, `cargo clippy`, and `cargo test`.
8. Run `make check` or `./scripts/cross-test.sh` to verify alignment.

## Security & Stability Notes

The FFI layer is a thin wrapper around Kubo's internal APIs. It does not alter Kubo's default CID recipe, gateway behavior, or RPC API. All Kubo stability and user-agency rules documented in `go/kubo-sys/AGENTS.md` apply to changes made through this bridge.
