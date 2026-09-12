# Hybrid Protocol — Xcode Integration Notes

This document tracks what needs to happen in the Xcode/Swift layer as the Nostr+IPFS+P2P hybrid protocol evolves in the parent Rust crate.

## Current State (2026-09-12)

The `xcode/rustylib/` crate wraps `kubo-rs` and exposes a **small subset** of the API to Swift via UniFFI. The SwiftUI app (`swiftyapp/`) consumes these through the `RustyLib` local package. Phases 1-20 are complete.

### Already Exposed to Swift (via `#[uniffi::export]`)

| Rust Function | Swift Name | Status |
|---|---|---|
| `rust_hello()` | `rustHello()` | ✅ Demo — init repo, add/cat bytes, return summary |
| `rust_add(a, b)` | `rustAdd(a:b:)` | ✅ Simple smoke test |
| `p2p_start()` | `p2pStart()` | ✅ Starts libp2p Host (lazy singleton) |
| `p2p_peer_id()` | `p2pPeerId()` | ✅ |
| `p2p_listening_addrs()` | `p2pListeningAddrs()` | ✅ |
| `p2p_connect(addr)` | `p2pConnect(addr:)` | ✅ |
| `p2p_ping(peer_id)` | `p2pPing(peerId:)` | ✅ |
| `p2p_protocols()` | `p2pProtocols()` | ✅ |
| `p2p_gossip_topic()` | `p2pGossipTopic()` | ✅ |
| `p2p_gossip_error()` | `p2pGossipError()` | ✅ |
| `p2p_gossip_publish(msg)` | `p2pGossipPublish(message:)` | ✅ |
| `p2p_gossip_drain()` | `p2pGossipDrain()` | ✅ |

### Implemented in Rust, Exposed to Swift (via `#[uniffi::export]`)

| Rust Function | Swift Name | Status |
|---|---|---|
| `hybrid_start(online)` | `hybridStart(online:)` | ✅ Phase 8 |
| `hybrid_stop()` | `hybridStop()` | ✅ Phase 8 |
| `hybrid_ipfs_peer_id()` | `hybridIpfsPeerId()` | ✅ Phase 8 |
| `hybrid_p2p_peer_id()` | `hybridP2pPeerId()` | ✅ Phase 8 |
| `hybrid_broadcast_event(...)` | `hybridBroadcastEvent(...)` | ✅ Phase 8 |
| `hybrid_drain_events(...)` | `hybridDrainEvents(...)` | ✅ Phase 8 |
| `hybrid_publish_file(...)` | `hybridPublishFile(...)` | ✅ Phase 8 |
| `hybrid_resolve_nip94(...)` | `hybridResolveNip94(...)` | ✅ Phase 11 |
| `hybrid_publish_repo(...)` | `hybridPublishRepo(...)` | ✅ Phase 11 |
| `hybrid_publish_patch(...)` | `hybridPublishPatch(...)` | ✅ Phase 11 |
| `hybrid_publish_issue(...)` | `hybridPublishIssue(...)` | ✅ Phase 11 |
| `p2p_gossip_join(topic)` | `p2pGossipJoin(topic:)` | ✅ Phase 8 |
| `p2p_gossip_leave(topic)` | `p2pGossipLeave(topic:)` | ✅ Phase 8 |
| `p2p_gossip_publish_to(...)` | `p2pGossipPublishTo(...)` | ✅ Phase 8 |
| `nostr_generate_key()` | `nostrGenerateKey()` | ✅ Phase 8 |
| `nostr_get_public_key(sk)` | `nostrGetPublicKey(sk:)` | ✅ Phase 8 |
| `nostr_event_sign(...)` | `nostrEventSign(...)` | ✅ Phase 8 |
| `nostr_event_verify(...)` | `nostrEventVerify(...)` | ✅ Phase 8 |
| `nostr_relay_connect(url)` | `nostrRelayConnect(url:)` | ✅ Phase 8 |
| `nostr_relay_close(handle)` | `nostrRelayClose(handle:)` | ✅ Phase 8 |
| `nostr_relay_publish(...)` | `nostrRelayPublish(...)` | ✅ Phase 8 |
| `nostr_relay_subscribe(...)` | `nostrRelaySubscribe(...)` | ✅ Phase 8 |
| `nostr_relay_drain(...)` | `nostrRelayDrain(...)` | ✅ Phase 8 |
| `nostr_relay_unsubscribe(...)` | `nostrRelayUnsubscribe(...)` | ✅ Phase 8 |

