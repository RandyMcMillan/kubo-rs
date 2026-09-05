# kubo-rs Development TODO

## Context / Handoff Note

**Last session: 2026-09-05** — In progress: adding core IPFS protocol functions (Pin, Swarm disconnect, DHT, Name/IPNS).

**Uncommitted changes** (on `main` working tree):
- `go/ffi/kubo.go` — 8 new Go FFI exports added
- `src/ffi.rs` — Rust unsafe bindings + safe wrappers for all 8
- `src/lib.rs` — `Node` methods added for all 8
- `scripts/website.sh` + `examples/website/.gitignore` — new website tooling
- `Makefile` — `website`, `run-website`, `build-website-release` targets
- `.github/workflows/gh-pages.yml` — website build step added
- `scripts/wasm-dashboard.sh` + `scripts/website.sh` — auto-port-selection + CORS restart fixes

**Rust builds successfully** (`cargo build` passes). Go FFI not yet rebuilt/tested.

---

## Active Work (In Progress)

- [ ] **Add CLI commands for new FFI functions** — `src/main.rs` needs new variants in `IpfsCommands` and `P2pCommands`, plus match arms in `run()`
  - `ipfs pin-add <cid> [--recursive]`
  - `ipfs pin-rm <cid> [--recursive]`
  - `ipfs pin-ls`
  - `ipfs name-publish <cid> [--lifetime-sec]`
  - `ipfs name-resolve <name>`
  - `p2p disconnect <addr>`
  - `p2p dht-findpeer <peer-id>`
  - `p2p dht-findprovs <cid>`
- [ ] **Add tests** — inline `src/lib.rs` tests + `tests/cli.rs` tests for pin/dht/name
- [ ] **Build Go FFI** — `cd go/ffi && go build` to verify new Go code compiles
- [ ] **Run full matrix** — `cargo build && cargo test && cargo fmt && cargo clippy`

## Recently Completed (uncommitted)

- [x] **Add Pin/DHT/Name/Swarm-disconnect to Go FFI** (`go/ffi/kubo.go`)
  - `kubo_swarm_disconnect`
  - `kubo_pin_add` / `kubo_pin_rm` / `kubo_pin_ls`
  - `kubo_dht_findpeer` / `kubo_dht_findprovs`
  - `kubo_name_publish` / `kubo_name_resolve`
- [x] **Add Rust FFI bindings** (`src/ffi.rs`) — unsafe extern declarations + safe wrappers
- [x] **Add safe Node methods** (`src/lib.rs`) — `disconnect`, `pin_add`, `pin_rm`, `pin_ls`, `dht_findpeer`, `dht_findprovs`, `name_publish`, `name_resolve`
- [x] **Add website tooling** — `scripts/website.sh`, Makefile targets, CI build step
- [x] **Fix wasm-dashboard/website port conflicts** — auto-pick free port, dynamic CORS, daemon restart logic
- [x] **Fix CORS restart bug** — `lsof -ti :5001` returns multiple PIDs; old `kill "$pid"` failed; now uses `kill $pids` (unquoted) + `kill -9` fallback

---

## Project Structure

```
kubo-rs/
├── Cargo.toml              # Main crate manifest
├── build.rs                # FFI build script (CGO cross-compilation support)
├── src/
│   ├── lib.rs              # Safe Rust API (Node, init_repo, version)
│   ├── ffi.rs              # Unsafe extern "C" bindings (24+ functions now)
│   ├── main.rs             # CLI binary (ipfs, p2p, nostr subcommands)
│   └── error.rs            # Error enum
├── tests/cli.rs            # CLI integration tests (7 tests)
├── examples/
│   ├── basic.rs            # Basic FFI demo
│   ├── p2p.rs              # Two-node p2p demo
│   ├── dashboard.rs        # ratatui TUI
│   ├── wasm-dashboard/     # ratzilla WASM demo
│   └── website/            # ratzilla WASM website
├── go/
│   ├── ffi/                # Go FFI source (kubo.go, libp2p.go, nostr.go, git.go)
│   ├── kubo-sys/           # Go submodule (Kubo IPFS)
│   └── nostr/              # Go nostr submodule
├── scripts/                # Test scripts + wasm-dashboard.sh + website.sh
├── Makefile                # Build/test targets
├── FFI.md                  # FFI architecture documentation
├── RELEASE.md              # cargo-dist release guide
└── TODO.md                 # This file
```

---

## FFI Function Inventory (24+ total)

