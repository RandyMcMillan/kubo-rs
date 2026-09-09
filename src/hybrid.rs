//! HybridNode — coordinates IPFS (Kubo), libp2p (Host), and Nostr in one handle.

use std::path::Path;

use crate::{
    Error, Host, Node, init_repo, nostr_event_sign_with_tags, nostr_event_verify,
    nostr_relay_drain, nostr_relay_publish,
};

/// A unified handle that owns both an IPFS node and a libp2p host.
///
/// `HybridNode` is the recommended entry point for hybrid-protocol apps.
/// It manages the lifecycles of both subsystems and provides convenience
/// methods that cross the IPFS / P2P / Nostr boundaries.
pub struct HybridNode {
    /// The underlying Kubo IPFS node.
    pub ipfs: Node,
    /// The underlying standalone libp2p host.
    pub p2p: Host,
}

impl HybridNode {
    /// Start a new hybrid node.
    ///
    /// * `repo_path` — filesystem path for the IPFS repo.
    /// * `online` — whether to start libp2p networking.
    ///
    /// # Errors
    ///
    /// Returns an error if the repo cannot be initialised or either subsystem
    /// fails to start.
    pub fn start<P: AsRef<Path>>(repo_path: P, online: bool) -> Result<Self, Error> {
        let path = repo_path.as_ref();
        if !path.join("config").exists() {
            init_repo(path)?;
        }
        let ipfs = Node::start(path, online)?;
        let p2p = Host::new()?;
        Ok(HybridNode { ipfs, p2p })
    }

    /// Publish a signed Nostr event to both a relay and GossipSub.
    ///
    /// * `event_json` — a signed Nostr event JSON string.
    /// * `relay_handle` — optional relay handle (from `nostr_relay_connect`).
    /// * `gossip_topic` — optional GossipSub topic override.
    ///
    /// # Errors
    ///
    /// Returns an error if the event is invalid, relay publish fails, or
    /// gossip publish fails.
    pub fn broadcast_event(
        &self,
        event_json: &str,
        relay_handle: Option<u64>,
        gossip_topic: Option<&str>,
    ) -> Result<(), Error> {
        if !nostr_event_verify(event_json).unwrap_or(false) {
            return Err(Error::InvalidNostrEvent);
        }

        if let Some(handle) = relay_handle {
            nostr_relay_publish(handle, event_json)?;
        }

        if let Some(topic) = gossip_topic {
            self.p2p.gossip_publish_to(topic, event_json)?;
        } else {
            self.p2p.gossip_publish(event_json)?;
        }

        Ok(())
    }

    /// Drain Nostr events from both a relay subscription and GossipSub.
    ///
    /// Events from both sources are merged into a single vector.
    /// Duplicate events (same `id`) are deduplicated.
    ///
    /// # Errors
    ///
    /// Returns an error if either drain operation fails.
    pub fn drain_events(&self, relay_sub_handle: Option<u64>) -> Result<Vec<String>, Error> {
        let mut events = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // Drain from relay subscription
        if let Some(sub) = relay_sub_handle {
            if let Some(evt) = nostr_relay_drain(sub)? {
                if let Some(id) = extract_event_id(&evt) {
                    seen_ids.insert(id);
                }
                events.push(evt);
            }
        }

        // Drain from GossipSub
        for evt in self.p2p.gossip_drain_nostr()? {
            if let Some(id) = extract_event_id(&evt) {
                if seen_ids.insert(id.clone()) {
                    events.push(evt);
                }
            } else {
                events.push(evt);
            }
        }

        Ok(events)
    }

