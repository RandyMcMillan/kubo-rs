//! Typed GossipSub messages for Nostr + IPFS + Git hybrid protocol.
//!
//! This module provides typed constructors and parsers for Nostr events
//! that flow over GossipSub. Instead of inventing a new wire format, we
//! use standard Nostr event JSON — the `kind` field already discriminates
//! message types (1063=NIP-94, 30617=NIP-34 repo, 1617=NIP-34 patch,
//! 1621=NIP-34 issue). The helpers here make it easier to build and
//! filter these events in Rust.

use serde::{Deserialize, Serialize};

/// A typed protocol message representing a Nostr event that can be
/// broadcast over GossipSub (or any other transport).
///
/// This is a convenience wrapper around raw Nostr event JSON. The
/// `kind` field is the primary discriminator. Use `to_event_json()`
/// to produce a signed event string ready for `broadcast_event()`,
/// and `from_event_json()` to parse a received event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridMessage {
    pub kind: u16,
    pub content: String,
    pub tags: Vec<Vec<String>>,
}

impl HybridMessage {
    /// Build a NIP-94 file metadata message (unsigned template).
    pub fn nip94_file(cid: &str, filename: &str, mime: &str) -> Self {
        HybridMessage {
            kind: 1063,
            content: filename.to_string(),
            tags: vec![
                vec!["url".to_string(), format!("ipfs://{cid}")],
                vec!["m".to_string(), mime.to_string()],
            ],
        }
    }

    /// Build a NIP-34 repository announcement message (unsigned template).
    pub fn nip34_repo(
        repo_id: &str,
        name: &str,
        description: &str,
        clone_urls: &[String],
    ) -> Self {
        let mut tags = vec![
            vec!["d".to_string(), repo_id.to_string()],
            vec!["name".to_string(), name.to_string()],
        ];
        if !description.is_empty() {
            tags.push(vec!["description".to_string(), description.to_string()]);
        }
        for url in clone_urls {
            tags.push(vec!["clone".to_string(), url.clone()]);
        }
        HybridMessage {
            kind: 30617,
            content: description.to_string(),
            tags,
        }
    }

    /// Build a NIP-34 patch message (unsigned template).
    pub fn nip34_patch(repo_ref: &str, diff: &str) -> Self {
        HybridMessage {
            kind: 1617,
            content: diff.to_string(),
            tags: vec![
                vec!["e".to_string(), repo_ref.to_string()],
                vec!["a".to_string(), format!("{repo_ref}")],
            ],
        }
    }

    /// Build a NIP-34 issue message (unsigned template).
    pub fn nip34_issue(repo_ref: &str, title: &str, body: &str) -> Self {
        HybridMessage {
            kind: 1621,
            content: format!("{title}\n\n{body}"),
            tags: vec![
                vec!["e".to_string(), repo_ref.to_string()],
                vec!["a".to_string(), format!("{repo_ref}")],
            ],
        }
    }

    /// Serialize to an unsigned Nostr event JSON template.
    ///
    /// Use `kubo_rs::nostr_event_sign_with_tags()` or similar to sign
    /// before broadcasting.
    pub fn to_unsigned_json(&self) -> Result<String, serde_json::Error> {
        let value = serde_json::json!({
            "kind": self.kind,
            "content": self.content,
            "tags": self.tags,
            "created_at": 0,
        });
        serde_json::to_string(&value)
    }

    /// Parse from a signed Nostr event JSON string.
    pub fn from_event_json(event_json: &str) -> Result<Self, String> {
        let value: serde_json::Value =
            serde_json::from_str(event_json).map_err(|e| e.to_string())?;
        let kind = value
            .get("kind")
            .and_then(|v| v.as_u64())
            .ok_or("missing kind")? as u16;
        let content = value
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let tags = value
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|tag| tag.as_array())
                    .map(|tag| {
                        tag.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(HybridMessage {
            kind,
            content,
            tags,
        })
    }

    /// Return the message category for routing.
    pub fn category(&self) -> MessageCategory {
        match self.kind {
            1063 => MessageCategory::File,
            30617 => MessageCategory::Repo,
            1617 => MessageCategory::Patch,
            1621 => MessageCategory::Issue,
            _ => MessageCategory::Other,
        }
    }
}

/// High-level message category for filtering and routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageCategory {
    File,
    Repo,
    Patch,
    Issue,
    Other,
}

/// Extract the Nostr event `kind` from a JSON string without full parsing.
pub fn extract_event_kind(event_json: &str) -> Option<u16> {
    let value: serde_json::Value = serde_json::from_str(event_json).ok()?;
    value.get("kind")?.as_u64().map(|k| k as u16)
}

/// Filter a list of event JSON strings by Nostr kind.
pub fn filter_by_kind(event_jsons: Vec<String>, kind: u16) -> Vec<String> {
    event_jsons
        .into_iter()
        .filter(|evt| extract_event_kind(evt) == Some(kind))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nip94_file_roundtrip() {
        let msg = HybridMessage::nip94_file("Qm123", "test.txt", "text/plain");
        assert_eq!(msg.kind, 1063);
        assert_eq!(msg.category(), MessageCategory::File);
        let json = msg.to_unsigned_json().unwrap();
        assert!(json.contains("Qm123"));
    }

    #[test]
    fn nip34_repo_roundtrip() {
        let msg = HybridMessage::nip34_repo("myrepo", "My Repo", "A repo", &["http://a.git".to_string()]);
        assert_eq!(msg.kind, 30617);
        assert_eq!(msg.category(), MessageCategory::Repo);
        let json = msg.to_unsigned_json().unwrap();
        assert!(json.contains("myrepo"));
    }

    #[test]
    fn nip34_patch_roundtrip() {
        let msg = HybridMessage::nip34_patch("repo/123", "diff --git a/foo b/foo\n+bar");
        assert_eq!(msg.kind, 1617);
        assert_eq!(msg.category(), MessageCategory::Patch);
    }

    #[test]
    fn nip34_issue_roundtrip() {
        let msg = HybridMessage::nip34_issue("repo/123", "Bug", "It broke");
        assert_eq!(msg.kind, 1621);
        assert_eq!(msg.category(), MessageCategory::Issue);
    }

    #[test]
    fn extract_kind_works() {
        let json = r#"{"id":"a","kind":1063,"content":"x"}"#;
        assert_eq!(extract_event_kind(json), Some(1063));
    }

    #[test]
    fn filter_by_kind_works() {
        let events = vec![
            r#"{"kind":1,"content":"hello"}"#.to_string(),
            r#"{"kind":1063,"content":"file"}"#.to_string(),
            r#"{"kind":1,"content":"world"}"#.to_string(),
        ];
        let filtered = filter_by_kind(events, 1063);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].contains("file"));
    }
}
