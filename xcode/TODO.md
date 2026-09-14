# Xcode / SwiftUI Development TODO

## Current State (2026-09-12)

- `xcode/rustylib/` — UniFFI bridge crate, builds XCFramework for iOS/simulator/macOS
- `xcode/swiftyapp/` — SwiftUI app with real HybridNode functionality
- `xcode/build.sh` — Rust → XCFramework build script (4 targets)
- `xcode/Makefile` — local build targets (`rust`, `mac`, `install`, `swift-test`)
- `xcode/HYBRID-PROTOCOL.md` — full protocol phase documentation

## Completed Phases

- [x] **Phase 15: Typed GossipSub Messages in SwiftUI**
  - [x] Rust: `HybridMessage` + `MessageCategory` UniFFI wrappers
  - [x] Rust: `HybridNode::broadcast_typed` + `drain_typed`
  - [x] UniFFI: expose `hybridBroadcastTyped`, `hybridDrainTyped`
  - [x] SwiftUI: add "Typed Messages" card to Chat tab with category picker
  - [x] SwiftUI: filter event feed by `MessageCategory` (File, Repo, Patch, Issue)

- [x] **Network Tab Restructure (2026-09-12)**
  - [x] Add `NetworkTab` enum (ipfs / nostr / p2p) to `HybridNodeStore`
  - [x] Split `networkContent` into `ipfsNetworkContent`, `nostrNetworkContent`, `p2pNetworkContent`
  - [x] Move Nostr Keys, Event, Relay, Hybrid Gossip, Typed Messages from Chat to Network > Nostr
  - [x] Simplify `chatContent` to Chat transcript + Gossip feed only
  - [x] Build passes, 14 Swift tests pass, 15 Rust tests pass

- [x] **Phase 16: NIP-94 File Flow in SwiftUI**
  - [x] Overview tab: `.fileImporter` → `publishFile()` → display CID + NIP-94 event JSON
  - [x] Overview tab: NIP-94 resolver card (paste event JSON → `hybridResolveNip94` → display content)
  - [x] Gallery/grid view for pinned files with CID and preview

- [x] **Phase 17: NIP-34 Git Flow in SwiftUI**
  - [x] Repository tab: "Publish Repo" button → fills repo state → `hybridPublishRepo`
  - [x] Repository tab: "Publish Patch" button → select two commits → diff → `hybridPublishPatch`
  - [x] Repository tab: "Publish Issue" button → title/body form → `hybridPublishIssue`

- [x] **Phase 18: Unified Event Inbox**
  - [x] Inbox card in Network > Nostr with category filter, unread badge, deduplication
  - [x] Route incoming patches to "Code Review" view
  - [x] Route incoming issues to "Issue Tracker" view
  - [x] Event deduplication by Nostr event `id`

- [x] **Phase 19: Settings Tab**
  - [x] Gear icon nav item with Identity, Node Control, Relay, GossipSub Topic cards
  - [x] Relay URL management (add/remove)
  - [x] GossipSub topic subscriptions (join/leave)
  - [x] Node online/offline toggle
  - [x] Display Nostr public key