### Implemented in Rust, NOT Yet Exposed to Swift

- **IPFS Node** — `add_bytes`, `cat`, `pin_*`, `block_*`, `dht_*`, `name_*`
- **Git** — `git_clone`, `Repository` handle

---

## Phases & Required Xcode Work

### Phase 1: Nostr Relay Read Path ✅ (Rust Done)

**Rust changes in `xcode/rustylib/src/lib.rs` needed:**

Expose the new relay functions to Swift:

```rust
#[uniffi::export]
pub fn nostr_relay_connect(url: &str) -> u64 { ... }

#[uniffi::export]
pub fn nostr_relay_close(handle: u64) -> bool { ... }

#[uniffi::export]
pub fn nostr_relay_publish(handle: u64, event_json: &str) -> bool { ... }

#[uniffi::export]
pub fn nostr_relay_subscribe(handle: u64, filter_json: &str) -> u64 { ... }

#[uniffi::export]
pub fn nostr_relay_drain(sub_handle: u64) -> Option<String> { ... }

#[uniffi::export]
pub fn nostr_relay_unsubscribe(sub_handle: u64) -> bool { ... }
```

**SwiftUI implications:**
- Add a "Nostr" tab or section
- Relay connection manager (URL → handle map)
- Subscription manager (handle → UI list)
- Event list view (parse JSON, display content/kind/pubkey)

**Build impact:** None — `build.sh` automatically regenerates Swift bindings when `rustylib` changes.

---

### Phase 2: GossipSub Multi-Topic + Nostr Bridge

**Rust changes in `xcode/rustylib/src/lib.rs` needed:**

```rust
#[uniffi::export]
pub fn p2p_gossip_join(topic: &str) -> bool { ... }

#[uniffi::export]
pub fn p2p_gossip_leave(topic: &str) -> bool { ... }

#[uniffi::export]
pub fn p2p_gossip_publish_nostr(event_json: &str, topic: Option<String>) -> bool { ... }

#[uniffi::export]
pub fn p2p_gossip_drain_nostr() -> Vec<String> { ... } // Vec<event JSON>
```

**SwiftUI implications:**
- Topic picker / manager
- Nostr event composer that can publish to both relay AND gossip
- Unified event feed (merge relay events + gossip events)
- Event verification in Swift (or trust Rust-side verification)

---

### Phase 3: IPFS ↔ Nostr Content Addressing

**Rust changes in `xcode/rustylib/src/lib.rs` needed:**

```rust
#[uniffi::export]
pub fn ipfs_node_start(online: bool) -> bool { ... } // lazy singleton like P2P_HOST

#[uniffi::export]
pub fn ipfs_add(data: &[u8]) -> String { ... } // returns CID

#[uniffi::export]
pub fn ipfs_cat(cid: &str) -> Vec<u8> { ... }

#[uniffi::export]
pub fn ipfs_pin_add(cid: &str, recursive: bool) -> bool { ... }

#[uniffi::export]
pub fn ipfs_pin_ls() -> Vec<(String, String)> { ... }

#[uniffi::export]
pub fn create_nip94_event(file_path: &str, mime_type: Option<String>) -> String { ... }
```

**SwiftUI implications:**
- File picker → "Add to IPFS" → get CID
- IPFS browser (list pinned CIDs, cat content)
- NIP-94 event composer (attach IPFS file → generate event JSON)
- `ipfs://<cid>` URL handler in SwiftUI

---

### Phase 7: Multi-Topic GossipSub ✅ (Rust Done)

**Go FFI:** `kubo_libp2p_host_gossip_join`, `gossip_leave`, `gossip_publish_to`

**Rust FFI:** `host_gossip_join`, `host_gossip_leave`, `host_gossip_publish_to`

**Safe API:** `Host::gossip_join`, `Host::gossip_leave`, `Host::gossip_publish_to`

**Swift exposure:** `p2pGossipJoin`, `p2pGossipLeave`, `p2pGossipPublishTo`

---

### Phase 8: Swift UniFFI Exposure ✅ (Rust Done)

