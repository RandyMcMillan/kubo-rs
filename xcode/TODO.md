# Xcode / SwiftUI Development TODO

## Current State (2026-09-10)

- `xcode/rustylib/` — UniFFI bridge crate, builds XCFramework for iOS/simulator/macCatalyst/macOS
- `xcode/swiftyapp/` — SwiftUI app with real HybridNode functionality
- `xcode/build.sh` — Rust → XCFramework build script (4 targets)
- `xcode/Makefile` — local build targets (`rust`, `mac`, `install`, `swift-test`)
- `xcode/HYBRID-PROTOCOL.md` — full protocol phase documentation

## Active Work

- [x] **Phase 15: Typed GossipSub Messages in SwiftUI**
  - [x] Rust: `HybridMessage` + `MessageCategory` UniFFI wrappers
  - [x] Rust: `HybridNode::broadcast_typed` + `drain_typed`
  - [x] UniFFI: expose `hybridBroadcastTyped`, `hybridDrainTyped`
  - [x] SwiftUI: add "Typed Messages" card to Chat tab with category picker
  - [x] SwiftUI: filter event feed by `MessageCategory` (File, Repo, Patch, Issue)
  - [ ] SwiftUI: route patches to "Code Review" view, issues to "Issue Tracker"

- [x] **Network Tab Restructure (2026-09-12)**
  - [x] Add `NetworkTab` enum (ipfs / nostr / p2p) to `HybridNodeStore`
  - [x] Split `networkContent` into `ipfsNetworkContent`, `nostrNetworkContent`, `p2pNetworkContent`
  - [x] Move Nostr Keys, Event, Relay, Hybrid Gossip, Typed Messages from Chat to Network > Nostr
  - [x] Simplify `chatContent` to Chat transcript + Gossip feed only
  - [x] Build passes, 14 Swift tests pass, 15 Rust tests pass

## Next Phases

### Phase 16: NIP-94 File Flow in SwiftUI
- [ ] Overview tab: `.fileImporter` → `publishFile()` → display CID + NIP-94 event JSON
- [ ] Overview tab: NIP-94 resolver card (paste event JSON → `hybridResolveNip94` → display content)
- [ ] Gallery/grid view for pinned files with CID and preview

### Phase 17: NIP-34 Git Flow in SwiftUI
- [ ] Repository tab: "Publish Repo" button → fills repo state → `hybridPublishRepo`
- [ ] Repository tab: "Publish Patch" button → select two commits → diff → `hybridPublishPatch`
- [ ] Repository tab: "Publish Issue" button → title/body form → `hybridPublishIssue`

### Phase 18: Unified Event Inbox
- [ ] New "Inbox" view or Network > Nostr enhancement: merge relay + gossip events
- [ ] Route incoming patches to "Code Review" view
- [ ] Route incoming issues to "Issue Tracker" view
- [ ] Event deduplication by Nostr event `id`

### Phase 19: Settings Tab
- [ ] New `DashboardSection.settings` in sidebar
- [ ] Relay URL management (multiple relays, add/remove)
- [ ] GossipSub topic subscriptions (join/leave)
- [ ] Node online/offline toggle
- [ ] Display Nostr public key + QR code

### Phase 20: Background Polling
- [ ] `Task` loop in `HybridNodeStore` for periodic `drainRelay`/`drainGossip`
- [ ] Debounced UI updates (don't refresh on every drain)
- [ ] Real-time badge counts on Network/Chat nav items

## Swift Tests

- [x] `RustyLibTests` target added to Swift Package
- [x] 10 tests passing (smoke, lifecycle, IPFS, Nostr, Git, P2P)
- [x] `make swift-test` target in `xcode/Makefile`
- [ ] Add UI tests for SwiftUI navigation
- [ ] Add integration test for full NIP-94 file publish → resolve round-trip

## Xcode Cloud

- [x] `ci_scripts/ci_post_clone.sh` builds XCFramework before SPM resolve
- [x] `ci_scripts/ci_pre_xcodebuild.sh` additional prep
- [ ] Verify Xcode Cloud clean build after Phase 15 changes
- [ ] Add `swift test` step to CI workflow (macOS runner)

## SwiftUI Feature Backlog

### NIP-94 Files (Overview tab enhancement)
- [ ] File picker (`.fileImporter`) → `publishFile()` → display CID + NIP-94 event
- [ ] NIP-94 event resolver: paste event JSON → `hybridResolveNip94` → display content
- [ ] Gallery/grid view for pinned files

### NIP-34 Git (Repository tab enhancement)
- [ ] Publish repo button: fills repo state → `hybridPublishRepo`
- [ ] Publish patch button: select two commits → diff → `hybridPublishPatch`
- [ ] Publish issue button: title/body form → `hybridPublishIssue`
- [ ] Incoming events: display repo/patch/issue events in a feed

### Network tab enhancements
- [ ] Peer connection graph/visualization
- [ ] DHT routing table explorer
- [ ] IPNS key management

### Settings tab (new)
- [ ] Relay URL management (multiple relays)
- [ ] GossipSub topic subscriptions
- [ ] Node online/offline toggle
- [ ] Display Nostr public key + QR code

## Performance & Polish

- [ ] Background `Task` for `drainRelay`/`drainGossip` polling
- [ ] Event deduplication in SwiftUI (already done in Rust)
- [ ] Lazy loading for large event feeds
- [ ] Error alerts instead of silent failures
- [ ] macOS (Designed for iPad) window sizing fixes

## Build & Tooling

- [x] macOS slice in XCFramework for `swift test`
- [ ] Intel macOS slice (`x86_64-apple-darwin`) if needed
- [ ] Code signing for distribution (App Store / TestFlight)
- [ ] App icon set completion (all required iOS sizes)
