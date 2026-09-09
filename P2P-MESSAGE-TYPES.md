# P2P Nostr Message Types

This document defines how Nostr events are transported over the libp2p GossipSub layer in the kubo-rs hybrid protocol.

## Message Format

GossipSub messages in kubo-rs use a **pipe-delimited envelope**:

```
topic|sender_peer_id|payload
```

| Field | Description |
|-------|-------------|
| `topic` | The GossipSub topic name (e.g. `kubo-desktop`, `kubo-hybrid`) |
| `sender_peer_id` | Base58-encoded libp2p peer ID of the sender |
| `payload` | The opaque message body |

This format is produced by `go/ffi/libp2p.go` and consumed by `Host::gossip_drain()`.

---

## Nostr Event Payload

When the payload is a **valid Nostr event JSON**, it is considered a *Nostr message*. No additional envelope wrapping is required — the payload IS the event.

### Example

```
kubo-hybrid|12D3KooWL5E3WhKhee5rt6JfWjdMTa4qQ4siJBXkGJvYesMrss5w|{"id":"abc...","pubkey":"def...","created_at":1695230400,"kind":30617,"tags":[["d","my-repo"],["clone","https://example.com/repo.git"]],"content":"","sig":"123..."}
```

### Filtering

`Host::gossip_drain_nostr()` filters raw gossip messages and returns only payloads that pass `nostr_event_verify()`:

```rust
let events: Vec<String> = host.gossip_drain_nostr()?;
// Each String is a verified Nostr event JSON.
```

Invalid JSON, malformed events, or bad signatures are silently dropped.

---

## Supported Event Kinds

The hybrid protocol recognizes all standard Nostr kinds. The following are particularly relevant for IPFS/P2P integration:

| Kind | Name | Usage in Hybrid Protocol |
|------|------|--------------------------|
| 1 | Text Note | General chat/messaging over gossip |
| 1063 | File Metadata (NIP-94) | Reference IPFS-hosted files via `ipfs://<cid>` URLs |
| 1617 | Git Patch (NIP-34) | Share code patches peer-to-peer |
| 1621 | Git Issue (NIP-34) | Issue tracking without central server |
| 30617 | Git Repo Announcement (NIP-34) | Announce a repo with clone URLs |
| 30618 | Git Repo State (NIP-34) | Publish HEAD + branch refs |

---

## NIP-34 (Git) over GossipSub

### Repo Announcement → Gossip

```rust
let repo = Repository::open("/path/to/repo")?;
let sk = nostr_generate_key()?;

let event = repo.create_nip34_announcement(
    "my-project",
    "My Project",
    "A decentralized project",
    &["ipfs://Qm...", "https://github.com/user/repo"],
    &sk,
)?;

host.gossip_publish(&event)?;
```

### Patch → Gossip

```rust
let diff = repo.diff_trees("old_hash", "new_hash")?;
let event = nip34_patch(&sk, "30617:<pk>:my-project", &diff)?;
host.gossip_publish(&event)?;
```

### Receiving → IPFS Resolution

```rust
for event_json in host.gossip_drain_nostr()? {
    // Parse event, extract ipfs:// URLs from tags
    // Fetch content via Node::cat(cid)
}
```

---

## Multi-Topic Routing (Planned)

In Phase 2, `Host` will support joining multiple topics:

```rust
host.gossip_join("kubo-hybrid")?;
host.gossip_join("nip34-patches")?;
```

Each topic carries the same pipe-delimited format. `gossip_drain_nostr()` will verify events from ALL joined topics.

---

## Versioning

The current format is **version 1** (implicit). If the envelope format changes in the future, the version will be embedded as a prefix:

```
v2|topic|sender|payload
```

`gossip_drain()` and `gossip_drain_nostr()` will detect and handle versioned envelopes.
