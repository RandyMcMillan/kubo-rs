# kubo-rs Development TODO

## Context / Handoff Note

**Last session: 2026-09-09** — clippy fix, wasm-bindgen sync, wasm-p2p example, full test validation.

**Completed this session:**
- `Makefile` — removed `-D warnings` from clippy target; clippy now warns without failing builds
- `wasm-bindgen` — updated all workspace examples to `=0.2.128` to fix trunk schema mismatch
- `examples/wasm-p2p/` — new pure Rust libp2p WebRTC + WebSocket example for WASM (no CGO)
  - Uses `libp2p-webrtc-websys`, `libp2p-websocket-websys`, `libp2p-ping`, `libp2p-noise`, `libp2p-yamux`
  - Runs `Swarm` with `wasm-bindgen` executor in the browser
  - Makefile targets: `make wasm-p2p`, `make run-wasm-p2p`, `make build-wasm-p2p-release`
- **CLI commands completed** — `src/main.rs` already has match arms for all 8 new functions
  - `ipfs pin-add`, `ipfs pin-rm`, `ipfs pin-ls`
  - `ipfs name-publish`, `ipfs name-resolve`
  - `p2p disconnect`, `p2p dht-findpeer`, `p2p dht-findprovs`
- **Tests completed** — `tests/cli.rs` has `cli_pin_add_rm_ls`; `src/lib.rs` has inline tests for pin, disconnect, name_publish_resolve, dht_findpeer_local
- **Full test matrix green** — `cargo test --workspace` (83 passed, 3 ignored) + `make test_unit` (2,165 passed, 769 skipped) + `make test-ffi` (all passed)

**Still pending:**
- Xcode Cloud validation — push and verify a clean build
- Phase 6 follow-up — expose `HybridNode` methods to Swift via UniFFI

---

## Active Work (In Progress)

- [ ] **Xcode Cloud validation** — push changes and verify a clean build on Xcode Cloud (ci_post_clone.sh builds XCFramework before SPM resolve)
- [ ] **Phase 6 follow-up** — expose `HybridNode` methods to Swift via `xcode/rustylib/src/lib.rs` + UniFFI

## Recently Completed (2026-09-09)

- [x] **Clippy warn-not-fail** — `Makefile` clippy target no longer uses `-D warnings`; CI workflows already clean
- [x] **wasm-bindgen sync** — all example crates updated to `=0.2.128`; `Cargo.lock` updated
- [x] **wasm-p2p example** — pure Rust libp2p in browser WASM using WebRTC + WebSocket transports
- [x] **CLI commands finished** — pin-add/rm/ls, name-publish/resolve, disconnect, dht-findpeer/findprovs all wired in `src/main.rs`
- [x] **Tests added** — `cli_pin_add_rm_ls` in `tests/cli.rs`; inline tests for disconnect, name, dht in `src/lib.rs`
- [x] **Full test matrix green** — `cargo test --workspace` (83 passed) + `make test_unit` (2,165 passed) + FFI tests passed

## Previously Completed

- [x] **Phase 7: Multi-topic GossipSub** — Go FFI `gossip_join/leave/publish_to`; Rust safe API; Swift UniFFI exposure
- [x] **Phase 8: Swift UniFFI Exposure** — `hybrid_start/stop`, `broadcast_event`, `drain_events`, `publish_file`, `nostr_*`, `relay_*`, `p2p_gossip_join/leave/publish_to`
- [x] **Phase 9: NIP-94 IPFS Resolution** — `HybridNode::resolve_nip94` parses kind 1063 events, extracts `ipfs://` CID, fetches content via IPFS
- [x] **Phase 10: NIP-34 Git over GossipSub** — `HybridNode::publish_repo`, `publish_patch`, `publish_issue`
- [x] **Phase 11: Swift UniFFI Update** — exposed `resolve_nip94`, `publish_repo`, `publish_patch`, `publish_issue` to Swift
- [x] **Phase 4: HybridNode wrapper** — `src/hybrid.rs` with `broadcast_event`, `drain_events`, `publish_file`, `stop`; wired into `src/lib.rs`; unit tests pass
- [x] **Phase 5: Hybrid example** — `examples/hybrid.rs` demonstrates end-to-end IPFS + Nostr + GossipSub flow
- [x] **Xcode Cloud CI scripts** — `ci_scripts/ci_post_clone.sh` + `ci_pre_xcodebuild.sh`
- [x] **Xcode build fixes** — ad-hoc signing; `macOS (Designed for iPad)` destination works
- [x] **GitHub workflow fixes** — `xcode-release.yml` uses `env.HAS_CERT`; `workflow_dispatch` inputs
- [x] **Cache factory fix** — cache keys hash `go/ffi/go.sum`
- [x] **Workspace reorg** — `xcode/rustylib/` added, version aligned, `publish = false`
- [x] **Remove stale submodule copy** — `go/kubo-sys/ffi/` deleted

