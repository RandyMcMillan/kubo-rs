# kubo-rs Development TODO

## Context / Handoff Note

**Last session: 2026-09-12** — Phases 16-20 completed (NIP-94, NIP-34, Inbox, Settings, Background Polling). All SwiftUI tabs now have real functionality.

**Completed this session:**
- **Phase 16: NIP-94 File Flow** — `.fileImporter` → `publishFile()` → CID display; `resolveNip94` resolver card
- **Phase 17: NIP-34 Git Flow** — Publish Repo/Patch/Issue cards in Repository tab
- **Phase 18: Unified Event Inbox** — Inbox card in Network > Nostr with category filter, unread badge, deduplication
- **Phase 19: Settings Tab** — Gear icon nav item with Identity, Node Control, Relay, GossipSub Topic
- **Phase 20: Background Polling** — 5-second auto-drain loop for relay/gossip/inbox, toggle in Settings
- **CI fixes** — clippy warn-not-fail, `make.yml` workflow, `xcode-release.yml`, wasm-bindgen cache fix
- **Build tooling** — auto-install wasi-sdk + wasmtime for `make run-wasm-hybrid-wasi-cli`
- **Full test matrix green** — `cargo test --workspace` + 14 Swift tests + `make mac` build

---

## Active Work (In Progress)

- [ ] **Xcode Cloud validation** — verify clean build on Xcode Cloud after all tab changes
- [ ] **Phase 28: IPNS Key Management** — generate, list, rotate IPNS keys
- [ ] **Phase 29: DAG API** — DAG put/get operations in SwiftUI
- [ ] **Phase 30: MFS / Files API** — mutable file system operations

## Recently Completed (2026-09-12)

- [x] **Phase 21: Code Review View** — `CodeReviewView.swift` with diff display, approve/reject
- [x] **Phase 22: Issue Tracker View** — `IssueTrackerView.swift` with status tracking
- [x] **Phase 23: Repo Discovery View** — `RepoDiscoveryView.swift` with clone/subscribe
- [x] **Phase 24: Multiple Relay Management** — `RelayEntry` model, multi-relay Settings UI
- [x] **Phase 25: GossipSub Multi-Topic UI** — `TopicEntry` model, join/leave/remove topics
- [x] **Phase 26: Nostr QR Code** — `generateQRCode()` helper, Identity card QR display
- [x] **Phase 27: Real-time Sidebar Badges** — red badge pills, auto-clear on navigation
- [x] **Phase 16: NIP-94 File Flow** — file picker, publish, resolver in Overview tab
- [x] **Phase 17: NIP-34 Git Flow** — publish repo/patch/issue from Repository tab
- [x] **Phase 18: Unified Event Inbox** — merge relay + gossip, category filter, unread badge
- [x] **Phase 19: Settings Tab** — Identity, Node Control, Relay, GossipSub Topic cards
- [x] **Phase 20: Background Polling** — auto-drain loop, Settings toggle
- [x] **Network tab restructure** — IPFS / NOSTR / P2P segmented picker
- [x] **make.yml CI** — dispatch options for testing make commands
- [x] **wasi-sdk auto-install** — `make wasm-hybrid-wasi` works on clean machine
- [x] **wasmtime auto-install** — `make run-wasm-hybrid-wasi-cli` works end-to-end

## Previously Completed

- [x] **Phase 7-15** — Multi-topic GossipSub, Swift UniFFI, NIP-94/34 Rust APIs, Typed Messages, Git API, RustyError
- [x] **Phase 4-5** — HybridNode wrapper, hybrid CLI example
- [x] **Xcode build** — ad-hoc signing, `macOS (Designed for iPad)`, XCFramework build script
- [x] **CI workflows** — cache-factory, ci.yml, macos-ci.yml, rust.yml, xcode-release.yml
- [x] **Workspace reorg** — `xcode/rustylib/` added, `go/kubo-sys/ffi/` removed

---

## Phase Map

| Phase | Status | Description |
|-------|--------|-------------|
| 1-6 | Done | Nostr relay read, GossipSub bridge, IPFS content addressing |
| 7 | Done | Multi-topic GossipSub |
| 8 | Done | Swift UniFFI Exposure |
| 9 | Done | NIP-94 IPFS Resolution (Rust) |
| 10 | Done | NIP-34 Git over GossipSub (Rust) |
| 11 | Done | Swift UniFFI Update (NIP-94/34) |
| 12-14 | Done | Git API + RustyError |
| 15 | Done | Typed GossipSub Messages |
| 16 | Done | NIP-94 File Flow in SwiftUI |
| 17 | Done | NIP-34 Git Flow in SwiftUI |
| 18 | Done | Unified Event Inbox |
| 19 | Done | Settings Tab |
| 20 | Done | Background Polling |
| 21 | Done | Code Review View (Patch messages) |
| 22 | Done | Issue Tracker View (Issue messages) |
| 23 | Done | Repo Discovery View (Repo announcements) |
| 24 | Done | Multiple Relay Management |
| 25 | Done | GossipSub Multi-Topic UI |
| 26 | Done | Nostr QR Code |
| 27 | Done | Real-time Sidebar Badges |
| 28 | Future | IPNS Key Management |
| 29 | Future | DAG API |
| 30 | Future | MFS / Files API |

---

## FFI Function Inventory

See `FFI.md` for full details.
