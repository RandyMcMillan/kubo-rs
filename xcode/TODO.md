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

### Phase 21: Code Review View
- [ ] Dedicated view for incoming `kind:1617` (Patch) messages
- [ ] Diff display with syntax highlighting
- [ ] Approve / reject actions

### Phase 22: Issue Tracker View
- [ ] Dedicated view for incoming `kind:1621` (Issue) messages
- [ ] Status tracking (open/closed)
- [ ] Comment thread support

### Phase 23: Repo Discovery View
- [ ] Dedicated view for incoming `kind:30617` (Repo) announcements
- [ ] Clone / subscribe actions
- [ ] Repo metadata display

### Phase 24: Multiple Relay Management
- [ ] Add/remove multiple Nostr relays in Settings
- [ ] Relay connection status indicators
- [ ] Auto-failover between relays

### Phase 25: GossipSub Multi-Topic UI
- [ ] Active topic list display
- [ ] Topic message filtering
- [ ] Topic discovery / recommendation

### Phase 26: Nostr QR Code
- [ ] Display public key as QR code in Settings
- [ ] Scan QR code to add relay or follow pubkey

### Phase 27: Real-time Sidebar Badges
- [ ] Unread counts on Network/Chat nav items
- [ ] Mention indicators
- [ ] Badge clearing on view

## Swift Tests

- [x] `RustyLibTests` target added to Swift Package
- [x] 14 tests passing (smoke, lifecycle, IPFS, Nostr, Git, P2P, Error Handling)
- [x] `make swift-test` target in `xcode/Makefile`
- [ ] Add UI tests for SwiftUI navigation
- [ ] Add integration test for full NIP-94 file publish → resolve round-trip

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

### Code Review / Issue Tracker / Repo Discovery (Next)
- [ ] Dedicated Code Review view for `kind:1617` patches with diff display
- [ ] Dedicated Issue Tracker view for `kind:1621` issues with status tracking
- [ ] Dedicated Repo Discovery view for `kind:30617` announcements

### Network tab enhancements
- [ ] Peer connection graph/visualization
- [ ] DHT routing table explorer
- [ ] IPNS key management

### Settings tab ✅
- [x] Relay URL management (add/remove)
- [x] GossipSub topic subscriptions (join/leave)
- [x] Node online/offline toggle
- [x] Display Nostr public key
- [ ] QR code generation for public key
- [ ] Multiple relay connection status indicators

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