All HybridNode, multi-topic, and Nostr methods exposed via `#[uniffi::export]` in `xcode/rustylib/src/lib.rs`.

---

### Phase 9: NIP-94 IPFS Resolution ✅ (Rust Done)

**Safe API:** `HybridNode::resolve_nip94(event_json)` → parses kind 1063, extracts `ipfs://` CID, fetches content.

**Swift exposure:** `hybridResolveNip94`

---

### Phase 10: NIP-34 Git over GossipSub ✅ (Rust Done)

**Safe API:**
- `HybridNode::publish_repo(...)` — repo → announcement (kind 30617) → broadcast
- `HybridNode::publish_patch(...)` — diff → patch event (kind 1617) → broadcast
- `HybridNode::publish_issue(...)` — issue event (kind 1621) → broadcast

**Swift exposure:** `hybridPublishRepo`, `hybridPublishPatch`, `hybridPublishIssue`

---

### Phase 11: Swift UniFFI Update ✅ (Rust Done)

Exposed Phase 9 + Phase 10 methods to Swift.

---

### Phase 12: IPFS Node API in Swift ✅ (Done)

**Goal:** Expose the full IPFS Node API to Swift so the SwiftUI app can add/cat/pin/dht/name without going through the Rust CLI.

**Status:** All functions exposed in `xcode/rustylib/src/lib.rs` and wired into `HybridNodeStore`.

**SwiftUI:** Overview tab has IPFS Add, IPFS Cat, Pin Management, and Block Operations cards. Network tab has DHT FindPeer, FindProvs, IPNS Publish, and IPNS Resolve.

---

### Phase 13: Git API in Swift ✅ (Done)

**Goal:** Expose Git operations to Swift for NIP-34 repo management.

**Status:** `git_clone`, `git_init`, `git_head`, `git_branches`, `git_remotes`, `git_status`, `git_commit_message`, `git_diff_trees` all exposed via UniFFI.

**SwiftUI:** Repository tab has Git Init, Git Clone, Repo State, Commit Lookup, and Diff Trees cards.

---

### Phase 14: Error Handling Improvement ✅ (Done)

**Goal:** Replace `bool` / empty-string error discarding with proper `throws` in Swift.

**Status:** `RustyError` enum defined with `#[derive(uniffi::Error)]`. Throwing variants: `hybrid_start_try`, `hybrid_stop_try`, `ipfs_add_try`, `ipfs_cat_try`, `p2p_connect_try`, `hybrid_publish_file_try`.

**SwiftUI:** `HybridNodeStore` uses `do/try/catch` and appends error descriptions to the activity log.

---

### Phase 15: NIP-34 P2P GossipSub Message Types ✅ (Done)

**Goal:** Define typed GossipSub messages for NIP-34 so peers can route git events intelligently.

**Status:** `HybridMessage` record and `MessageCategory` enum exposed via UniFFI. `HybridNode::broadcast_typed` and `drain_typed` implemented. Swift bindings regenerated with macOS slice.

**Rust:**
- `src/p2p_messages.rs` — `HybridMessage` { kind, content, tags } + `MessageCategory` enum
- `src/hybrid.rs` — `broadcast_typed(msg, sk, relay, topic)` and `drain_typed()`
- `xcode/rustylib/src/lib.rs` — `hybrid_broadcast_typed`, `hybrid_drain_typed` wrappers

**SwiftUI:** ✅ Done — Typed Messages card in Chat tab with category picker, dynamic input fields per type (File/Repo/Patch/Issue), Broadcast + Drain buttons, and filtered message list with category badges.

---

### Phase 16: NIP-94 File Flow in SwiftUI ✅

**Goal:** End-to-end NIP-94 workflow: pick a file → add to IPFS → construct NIP-94 event → sign → broadcast to relay + gossip.

**Status:** Complete. Overview tab has "Publish File" card with `.fileImporter`, CID display, and "Resolve NIP-94" resolver card.

**SwiftUI:**
- Overview tab: "Publish File" card with `.fileImporter`
- Display returned CID + signed NIP-94 event JSON
- "Resolve NIP-94" card: paste event JSON → call `hybridResolveNip94` → display fetched content

---

### Phase 17: NIP-34 Git Flow in SwiftUI ✅

**Goal:** Publish repo announcements, patches, and issues directly from the Repository tab.