### Kubo (Node)
| Go Function | Rust Binding | Safe Wrapper | CLI |
|---|---|---|---|
| `kubo_version` | ✅ | ✅ `version()` | `ipfs version` |
| `kubo_init_repo` | ✅ | ✅ `init_repo()` | `ipfs init` |
| `kubo_node_start` | ✅ | ✅ `Node::start()` | (internal) |
| `kubo_node_stop` | ✅ | ✅ `Node::stop()` / Drop | — |
| `kubo_node_start_api` | ✅ | ✅ `start_api()` | `ipfs daemon --api` |
| `kubo_node_api_addrs` | ✅ | ✅ `api_addrs()` | — |
| `kubo_node_peer_id` | ✅ | ✅ `peer_id()` | `ipfs peer-id`, `p2p peer-id` |
| `kubo_node_listening_addrs` | ✅ | ✅ `listening_addrs()` | `p2p listen` |
| `kubo_node_connect` | ✅ | ✅ `connect()` | `p2p connect` |
| `kubo_swarm_peers` | ✅ | ✅ `swarm_peers()` | — |
| `kubo_swarm_disconnect` | ✅ | ✅ `disconnect()` | `p2p disconnect` *(needs CLI)* |
| `kubo_node_id` | ✅ | ✅ `id()` | — |
| `kubo_unixfs_add_bytes` | ✅ | ✅ `add_bytes()` | `ipfs add` |
| `kubo_unixfs_cat` | ✅ | ✅ `cat()` | `ipfs cat` |
| `kubo_block_put` | ✅ | ✅ `block_put()` | `ipfs block-put` |
| `kubo_block_get` | ✅ | ✅ `block_get()` | `ipfs block-get` |
| `kubo_block_stat` | ✅ | ✅ `block_stat()` | `ipfs block-stat` |
| `kubo_pin_add` | ✅ | ✅ `pin_add()` | `ipfs pin-add` *(needs CLI)* |
| `kubo_pin_rm` | ✅ | ✅ `pin_rm()` | `ipfs pin-rm` *(needs CLI)* |
| `kubo_pin_ls` | ✅ | ✅ `pin_ls()` | `ipfs pin-ls` *(needs CLI)* |
| `kubo_dht_findpeer` | ✅ | ✅ `dht_findpeer()` | `p2p dht-findpeer` *(needs CLI)* |
| `kubo_dht_findprovs` | ✅ | ✅ `dht_findprovs()` | `p2p dht-findprovs` *(needs CLI)* |
| `kubo_name_publish` | ✅ | ✅ `name_publish()` | `ipfs name-publish` *(needs CLI)* |
| `kubo_name_resolve` | ✅ | ✅ `name_resolve()` | `ipfs name-resolve` *(needs CLI)* |

### libp2p (Host)
| Go Function | Rust Binding | Safe Wrapper | CLI |
|---|---|---|---|
| `kubo_libp2p_host_new` | ✅ | ✅ `Host::new()` | `libp2p host` |
| `kubo_libp2p_host_close` | ✅ | ✅ `Host::close()` / Drop | — |
| `kubo_libp2p_host_peer_id` | ✅ | ✅ `peer_id()` | — |
| `kubo_libp2p_host_listening_addrs` | ✅ | ✅ `listening_addrs()` | `libp2p listen` |
| `kubo_libp2p_host_connect` | ✅ | ✅ `connect()` | `libp2p connect` |
| `kubo_libp2p_host_ping` | ✅ | ✅ `ping()` | — |
| `kubo_libp2p_host_protocols` | ✅ | ✅ `protocols()` | — |

### Nostr
| Go Function | Rust Binding | Safe Wrapper | CLI |
|---|---|---|---|
| `kubo_nostr_generate_key` | ✅ | ✅ `nostr_generate_key()` | `nostr keygen` |
| `kubo_nostr_get_public_key` | ✅ | ✅ `nostr_get_public_key()` | — |
| `kubo_nostr_event_sign` | ✅ | ✅ `nostr_event_sign()` | `nostr sign` |
| `kubo_nostr_event_verify` | ✅ | ✅ `nostr_event_verify()` | `nostr verify` |
| `kubo_nostr_nip19_*` | ✅ | ✅ (8 functions) | — |
| `kubo_nostr_nip05_*` | ✅ | ✅ (2 functions) | — |
| `kubo_nostr_relay_connect` | ✅ | ✅ `nostr_relay_connect()` | — |
| `kubo_nostr_relay_close` | ✅ | ✅ `nostr_relay_close()` | — |
| `kubo_nostr_relay_publish` | ✅ | ✅ `nostr_relay_publish()` | `nostr publish` |

### Git
| Go Function | Rust Binding | Safe Wrapper | CLI |
|---|---|---|---|
| `kubo_git_clone` | ✅ | ✅ `git_clone()` | `git clone` |
| `kubo_git_init` | ✅ | ✅ `git_init()` | `git init` |
| `kubo_git_open` | ✅ | ✅ `Repository::open()` | — |
| `kubo_git_repo_head` | ✅ | ✅ `head()` | `git head` |
| `kubo_git_repo_free` | ✅ | ✅ `close()` / Drop | — |
| `kubo_git_repo_is_bare` | ✅ | ✅ `is_bare()` | — |
| `kubo_git_repo_branches` | ✅ | ✅ `branches()` | — |
| `kubo_git_repo_remotes` | ✅ | ✅ `remotes()` | — |
| `kubo_git_repo_create_branch` | ✅ | ✅ `create_branch()` | — |
| `kubo_git_repo_commit_lookup` | ✅ | ✅ `commit_message()` | — |
| `kubo_git_repo_tree_entries` | ✅ | ✅ `tree_entries()` | — |
| `kubo_git_repo_blob_read` | ✅ | ✅ `blob_read()` | — |
| `kubo_git_repo_status` | ✅ | ✅ `status()` | — |
| `kubo_git_repo_diff_trees` | ✅ | ✅ `diff_trees()` | — |

---

## Next Steps (Priority)

1. **Finish CLI commands** — add match arms in `src/main.rs` for all 8 new functions
2. **Add tests** — inline lib tests + CLI tests for pin/dht/name
3. **Build Go FFI** — `cd go/ffi && go build` to verify Go compiles
4. **Run full validation** — `cargo build && cargo test && cargo fmt && cargo clippy`
5. **Future: DAG API** — `dag get`, `dag put`, `dag resolve` (complex due to ipld-prime)
6. **Future: Key API** — `key gen`, `key list`, `key rm` (needed for advanced IPNS)
7. **Future: MFS / Files API** — `files ls`, `files read`, `files write`, `files mkdir`
8. **Future: PubSub** — `pubsub pub`, `pubsub sub`, `pubsub peers`, `pubsub ls`
9. **Future: Bootstrap** — `bootstrap list`, `bootstrap add`, `bootstrap rm`
10. **Future: Repo GC** — `repo stat`, `repo gc`