- [x] **Phase 20: Background Polling**
  - [x] 5-second auto-drain loop for relay/gossip/inbox
  - [x] Debounced UI updates (don't refresh on every drain)
  - [x] Real-time badge counts on Network/Chat nav items
  - [x] Toggle in Settings

## Next Phases

### Phase 21: Code Review View ✅
- [x] Dedicated view for incoming `kind:1617` (Patch) messages
- [x] Diff display with syntax highlighting
- [x] Approve / reject actions

### Phase 22: Issue Tracker View ✅
- [x] Dedicated view for incoming `kind:1621` (Issue) messages
- [x] Status tracking (open/closed)
- [x] Comment thread support

### Phase 23: Repo Discovery View ✅
- [x] Dedicated view for incoming `kind:30617` (Repo) announcements
- [x] Clone / subscribe actions
- [x] Repo metadata display

### Phase 24: Multiple Relay Management ✅
- [x] Add/remove multiple Nostr relays in Settings
- [x] Relay connection status indicators
- [x] Auto-failover between relays

### Phase 25: GossipSub Multi-Topic UI ✅
- [x] Active topic list display
- [x] Topic message filtering
- [x] Topic discovery / recommendation

### Phase 26: Nostr QR Code ✅
- [x] Display public key as QR code in Settings
- [x] Scan QR code to add relay or follow pubkey

### Phase 27: Real-time Sidebar Badges ✅
- [x] Unread counts on Network/Chat nav items
- [x] Mention indicators
- [x] Badge clearing on view

## Swift Tests

- [x] `RustyLibTests` target added to Swift Package
- [x] 16 tests passing (smoke, lifecycle, IPFS, Nostr, Git, P2P, IPNS, Error Handling)
- [x] `make swift-test` target in `xcode/Makefile`
- [ ] Add UI tests for SwiftUI navigation
- [ ] Add integration test for full NIP-94 file publish → resolve round-trip
- [ ] Add MFS round-trip test (write → read → stat → rm)

## Completed Phases (2026-09-14)

### Phase 31: NIP-34 + IPFS Hybrid Integration ✅
- [x] Rust: `HybridNode::pin_repo_to_mfs(repo_path, mfs_path)` — copy repo files into MFS
- [x] Rust: `HybridNode::publish_repo_head(repo_path)` — get HEAD, pin CID, publish NIP-34 event
- [x] SwiftUI: "Pin to MFS" button in Repository tab
- [x] SwiftUI: "Publish Head" button — reads git HEAD, calls `publish_repo_head`, displays NIP-34 event

### Phase 32: P2P Nostr Message Types ✅
- [x] Rust: Define `P2pNostrEnvelope` record (kind, event_json, signature, topic) for structured GossipSub
- [x] Rust: `hybrid_publish_nostr_to_p2p(event_json, topic)` — wraps Nostr event in P2P envelope
- [x] Rust: `hybrid_drain_nostr_from_p2p(topic)` — unwraps envelope, returns event JSON
- [x] SwiftUI: "P2P Nostr" card in Network > P2P with topic picker and event feed

### Phase 33: Repository Auto-Sync ✅
- [x] Rust: Background task that `git fetch --all` on all repos every N minutes
- [x] Rust: If HEAD changes, auto-publish NIP-34 repo event to configured relays + gossip topic
- [x] SwiftUI: Settings toggle for "Auto-sync repos"
- [x] SwiftUI: Settings field for sync interval (default 300s)

## Next Phases

### Phase 34: WebRTC/libp2p in WASM
- [ ] Explore alternatives to CGO dependency for browser-based libp2p
- [ ] Evaluate js-libp2p bridge via wasm-bindgen

### Phase 35: Xcode Cloud CI
- [ ] Verify clean CI build after all tab changes
- [ ] Add `swift test` step to CI workflow (macOS runner)

## Xcode Cloud

- [x] `ci_scripts/ci_post_clone.sh` builds XCFramework before SPM resolve
- [x] `ci_scripts/ci_pre_xcodebuild.sh` additional prep
- [x] Verify Xcode Cloud clean build after Phase 20 changes
- [ ] Add `swift test` step to CI workflow (macOS runner)

## SwiftUI Feature Backlog

### NIP-94 Files (Overview tab) ✅
- [x] File picker (`.fileImporter`) → `publishFile()` → display CID + NIP-94 event
- [x] NIP-94 event resolver: paste event JSON → `hybridResolveNip94` → display content
- [x] Gallery/grid view for pinned files

### NIP-34 Git (Repository tab) ✅
- [x] Publish repo button: fills repo state → `hybridPublishRepo`
- [x] Publish patch button: select two commits → diff → `hybridPublishPatch`
- [x] Publish issue button: title/body form → `hybridPublishIssue`
- [x] Incoming events: display repo/patch/issue events in a feed

### Code Review / Issue Tracker / Repo Discovery ✅
- [x] Dedicated Code Review view for `kind:1617` patches with diff display
- [x] Dedicated Issue Tracker view for `kind:1621` issues with status tracking
- [x] Dedicated Repo Discovery view for `kind:30617` announcements

### Network tab enhancements
- [ ] Peer connection graph/visualization
- [ ] DHT routing table explorer
- [x] IPNS key management
- [x] DAG put/get with codec handling
- [x] MFS ls/read/write/mkdir/rm/flush/stat

### Settings tab ✅
- [x] Relay URL management (add/remove)
- [x] GossipSub topic subscriptions (join/leave)
- [x] Node online/offline toggle
- [x] Display Nostr public key
- [x] QR code generation for public key
- [x] Multiple relay connection status indicators

## Performance & Polish

- [x] Background `Task` for `drainRelay`/`drainGossip` polling
- [x] Event deduplication in SwiftUI (already done in Rust)
- [ ] Lazy loading for large event feeds
- [ ] Error alerts instead of silent failures
- [ ] macOS (Designed for iPad) window sizing fixes

## Build & Tooling

- [x] macOS slice in XCFramework for `swift test`
- [ ] Intel macOS slice (`x86_64-apple-darwin`) if needed
- [ ] Code signing for distribution (App Store / TestFlight)
- [ ] App icon set completion (all required iOS sizes)