**Status:** Complete. Repository tab has Publish Repo/Patch/Issue cards wired to UniFFI.

**SwiftUI:**
- Repository tab: "Publish Repo" button → uses current `gitPath` → `hybridPublishRepo`
- Repository tab: "Publish Patch" button → select two commits from `commitHistory` → `hybridPublishPatch`
- Repository tab: "Publish Issue" button → title/body form → `hybridPublishIssue`

---

### Phase 18: Unified Event Inbox ✅

**Goal:** Merge relay + gossip events into a single unified feed with routing.

**Status:** Complete. Inbox card in Network > Nostr with category filter, unread badge, and deduplication.

**SwiftUI:**
- Network > Nostr: Inbox section with periodic `drainTyped` poll
- Route `kind:1617` (Patch) messages to "Code Review" view
- Route `kind:1621` (Issue) messages to "Issue Tracker" view
- Route `kind:30617` (Repo) messages to "Repo Discovery" view
- Deduplicate by Nostr event `id`

---

### Phase 19: Settings Tab ✅

**Goal:** Configuration surface for relays, topics, node state, and identity.

**Status:** Complete. Gear icon nav item with Identity, Node Control, Relay, and GossipSub Topic cards.

**SwiftUI:**
- `DashboardSection.settings` in sidebar with gear icon
- Relay URL list (add/remove)
- GossipSub topic subscriptions (join/leave)
- Node online/offline toggle
- Nostr public key display

---

### Phase 20: Background Polling & Real-Time Updates ✅

**Goal:** Automatic event draining without manual button presses.

**Status:** Complete. 5-second auto-drain loop with Settings toggle and real-time badge counts.

**SwiftUI:**
- `Task` loop in `HybridNodeStore` periodically calls `drainRelay`/`drainGossip`
- Debounced UI updates (every 5 seconds)
- Badge counts on Network/Chat sidebar items showing unread events
- Toggle in Settings

---

### Phase 21: Code Review View ✅

**Goal:** Dedicated view for incoming `kind:1617` (Patch) messages with diff display.

**Status:** Complete. `CodeReviewView.swift` with patch list, diff display, approve/reject.

---

### Phase 22: Issue Tracker View ✅

**Goal:** Dedicated view for incoming `kind:1621` (Issue) messages.

**Status:** Complete. `IssueTrackerView.swift` with status tracking.

---

### Phase 23: Repo Discovery View ✅

**Goal:** Dedicated view for incoming `kind:30617` (Repo) announcements.

**Status:** Complete. `RepoDiscoveryView.swift` with clone/subscribe actions.

---

### Phase 24: Multiple Relay Management ✅

**Goal:** Full multi-relay support with status indicators and failover.

**Status:** Complete. `RelayEntry` model, multi-relay UI in Settings, `drainAllRelays()`.

---

### Phase 25: GossipSub Multi-Topic UI ✅

**Goal:** Full topic management and message filtering.

**Status:** Complete. `TopicEntry` model, join/leave/remove UI in Settings.

---

### Phase 26: Nostr QR Code ✅

**Goal:** Display and scan Nostr public keys as QR codes.

**Status:** Complete. `generateQRCode()` helper, QR code in Settings > Identity.

---

### Phase 27: Real-time Sidebar Badges ✅

**Goal:** Full badge system for unread counts and mentions.

**Status:** Complete. Red badge pills on sidebar, auto-clear on navigation.

---

### Phase 4: HybridNode Wrapper ✅ (Rust Done)

**Rust implementation:** `src/hybrid.rs` + `pub use hybrid::HybridNode` in `src/lib.rs`

`HybridNode` owns both `Node` and `Host`:

```rust
pub struct HybridNode {
    pub ipfs: Node,
    pub p2p: Host,
}

impl HybridNode {
    pub fn start(repo_path: P, online: bool) -> Result<Self, Error>
    pub fn broadcast_event(&self, event_json: &str, relay_handle: Option<u64>, gossip_topic: Option<&str>) -> Result<(), Error>
    pub fn drain_events(&self, relay_sub_handle: Option<u64>) -> Result<Vec<String>, Error>
    pub fn publish_file(&self, file_path: P, secret_key: &str, relay_handle: Option<u64>, gossip_topic: Option<&str>) -> Result<String, Error>
    pub fn stop(self) -> Result<(), Error>
}
```