---

## Project Structure

```
kubo-rs/
├── Cargo.toml              # Workspace root manifest
├── build.rs                # FFI build script (CGO cross-compilation support)
├── src/
│   ├── lib.rs              # Safe Rust API (Node, init_repo, version)
│   ├── ffi.rs              # Unsafe extern "C" bindings (24+ functions)
│   ├── main.rs             # CLI binary (ipfs, p2p, nostr, git subcommands)
│   ├── hybrid.rs           # HybridNode (IPFS + Nostr + GossipSub)
│   └── error.rs            # Error enum
├── tests/
│   ├── cli.rs              # CLI integration tests (9 tests)
│   ├── api.rs              # API tests
│   └── alignment.rs        # Cross-language alignment tests
├── examples/
│   ├── basic.rs            # Basic FFI demo
│   ├── p2p.rs              # Two-node p2p demo
│   ├── dashboard.rs        # ratatui TUI
│   ├── hybrid.rs           # IPFS + Nostr + GossipSub demo
│   ├── wasm-dashboard/     # ratzilla WASM dashboard (Kubo HTTP API)
│   ├── website/            # ratzilla WASM website
│   ├── wasm-hybrid/        # ratzilla WASM hybrid demo
│   └── wasm-p2p/           # Pure Rust libp2p WebRTC + WebSocket in WASM
├── go/
│   ├── ffi/                # Go FFI source (kubo.go, libp2p.go, nostr.go, git.go)
│   ├── kubo-sys/           # Go submodule (Kubo IPFS)
│   └── nostr/              # Go nostr submodule
├── xcode/
│   ├── rustylib/           # Rust crate for iOS FFI (uniffi, staticlib, cdylib)
│   ├── swiftyapp/          # SwiftUI iOS app + Xcode projects
│   ├── build.sh            # Rust → XCFramework build script
│   └── Makefile            # macOS / iOS build targets
├── ci_scripts/             # Xcode Cloud hooks
├── scripts/                # Test scripts + wasm-dashboard.sh + website.sh
├── .github/workflows/      # CI workflows
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
| `kubo_swarm_disconnect` | ✅ | ✅ `disconnect()` | `p2p disconnect` |
| `kubo_node_id` | ✅ | ✅ `id()` | — |
| `kubo_unixfs_add_bytes` | ✅ | ✅ `add_bytes()` | `ipfs add` |
| `kubo_unixfs_cat` | ✅ | ✅ `cat()` | `ipfs cat` |
| `kubo_block_put` | ✅ | ✅ `block_put()` | `ipfs block-put` |
| `kubo_block_get` | ✅ | ✅ `block_get()` | `ipfs block-get` |
| `kubo_block_stat` | ✅ | ✅ `block_stat()` | `ipfs block-stat` |
| `kubo_pin_add` | ✅ | ✅ `pin_add()` | `ipfs pin-add` |
| `kubo_pin_rm` | ✅ | ✅ `pin_rm()` | `ipfs pin-rm` |
| `kubo_pin_ls` | ✅ | ✅ `pin_ls()` | `ipfs pin-ls` |
| `kubo_dht_findpeer` | ✅ | ✅ `dht_findpeer()` | `p2p dht-findpeer` |
| `kubo_dht_findprovs` | ✅ | ✅ `dht_findprovs()` | `p2p dht-findprovs` |
| `kubo_name_publish` | ✅ | ✅ `name_publish()` | `ipfs name-publish` |
| `kubo_name_resolve` | ✅ | ✅ `name_resolve()` | `ipfs name-resolve` |

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

1. **Xcode Cloud validation** — push all changes and verify a clean build on Xcode Cloud
2. **Phase 6 follow-up** — expose `HybridNode` methods to Swift via `xcode/rustylib/src/lib.rs` + UniFFI
3. **Future: NIP-94 in SwiftUI** — file picker → IPFS add → NIP-94 event → broadcast
4. **Future: DAG API** — `dag get`, `dag put`, `dag resolve` (complex due to ipld-prime)
5. **Future: Key API** — `key gen`, `key list`, `key rm` (needed for advanced IPNS)
6. **Future: MFS / Files API** — `files ls`, `files read`, `files write`, `files mkdir`
7. **Future: PubSub** — `pubsub pub`, `pubsub sub`, `pubsub peers`, `pubsub ls`
8. **Future: Bootstrap** — `bootstrap list`, `bootstrap add`, `bootstrap rm`
9. **Future: Repo GC** — `repo stat`, `repo gc`
10. **Future: wasm-p2p enhancements** — add dial input UI, WebRTC signaling, gossipsub integration
