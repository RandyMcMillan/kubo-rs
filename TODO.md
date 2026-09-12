# kubo-rs Development TODO

## Context / Handoff Note

**Last session: 2026-09-09** — SwiftUI real functionality, Network/IPFS/Nostr menus, xcode-release fix, full test validation.

**Completed this session:**
- **Regenerated UniFFI bindings** — throwing functions (`hybridStartTry`, `ipfsAddTry`, `ipfsCatTry`, etc.) now in generated Swift
- **SwiftUI real functionality** — `DashboardStore` replaced with `HybridNodeStore` using live UniFFI APIs
  - **Overview**: IPFS add/cat/pin, block put/get/stat, live peer IDs, pin list
  - **Repository**: Git init/clone, head/branches/remotes/status, commit lookup, diff trees
  - **Network**: libp2p dial, DHT findpeer/findprovs, IPNS publish/resolve, Nostr relay/gossip
  - **Chat**: Nostr keygen, kind-1 event signing, relay connect/publish/drain, hybrid gossip pub/drain
- **xcode-release.yml fix** — moved `HAS_CERT` from workflow-level `env` to job-level `env` (GitHub Actions `secrets` context restriction)
- **Xcode build green** — `make mac` succeeds for `macOS (Designed for iPad)` target
- **Full test matrix green** — `cargo test --workspace` + `make test_unit` (2,165 passed) + `make test-ffi` (all passed)

---

## Active Work (In Progress)

- [ ] **Xcode Cloud validation** — push changes and verify a clean build on Xcode Cloud
- [ ] **Protocol phase continuation** — NIP-34 hybrid integration, p2p nostr message types, WASM examples

## Recently Completed (2026-09-09)

- [x] **SwiftUI real functionality** — `HybridNodeStore` with live IPFS, Git, Nostr, GossipSub APIs
- [x] **Network tab enrichment** — IPFS DHT, IPNS, libp2p dial + Nostr relay/gossip sub-menus
- [x] **Repository extras** — Git commit lookup and diff trees
- [x] **Overview extras** — Block put/get/stat operations
- [x] **xcode-release.yml fix** — `secrets` context moved to job-level `env`
- [x] **UniFFI bindings regenerated** — throwing functions now available in Swift
- [x] **Clippy warn-not-fail** — `Makefile` and CI workflows clean
- [x] **wasm-bindgen sync** — all example crates updated to `=0.2.128`
- [x] **wasm-p2p example** — pure Rust libp2p in browser WASM using WebRTC + WebSocket
- [x] **CLI commands finished** — pin-add/rm/ls, name-publish/resolve, disconnect, dht-findpeer/findprovs
- [x] **Tests added** — `cli_pin_add_rm_ls` in `tests/cli.rs`; inline tests for disconnect, name, dht
- [x] **Full test matrix green** — `cargo test --workspace` + `make test_unit` (2,165 passed) + FFI tests passed

## Previously Completed

- [x] **Phase 7: Multi-topic GossipSub** — Go FFI `gossip_join/leave/publish_to`; Rust safe API; Swift UniFFI exposure
- [x] **Phase 8: Swift UniFFI Exposure** — `hybrid_start/stop`, `broadcast_event`, `drain_events`, `publish_file`, `nostr_*`, `relay_*`, `p2p_gossip_join/leave/publish_to`
- [x] **Phase 9: NIP-94 IPFS Resolution** — `HybridNode::resolve_nip94` parses kind 1063 events, extracts `ipfs://` CID, fetches content via IPFS
- [x] **Phase 10: NIP-34 Git over GossipSub** — `HybridNode::publish_repo`, `publish_patch`, `publish_issue`
- [x] **Phase 11: Swift UniFFI Update** — exposed `resolve_nip94`, `publish_repo`, `publish_patch`, `publish_issue` to Swift
- [x] **Phase 12-14: Git API + RustyError** — `git_clone`, `git_init`, `Repository` methods; `RustyError` enum with throwing variants
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

1. **Phase 16: NIP-94 File Flow in SwiftUI** — `.fileImporter` → `publishFile()` → display CID + NIP-94 event; `resolveNip94` event JSON → fetch content
2. **Phase 17: NIP-34 Git Flow in SwiftUI** — Repository tab buttons: Publish Repo (kind 30617), Publish Patch (kind 1617), Publish Issue (kind 1621)
3. **Phase 18: Unified Event Inbox** — Merge relay + gossip events into a single feed; route patches to Code Review view, issues to Issue Tracker view
4. **Phase 19: Settings Tab** — Relay URL management, GossipSub topic subscriptions, node online/offline toggle, Nostr public key + QR code
5. **Phase 20: Background Polling** — `Task` loops for `drainRelay`/`drainGossip` with debouncing; real-time event feed updates
6. **Xcode Cloud validation** — verify clean build after Network tab restructuring
7. **Future: DAG API** — `dag get`, `dag put`, `dag resolve` (complex due to ipld-prime)
8. **Future: Key API** — `key gen`, `key list`, `key rm` (needed for advanced IPNS)
9. **Future: MFS / Files API** — `files ls`, `files read`, `files write`, `files mkdir`
10. **Future: PubSub** — `pubsub pub`, `pubsub sub`, `pubsub peers`, `pubsub ls`
11. **Future: Bootstrap** — `bootstrap list`, `bootstrap add`, `bootstrap rm`
12. **Future: Repo GC** — `repo stat`, `repo gc`
13. **Future: wasm-p2p enhancements** — dial input UI, WebRTC signaling, gossipsub integration