**Swift exposure needed:**
```rust
#[uniffi::export]
pub fn hybrid_start(online: bool) -> bool { ... }

#[uniffi::export]
pub fn hybrid_broadcast_event(event_json: &str, relay_handle: Option<u64>, gossip_topic: Option<String>) -> bool { ... }

#[uniffi::export]
pub fn hybrid_drain_events(relay_sub_handle: Option<u64>) -> Vec<String> { ... }

#[uniffi::export]
pub fn hybrid_publish_file(file_path: &str, secret_key: &str, relay_handle: Option<u64>, gossip_topic: Option<String>) -> String { ... }
```

**SwiftUI implications:**
- Single "Hybrid" mode toggle
- Unified compose view: text + optional file attachment
- Broadcast button publishes to all configured transports
- Unified inbox (relay + gossip merged, deduplicated by event id)

---

### Phase 5: Example App (`examples/hybrid.rs`) ✅ (Rust Done)

Rust CLI example at `examples/hybrid.rs` demonstrates:

1. Start HybridNode
2. Generate Nostr key
3. Pick a file → Add to IPFS → CID
4. Construct NIP-94 event
5. Sign event
6. Publish to gossip topic
7. Drain events back, verify IPFS round-trip

No Xcode work required for Phase 5, but the SwiftUI app should be modeled after this flow.

---

## Build & CI Checklist for Each Phase

When adding new `#[uniffi::export]` functions to `xcode/rustylib/src/lib.rs`:

- [ ] **Build XCFramework**: `cd xcode && make rust` (or let Xcode build phase handle it)
- [ ] **Verify Swift bindings**: Check `xcode/rustylib/out/rustylib.swift` for new functions
- [ ] **Clean build in Xcode**: Product → Clean Build Folder, then build
- [ ] **Test on device/simulator**: Run `kubo-macos` scheme or iOS Simulator
- [ ] **Xcode Cloud**: Push to trigger CI; verify `ci_post_clone.sh` still builds XCFramework before SPM resolve

---

## Design Decisions Needed

### 1. Singleton vs. Instance Handles

Current pattern: `P2P_HOST` is a global `Mutex<Option<Host>>`. This is simple for Swift but limits you to one host.

For `Node` and `HybridNode`, we should probably use the same singleton pattern initially. If the app ever needs multiple nodes, switch to handle-based registries (like Nostr relays).

### 2. Error Handling in Swift

UniFFI maps Rust `Result<T, Error>` to Swift `throws`. The current `rustylib` code often discards errors (returns `bool` or empty string). Consider changing to `throws` for critical operations:

```rust
#[uniffi::export]
pub fn ipfs_add(data: Vec<u8>) -> Result<String, RustyError> { ... }
```

This requires adding an error type:
```rust
#[derive(uniffi::Error)]
pub enum RustyError { ... }
```

### 3. Background Threads

IPFS node operations (DHT, Bitswap) can block. The SwiftUI app should call Rust functions on background threads (e.g., `Task { ... }`) and update UI on main thread.

Current `rustylib` functions are synchronous. Consider adding async wrappers if SwiftUI becomes unresponsive.

### 4. Data Types

UniFFI supports:
- Primitives (`u32`, `i64`, `bool`, `String`)
- `Vec<T>` and `Option<T>`
- Records (structs with named fields)
- Enums

For complex types like `NostrEvent`, define a UniFFI record:

```rust
#[derive(uniffi::Record)]
pub struct NostrEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: i32,
    pub content: String,
    pub tags: Vec<Vec<String>>,
    pub sig: String,
}
```

Then parse JSON in Rust and return structured data to Swift.

---

## Quick Reference: Adding a New Function

1. Add `#[uniffi::export]` function to `xcode/rustylib/src/lib.rs`
2. Call the underlying `kubo_rs::` function
3. Run `cd xcode && make rust` to regenerate Swift bindings
4. Clean build in Xcode
5. Use the new function from Swift

Example:

```rust
// xcode/rustylib/src/lib.rs
#[uniffi::export]
pub fn nostr_generate_key() -> String {
    kubo_rs::nostr_generate_key().unwrap_or_default()
}
```

```swift
// swiftyapp/SomeView.swift
let sk = RustyLib.nostrGenerateKey()
```
