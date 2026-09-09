# Hybrid Protocol — Xcode Integration Notes

This document tracks what needs to happen in the Xcode/Swift layer as the Nostr+IPFS+P2P hybrid protocol evolves in the parent Rust crate.

## Current State (2026-09-09)

The `xcode/rustylib/` crate wraps `kubo-rs` and exposes a **small subset** of the API to Swift via UniFFI. The SwiftUI app (`swiftyapp/`) consumes these through the `RustyLib` local package.

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

### Phase 12: IPFS Node API in Swift 🔄 (In Progress)

**Goal:** Expose the full IPFS Node API to Swift so the SwiftUI app can add/cat/pin/dht/name without going through the Rust CLI.

**Rust changes in `xcode/rustylib/src/lib.rs`:**

```rust
#[uniffi::export]
pub fn ipfs_add(data: Vec<u8>) -> String { ... }

#[uniffi::export]
pub fn ipfs_cat(cid: &str) -> Vec<u8> { ... }

#[uniffi::export]
pub fn ipfs_pin_add(cid: &str, recursive: bool) -> bool { ... }

#[uniffi::export]
pub fn ipfs_pin_rm(cid: &str, recursive: bool) -> bool { ... }

#[uniffi::export]
pub fn ipfs_pin_ls() -> Vec<(String, String)> { ... }

#[uniffi::export]
pub fn ipfs_block_put(data: Vec<u8>) -> String { ... }

#[uniffi::export]
pub fn ipfs_block_get(cid: &str) -> Vec<u8> { ... }

#[uniffi::export]
pub fn ipfs_block_stat(cid: &str) -> u64 { ... }

#[uniffi::export]
pub fn ipfs_dht_findpeer(peer_id: &str) -> Vec<String> { ... }

#[uniffi::export]
pub fn ipfs_dht_findprovs(cid: &str) -> Vec<String> { ... }

#[uniffi::export]
pub fn ipfs_name_publish(cid: &str, lifetime_sec: i64) -> String { ... }

#[uniffi::export]
pub fn ipfs_name_resolve(name: &str) -> String { ... }
```

**SwiftUI implications:**
- "Files" tab: add files, browse CIDs, cat content
- "Pins" tab: manage pinned objects
- "DHT" tab: peer lookup, provider search
- "IPNS" tab: publish and resolve names

---

### Phase 13: Git API in Swift 🔄 (In Progress)

**Goal:** Expose Git operations to Swift for NIP-34 repo management.

**Rust changes:**
```rust
#[uniffi::export]
pub fn git_clone(url: &str, path: &str, bare: bool) -> bool { ... }

#[uniffi::export]
pub fn git_init(path: &str, bare: bool) -> bool { ... }

#[uniffi::export]
pub fn git_head(path: &str) -> String { ... }

#[uniffi::export]
pub fn git_branches(path: &str) -> Vec<String> { ... }
```

**SwiftUI implications:**
- "Git" tab: clone/init repos, browse branches, view HEAD
- Integration with NIP-34 publish buttons

---

### Phase 14: Error Handling Improvement 🔄 (In Progress)

**Goal:** Replace `bool` / empty-string error discarding with proper `throws` in Swift.

**Rust changes:**
```rust
#[derive(uniffi::Error)]
pub enum RustyError {
    Ipfs { msg: String },
    P2p { msg: String },
    Nostr { msg: String },
    Git { msg: String },
}
```

Then change critical functions to return `Result<T, RustyError>`:
```rust
#[uniffi::export]
pub fn ipfs_add(data: Vec<u8>) -> Result<String, RustyError> { ... }
```

**SwiftUI implications:**
- Use `do/try/catch` in Swift
- Show meaningful error alerts instead of silent failures

---

### Phase 15: NIP-34 P2P GossipSub Message Types 🔄 (In Progress)

**Goal:** Define typed GossipSub messages for NIP-34 so peers can route git events intelligently.

**Problem:** Currently NIP-34 events are broadcast as raw JSON strings. Peers can't filter by message type without parsing JSON.

**Solution:** Define a protocol envelope:

```rust
pub enum NostrMessage {
    Nip94File { cid: String, filename: String, mime: String },
    Nip34Repo { repo_id: String, name: String, clone_urls: Vec<String> },
    Nip34Patch { repo_ref: String, diff: String },
    Nip34Issue { repo_ref: String, title: String, body: String },
    GenericEvent { kind: u16, json: String },
}
```

Serialize with a 1-byte type prefix + CBOR/JSON payload. GossipSub handlers can route by type without full JSON parsing.

**Rust changes:**
- New module: `src/p2p_messages.rs`
- `HybridNode::broadcast_typed(...)` — publishes typed messages
- `HybridNode::drain_typed(...)` — returns `Vec<NostrMessage>`
- Swift exposure: `hybridBroadcastTyped`, `hybridDrainTyped`

**SwiftUI implications:**
- Filter event feed by type (files, repos, patches, issues)
- Route patches to a "Code Review" view
- Route issues to an "Issue Tracker" view

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
