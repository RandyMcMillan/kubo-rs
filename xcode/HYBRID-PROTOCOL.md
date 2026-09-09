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

### Implemented in Rust, NOT Yet Exposed to Swift

- **IPFS Node** — `Node::start`, `add_bytes`, `cat`, `pin_*`, `block_*`, `dht_*`, `name_*`
- **Nostr** — keygen, sign, verify, relay connect/publish/subscribe/drain
- **Git** — `git_clone`, `Repository` handle
- **HybridNode** — `HybridNode::start`, `broadcast_event`, `drain_events`, `publish_file` (Phase 4 ✅)
- **examples/hybrid.rs** — full end-to-end Rust demo (Phase 5 ✅)

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