    /// Add a file to IPFS, construct a NIP-94 event, sign it, and broadcast.
    ///
    /// Returns the CID of the added file.
    ///
    /// # Errors
    ///
    /// Returns an error if IPFS add fails, event construction fails,
    /// signing fails, or broadcast fails.
    pub fn publish_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        secret_key: &str,
        relay_handle: Option<u64>,
        gossip_topic: Option<&str>,
    ) -> Result<String, Error> {
        let path = file_path.as_ref();
        let data = std::fs::read(path).map_err(|e| Error::Go(format!("read file: {e}")))?;
        let cid = self.ipfs.add_bytes(&data)?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");

        // NIP-94 tags: url, m (mime), x (sha256 — skipped for brevity)
        let tags: Vec<Vec<String>> = vec![
            vec!["url".to_string(), format!("ipfs://{cid}")],
            vec!["m".to_string(), guess_mime(file_name)],
        ];
        let tags_json =
            serde_json::to_string(&tags).map_err(|e| Error::Go(format!("serialize tags: {e}")))?;

        let event = nostr_event_sign_with_tags(secret_key, file_name, 1063, &tags_json)?;

        self.broadcast_event(&event, relay_handle, gossip_topic)?;

        Ok(cid)
    }

    /// Shut down both subsystems.
    ///
    /// # Errors
    ///
    /// Returns an error if either stop operation fails.
    pub fn stop(self) -> Result<(), Error> {
        self.ipfs.stop()?;
        self.p2p.close()?;
        Ok(())
    }

    /// Parse a NIP-94 file metadata event and fetch the referenced IPFS content.
    ///
    /// Returns `(cid, content)` where `cid` is extracted from the `url` tag
    /// (`ipfs://<cid>`).
    ///
    /// # Errors
    ///
    /// Returns an error if the event is not a valid NIP-94 event, has no
    /// `ipfs://` URL, or the content cannot be fetched.
    pub fn resolve_nip94(&self, event_json: &str) -> Result<(String, Vec<u8>), Error> {
        let event: serde_json::Value = serde_json::from_str(event_json)
            .map_err(|e| Error::Go(format!("invalid JSON: {e}")))?;

        let kind = event.get("kind").and_then(|v| v.as_i64()).unwrap_or(0);
        if kind != 1063 {
            return Err(Error::Go(format!("expected kind 1063, got {kind}")));
        }

        let tags = event.get("tags").and_then(|v| v.as_array());
        let mut cid = None;
        if let Some(tags) = tags {
            for tag in tags {
                if let Some(arr) = tag.as_array() {
                    if arr.len() >= 2 && arr[0].as_str() == Some("url") && arr[1].as_str().is_some()
                    {
                        let url = arr[1].as_str().unwrap();
                        if let Some(stripped) = url.strip_prefix("ipfs://") {
                            cid = Some(stripped.to_string());
                            break;
                        }
                    }
                }
            }
        }

        let cid = cid.ok_or_else(|| Error::Go("no ipfs:// URL found in tags".to_string()))?;
        let content = self.ipfs.cat(&cid)?;
        Ok((cid, content))
    }
}

/// Extract the `id` field from a Nostr event JSON string.
fn extract_event_id(event_json: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(event_json).ok()?;
    parsed.get("id")?.as_str().map(|s| s.to_string())
}

/// Naive MIME-type guess based on file extension.
fn guess_mime(file_name: &str) -> String {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "tar" => "application/x-tar",
        "md" => "text/markdown",
        "rs" => "text/rust",
        "go" => "text/x-go",
        "swift" => "text/x-swift",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let path = PathBuf::from("tmp").join("hybrid-test").join(name);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_hybrid_start_stop() {
        let repo = tmp_dir("start_stop").join("repo");
        let node = HybridNode::start(&repo, false).expect("start should succeed");
        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_hybrid_publish_file_roundtrip() {
        let base = tmp_dir("publish_file");
        let repo = base.join("repo");
        let file = base.join("hello.txt");
        std::fs::write(&file, b"hello hybrid").unwrap();

        let node = HybridNode::start(&repo, false).expect("start should succeed");
        let sk = crate::nostr_generate_key().expect("keygen should succeed");

        let cid = node
            .publish_file(&file, &sk, None, None)
            .expect("publish_file should succeed");
        assert!(!cid.is_empty(), "cid should not be empty");

        // Verify the file is retrievable via IPFS
        let fetched = node.ipfs.cat(&cid).expect("cat should succeed");
        assert_eq!(fetched, b"hello hybrid", "round-trip data should match");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_guess_mime() {
        assert_eq!(guess_mime("photo.png"), "image/png");
        assert_eq!(guess_mime("index.html"), "text/html");
        assert_eq!(guess_mime("unknown.xyz"), "application/octet-stream");
    }

    #[test]
    fn test_extract_event_id() {
        let json = r#"{"id":"abc123","pubkey":"def456","kind":1}"#;
        assert_eq!(extract_event_id(json), Some("abc123".to_string()));
        assert_eq!(extract_event_id("{}"), None);
    }

    #[test]
    fn test_resolve_nip94() {
        let base = tmp_dir("resolve_nip94");
        let repo = base.join("repo");
        let file = base.join("test.txt");
        std::fs::write(&file, b"nip94 content").unwrap();

        let node = HybridNode::start(&repo, false).expect("start should succeed");
        let sk = crate::nostr_generate_key().expect("keygen should succeed");

        let cid = node
            .publish_file(&file, &sk, None, None)
            .expect("publish_file should succeed");

        // Build a synthetic NIP-94 event JSON
        let event = format!(
            r#"{{"id":"test","pubkey":"pk","created_at":0,"kind":1063,"tags":[["url","ipfs://{}"],["m","text/plain"]],"content":"test.txt","sig":"sig"}}"#,
            cid
        );

        let (resolved_cid, content) = node
            .resolve_nip94(&event)
            .expect("resolve_nip94 should succeed");
        assert_eq!(resolved_cid, cid);
        assert_eq!(content, b"nip94 content");

        node.stop().expect("stop should succeed");
    }
}
