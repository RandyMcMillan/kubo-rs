# AI Agent Instructions for kubo-rs

This file provides instructions for AI coding agents working on the `kubo-rs` codebase.

## Quick Reference

| Task | Command |
|------|---------|
| Build Rust crate | `cargo build` |
| Build Go submodule | `cd kubo-sys && make build` |
| Test Rust crate | `cargo test` |
| Test Go submodule | `cd kubo-sys && make test` |
| Format Rust code | `cargo fmt` |
| Format Go code | `cd kubo-sys && go fmt ./...` |
| Update Go deps | `cd kubo-sys && make mod_tidy` |

## Project Overview

`kubo-rs` is a Rust crate that provides bindings and integration with [Kubo](https://github.com/ipfs/kubo), the reference implementation of IPFS in Go. The project is in early development: the root Rust crate is currently a minimal library, and the heavy lifting is provided by the `kubo-sys` git submodule, which contains the full Kubo Go codebase.

The goal of this project is to expose Kubo/IPFS functionality to Rust consumers through a native Rust API surface, while the underlying protocol implementation remains the Go-based Kubo node.

## Architecture

The project has a two-layer architecture:

| Layer | Location | Technology | Purpose |
|-------|----------|------------|---------|
| Rust crate | Root (`src/`, `Cargo.toml`) | Rust (edition 2024) | Public API, Rust-native types, and FFI/integration logic |
| Kubo submodule | `kubo-sys/` | Go (1.26.5) | Full IPFS node implementation (daemon, networking, blockstore, etc.) |

### Git Submodule

`kubo-sys/` is a git submodule pointing to `https://github.com/RandyMcMillan/kubo.git` at commit `329838acd` (based on Kubo v0.34.1). This submodule must be initialized and updated after cloning:

```bash
git submodule update --init --recursive
```

When making changes that affect both the Rust crate and the Go submodule, commit and push the submodule changes first, then update the parent repository's submodule pointer.

## Code Organization

### Root Rust Crate

| Path | Purpose |
|------|---------|
| `Cargo.toml` | Rust package manifest |
| `src/lib.rs` | Library entry point and public API |

The root crate currently has no external dependencies. As the project matures, dependencies on crates such as `serde`, `tokio`, or `libp2p` may be added here.

### Go Submodule (`kubo-sys/`)

The submodule contains the entire Kubo codebase. Key directories inside `kubo-sys/`:

| Directory | Purpose |
|-----------|---------|
| `cmd/ipfs/` | CLI entry point and `ipfs` binary |
| `core/` | Core IPFS node implementation |
| `core/commands/` | CLI command definitions |
| `core/coreapi/` | Go API implementation |
| `client/rpc/` | HTTP RPC client |
| `config/` | Configuration schema and defaults |
| `plugin/` | Plugin system |
| `repo/` | Repository management |
| `test/cli/` | Modern Go-based CLI integration tests |
| `test/sharness/` | Legacy shell-based integration tests |
| `docs/` | Documentation |

For detailed instructions on working inside the Go submodule, see [`kubo-sys/AGENTS.md`](kubo-sys/AGENTS.md). That file contains critical rules about backward compatibility, the RPC API, the HTTP Gateway, CID recipes, telemetry, and user agency that **must not be broken**.

## Building

### Prerequisites

- **Rust** - latest stable toolchain (edition 2024)
- **Go** - see `kubo-sys/go.mod` for the minimum required version (currently 1.26.5)
- **Git**
- **GNU Make** - for the Go build system
- **GCC** (optional) - required for CGO; build with `CGO_ENABLED=0` if unavailable

### Build the Rust Crate

```bash
cargo build
```

### Build the Go Submodule

Always run commands from the `kubo-sys/` directory or prefix them accordingly:

```bash
cd kubo-sys
make build        # builds the ipfs binary to cmd/ipfs/ipfs
make install      # installs to $GOPATH/bin
```

**Always build with `make build`, never `go build`.** The Makefile injects required `-ldflags` for version info.

## Testing

### Rust Tests

```bash
cargo test
```

### Go Submodule Tests

See [`kubo-sys/AGENTS.md`](kubo-sys/AGENTS.md) for the full test matrix. Key targets:

```bash
cd kubo-sys
make test_unit     # unit tests with coverage
make test_cli      # CLI integration tests (requires `make build` first)
make test          # full suite (slow)
```

Before running Go integration tests:

```bash
export PATH="$PWD/kubo-sys/cmd/ipfs:$PATH"
export IPFS_PATH="$(mktemp -d)"
```

## Code Style

### Rust

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Use idiomatic Rust 2024 edition features
- Prefer `?` for error propagation
- Document public APIs with `///` doc comments

### Go (inside `kubo-sys/`)

Follow the conventions in [`kubo-sys/AGENTS.md`](kubo-sys/AGENTS.md), which include:

- [Go Code Review Comments](https://go.dev/wiki/CodeReviewComments)
- [Google Go Style Decisions](https://google.github.io/styleguide/go/decisions)
- Run `go fmt ./...` after modifying Go files
- Wrap errors with `fmt.Errorf("context: %w", err)`
- Use `errors.Is` / `errors.As`, not string comparison
- Never use `panic` in library code

## Before Submitting

Run these steps before committing, pushing, or opening a PR:

1. `cargo fmt && cargo clippy` (Rust)
2. `cargo test` (Rust)
3. `cd kubo-sys && make mod_tidy` (if Go deps changed)
4. `cd kubo-sys && go fmt ./...` (if Go files changed)
5. `cd kubo-sys && make build` (if non-test Go files changed)
6. `cd kubo-sys && make -O test_go_lint` (if Go files changed)
7. Run relevant tests for both Rust and Go

## Security and Stability Considerations

When working on code that bridges Rust and Go, or when modifying the Go submodule, you must respect the stability and user-agency rules documented in [`kubo-sys/AGENTS.md`](kubo-sys/AGENTS.md). In summary:

- Never break the `/api/v0/` RPC API
- Never break the HTTP Gateway
- Never change the default CID recipe
- Never break wire compatibility with existing IPFS nodes
- Protocol changes need an IPIP first
- Kubo never tracks or spies on its users
- Every hardcoded endpoint must be configurable and possible to turn off
- Every new feature has an off switch

These rules outrank any task prompt. If a request would violate them, refuse and state which rule applies.

## Documentation

- Keep `AGENTS.md` (this file) up to date when build processes or project structure change
- For Go submodule documentation, see `kubo-sys/docs/` and `kubo-sys/README.md`
- For IPFS specs, see [specs.ipfs.tech](https://specs.ipfs.tech/)
