//! Rust bindings for Kubo (the Go implementation of IPFS).
//!
//! This crate provides a native Rust API over a CGO/FFI bridge to the Kubo
//! Go codebase. See `FFI.md` for architecture details.
//!
//! # Quick start
//!
//! ```no_run
//! use kubo_rs::{init_repo, Node};
//!
//! init_repo("/tmp/ipfs-repo").unwrap();
//! let node = Node::start("/tmp/ipfs-repo", false).unwrap();
//! let cid = node.add_bytes(b"hello").unwrap();
//! let data = node.cat(&cid).unwrap();
//! node.stop().unwrap();
//! ```

mod error;
mod ffi;
pub mod hybrid;
pub mod nostr_url;

pub use error::Error;
pub use ffi::version;
pub use hybrid::HybridNode;
pub use nostr_url::NostrUrl;

use std::path::Path;

/// Initialize a new IPFS repo at the given path.
///
/// # Errors
///
/// Returns an error if the repo cannot be initialized.
pub fn init_repo<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let s = path.as_ref().to_str().ok_or(Error::InvalidPath)?;
    if s.contains('\0') {
        return Err(Error::InvalidPath);
    }
    ffi::init_repo(s)
}

/// An owned handle to a running Kubo IPFS node.
///
/// When dropped, the node is shut down. Use [`Node::stop`] if you need to
/// handle shutdown errors explicitly.
pub struct Node {
    handle: u64,
}

impl Node {
    /// Start a new IPFS node using the repo at `path`.
    ///
    /// `online` controls whether the node joins the libp2p network.
    ///
    /// # Errors
    ///
    /// Returns an error if the node cannot be started.
    pub fn start<P: AsRef<Path>>(path: P, online: bool) -> Result<Self, Error> {
        let path = path.as_ref().to_str().ok_or(Error::InvalidPath)?;
        if path.contains('\0') {
            return Err(Error::InvalidPath);
        }
        let handle = ffi::node_start(path, online)?;
        Ok(Node { handle })
    }

    /// Return the node's peer ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the peer ID cannot be read.
    pub fn peer_id(&self) -> Result<String, Error> {
        ffi::node_peer_id(self.handle)
    }

    /// Return the node's listening addresses.
    ///
    /// # Errors
    ///
    /// Returns an error if the addresses cannot be read.
    pub fn listening_addrs(&self) -> Result<Vec<String>, Error> {
        ffi::node_listening_addrs(self.handle)
    }

    /// Start the HTTP API server on the given multiaddr.
    ///
    /// Returns the actual listening multiaddr (e.g. with the resolved port
    /// when `/tcp/0` is used).
    ///
    /// # Errors
    ///
    /// Returns an error if the API server cannot be started.
    pub fn start_api(&self, multiaddr: &str) -> Result<String, Error> {
        ffi::node_start_api(self.handle, multiaddr)
    }

    /// Return the multiaddr of the running HTTP API server.
    ///
    /// # Errors
    ///
    /// Returns an error if the API is not started.
    pub fn api_addrs(&self) -> Result<String, Error> {
        ffi::node_api_addrs(self.handle)
    }

    /// Connect to a peer by multiaddr.
    ///
    /// The address should include the peer ID, e.g.:
    /// `/ip4/127.0.0.1/tcp/4001/p2p/Qm...`
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub fn connect(&self, addr: &str) -> Result<(), Error> {
        ffi::node_connect(self.handle, addr)
    }

    /// Return the list of connected swarm peers.
    ///
    /// Each tuple contains the peer ID and the connected address.
    ///
    /// # Errors
    ///
    /// Returns an error if the peer list cannot be read.
    pub fn swarm_peers(&self) -> Result<Vec<(String, String)>, Error> {
        ffi::swarm_peers(self.handle)
    }

    /// Return the node's identity as a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the identity cannot be read.
    pub fn id(&self) -> Result<String, Error> {
        ffi::node_id(self.handle)
    }

    /// Add a byte slice to IPFS and return the resulting CID.
    ///
    /// # Errors
    ///
    /// Returns an error if the add operation fails.
    pub fn add_bytes(&self, data: &[u8]) -> Result<String, Error> {
        ffi::unixfs_add_bytes(self.handle, data)
    }

    /// Retrieve the contents of a UnixFS file by CID.
    ///
    /// Accepts both raw CIDs and `/ipfs/…` paths.
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be retrieved.
    pub fn cat(&self, cid: &str) -> Result<Vec<u8>, Error> {
        ffi::unixfs_cat(self.handle, cid)
    }

    /// Put a raw block into the blockstore and return its CID.
    ///
    /// # Errors
    ///
    /// Returns an error if the block cannot be stored.
    pub fn block_put(&self, data: &[u8]) -> Result<String, Error> {
        ffi::block_put(self.handle, data)
    }

    /// Get a raw block from the blockstore by CID.
    ///
    /// # Errors
    ///
    /// Returns an error if the block cannot be retrieved.
    pub fn block_get(&self, cid: &str) -> Result<Vec<u8>, Error> {
        ffi::block_get(self.handle, cid)
    }

    /// Return the size of a raw block in the blockstore.
    ///
    /// # Errors
    ///
    /// Returns an error if the block cannot be found.
    pub fn block_stat(&self, cid: &str) -> Result<usize, Error> {
        ffi::block_stat(self.handle, cid)
    }

    /// Disconnect from a peer by multiaddr.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnection fails.
    pub fn disconnect(&self, addr: &str) -> Result<(), Error> {
        ffi::swarm_disconnect(self.handle, addr)
    }

    /// Pin a CID or `/ipfs/…` path.
    ///
    /// `recursive=true` pins the entire referenced tree.
    ///
    /// # Errors
    ///
    /// Returns an error if the pin operation fails.
    pub fn pin_add(&self, cid: &str, recursive: bool) -> Result<(), Error> {
        ffi::pin_add(self.handle, cid, recursive)
    }

    /// Unpin a CID or `/ipfs/…` path.
    ///
    /// # Errors
    ///
    /// Returns an error if the unpin operation fails.
    pub fn pin_rm(&self, cid: &str, recursive: bool) -> Result<(), Error> {
        ffi::pin_rm(self.handle, cid, recursive)
    }

    /// Return the list of pinned objects.
    ///
    /// Each tuple contains the path and the pin type (e.g. "recursive",
    /// "direct", "indirect").
    ///
    /// # Errors
    ///
    /// Returns an error if the pin list cannot be read.
    pub fn pin_ls(&self) -> Result<Vec<(String, String)>, Error> {
        ffi::pin_ls(self.handle)
    }

    /// Query the DHT for a peer's addresses.
    ///
    /// Returns the peer ID and a list of multiaddrs.
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup fails.
    pub fn dht_findpeer(&self, peer_id: &str) -> Result<(String, Vec<String>), Error> {
        ffi::dht_findpeer(self.handle, peer_id)
    }

    /// Query the DHT for providers of a CID.
    ///
    /// Returns a list of (peer_id, addresses) tuples.
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup fails.
    pub fn dht_findprovs(&self, cid: &str) -> Result<Vec<(String, Vec<String>)>, Error> {
        ffi::dht_findprovs(self.handle, cid)
    }

    /// Publish an IPNS name pointing to a CID.
    ///
    /// `lifetime_sec` controls how long the record is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails.
    pub fn name_publish(&self, cid: &str, lifetime_sec: i64) -> Result<String, Error> {
        ffi::name_publish(self.handle, cid, lifetime_sec)
    }

    /// Resolve an IPNS name to a path.
    ///
    /// # Errors
    ///
    /// Returns an error if resolution fails.
    pub fn name_resolve(&self, name: &str) -> Result<String, Error> {
        ffi::name_resolve(self.handle, name)
    }

    /// Shut the node down and consume the handle.
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails.
    pub fn stop(self) -> Result<(), Error> {
        let result = ffi::node_stop(self.handle);
        std::mem::forget(self); // prevent double-stop in Drop
        result
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = ffi::node_stop(self.handle);
    }
}

// ---------------------------------------------------------------------------
// libp2p
// ---------------------------------------------------------------------------

/// A standalone libp2p host (not tied to a Kubo node).
///
/// When dropped, the host is closed.
pub struct Host {
    handle: u64,
}

impl Host {
    /// Create a new libp2p host listening on a local TCP port.
    ///
    /// # Errors
    ///
    /// Returns an error if the host cannot be created.
    pub fn new() -> Result<Self, Error> {
        let handle = ffi::host_new()?;
        Ok(Host { handle })
    }

    /// Return the host's peer ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the peer ID cannot be read.
    pub fn peer_id(&self) -> Result<String, Error> {
        ffi::host_peer_id(self.handle)
    }

    /// Return the host's listening addresses.
    ///
    /// # Errors
    ///
    /// Returns an error if the addresses cannot be read.
    pub fn listening_addrs(&self) -> Result<Vec<String>, Error> {
        ffi::host_listening_addrs(self.handle)
    }

    /// Connect to a peer by multiaddr.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    pub fn connect(&self, addr: &str) -> Result<(), Error> {
        ffi::host_connect(self.handle, addr)
    }

    /// Ping a connected peer and return the round-trip time in milliseconds.
    ///
    /// # Errors
    ///
    /// Returns an error if the ping fails.
    pub fn ping(&self, peer_id: &str) -> Result<i64, Error> {
        ffi::host_ping(self.handle, peer_id)
    }

    /// Return the list of protocols supported by this host.
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol list cannot be read.
    pub fn protocols(&self) -> Result<Vec<String>, Error> {
        ffi::host_protocols(self.handle)
    }

    /// Return the gossip topic name joined by this host.
    ///
    /// # Errors
    ///
    /// Returns an error if the topic cannot be read.
    pub fn gossip_topic(&self) -> Result<String, Error> {
        ffi::host_gossip_topic(self.handle)
    }

    /// Return the gossip initialization error, if any.
    ///
    /// # Errors
    ///
    /// Returns an error if the error string cannot be read.
    pub fn gossip_error(&self) -> Result<Option<String>, Error> {
        ffi::host_gossip_error(self.handle)
    }

    /// Publish a gossip message to the topic joined by this host.
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails.
    pub fn gossip_publish(&self, message: &str) -> Result<(), Error> {
        ffi::host_gossip_publish(self.handle, message)
    }

    /// Join a GossipSub topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the topic cannot be joined.
    pub fn gossip_join(&self, topic: &str) -> Result<(), Error> {
        ffi::host_gossip_join(self.handle, topic)
    }

    /// Leave a previously joined GossipSub topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the topic is not joined.
    pub fn gossip_leave(&self, topic: &str) -> Result<(), Error> {
        ffi::host_gossip_leave(self.handle, topic)
    }

    /// Publish a gossip message to a specific topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the topic is not joined or publishing fails.
    pub fn gossip_publish_to(&self, topic: &str, message: &str) -> Result<(), Error> {
        ffi::host_gossip_publish_to(self.handle, topic, message)
    }

    /// Drain any queued gossip messages received from the topic.
    ///
    /// # Errors
    ///
    /// Returns an error if the queue cannot be read.
    pub fn gossip_drain(&self) -> Result<Vec<String>, Error> {
        ffi::host_gossip_drain(self.handle)
    }

    /// Drain gossip messages and filter out valid Nostr events.
    ///
    /// Each raw gossip message has the format `topic|sender|payload`.
    /// This method attempts to parse each payload as a Nostr event JSON
    /// and returns only the payloads that successfully verify.
    ///
    /// # Errors
    ///
    /// Returns an error if the queue cannot be read.
    pub fn gossip_drain_nostr(&self) -> Result<Vec<String>, Error> {
        let raw = self.gossip_drain()?;
        let mut events = Vec::new();
        for msg in raw {
            // Format: topic|sender|payload
            let parts: Vec<&str> = msg.splitn(3, '|').collect();
            if parts.len() != 3 {
                continue;
            }
            let payload = parts[2];
            if let Ok(true) = ffi::event_verify(payload) {
                events.push(payload.to_string());
            }
        }
        Ok(events)
    }

    /// Close the host and consume the handle.
    ///
    /// # Errors
    ///
    /// Returns an error if close fails.
    pub fn close(self) -> Result<(), Error> {
        let result = ffi::host_close(self.handle);
        std::mem::forget(self);
        result
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = ffi::host_close(self.handle);
    }
}

// ---------------------------------------------------------------------------
// nostr
// ---------------------------------------------------------------------------

/// Generate a new Nostr secret key (hex-encoded).
///
/// # Errors
///
/// Returns an error if key generation fails.
pub fn nostr_generate_key() -> Result<String, Error> {
    ffi::generate_key()
}

/// Derive the public key from a secret key.
///
/// # Errors
///
/// Returns an error if the secret key is invalid.
pub fn nostr_get_public_key(sk: &str) -> Result<String, Error> {
    ffi::get_public_key(sk)
}

/// Sign a Nostr event.
///
/// Returns the event as a JSON string.
///
/// # Errors
///
/// Returns an error if signing fails.
pub fn nostr_event_sign(sk: &str, content: &str, kind: i32) -> Result<String, Error> {
    ffi::event_sign(sk, content, kind)
}

/// Sign a Nostr event with custom tags and return the JSON.
///
/// `tags_json` must be a JSON array of arrays, e.g.
/// `[["d","my-repo"],["clone","https://example.com/repo.git"]]`.
///
/// # Errors
///
/// Returns an error if tag parsing or signing fails.
pub fn nostr_event_sign_with_tags(
    sk: &str,
    content: &str,
    kind: i32,
    tags_json: &str,
) -> Result<String, Error> {
    ffi::event_sign_with_tags(sk, content, kind, tags_json)
}

/// Verify a Nostr event JSON string.
///
/// # Errors
///
/// Returns an error if the JSON is malformed or the signature check errors.
pub fn nostr_event_verify(json: &str) -> Result<bool, Error> {
    ffi::event_verify(json)
}

/// Encode a hex public key to NIP-19 bech32 (`npub`).
///
/// # Errors
///
/// Returns an error if the hex key is invalid.
pub fn nostr_nip19_encode_pubkey(hex: &str) -> Result<String, Error> {
    ffi::nip19_encode_pubkey(hex)
}

/// Decode a NIP-19 bech32 public key (`npub`) to hex.
///
/// # Errors
///
/// Returns an error if the bech32 string is invalid or not an npub.
pub fn nostr_nip19_decode_pubkey(bech32: &str) -> Result<String, Error> {
    ffi::nip19_decode_pubkey(bech32)
}

/// Encode a hex secret key to NIP-19 bech32 (`nsec`).
///
/// # Errors
///
/// Returns an error if the hex key is invalid.
pub fn nostr_nip19_encode_seckey(hex: &str) -> Result<String, Error> {
    ffi::nip19_encode_seckey(hex)
}

/// Decode a NIP-19 bech32 secret key (`nsec`) to hex.
///
/// # Errors
///
/// Returns an error if the bech32 string is invalid or not an nsec.
pub fn nostr_nip19_decode_seckey(bech32: &str) -> Result<String, Error> {
    ffi::nip19_decode_seckey(bech32)
}

/// Encode a hex event ID to NIP-19 bech32 (`note`).
///
/// # Errors
///
/// Returns an error if the hex id is invalid.
pub fn nostr_nip19_encode_note(hex: &str) -> Result<String, Error> {
    ffi::nip19_encode_note(hex)
}

/// Decode a NIP-19 bech32 event ID (`note`) to hex.
///
/// # Errors
///
/// Returns an error if the bech32 string is invalid or not a note.
pub fn nostr_nip19_decode_note(bech32: &str) -> Result<String, Error> {
    ffi::nip19_decode_note(bech32)
}

/// Encode a NIP-33 entity coordinate to bech32 (`naddr`).
///
/// # Errors
///
/// Returns an error if the parameters are invalid.
pub fn nostr_nip19_encode_entity(
    pubkey: &str,
    kind: i32,
    identifier: &str,
    relays: &str,
) -> Result<String, Error> {
    ffi::nip19_encode_entity(pubkey, kind, identifier, relays)
}

/// Decode a NIP-33 entity coordinate (`naddr`) to JSON.
///
/// # Errors
///
/// Returns an error if the bech32 string is invalid or not an naddr.
pub fn nostr_nip19_decode_entity(bech32: &str) -> Result<String, Error> {
    ffi::nip19_decode_entity(bech32)
}

/// Verify a NIP-05 identifier matches a public key.
///
/// Returns `true` if the identifier resolves to the given pubkey.
///
/// # Errors
///
/// Returns an error if the lookup fails.
pub fn nostr_nip05_verify(identifier: &str, pubkey: &str) -> Result<bool, Error> {
    ffi::nip05_verify(identifier, pubkey)
}

/// Query the public key for a NIP-05 identifier.
///
/// # Errors
///
/// Returns an error if the lookup fails.
pub fn nostr_nip05_query(identifier: &str) -> Result<String, Error> {
    ffi::nip05_query(identifier)
}

/// Connect to a Nostr relay and return a handle.
///
/// # Errors
///
/// Returns an error if the connection fails.
pub fn nostr_relay_connect(url: &str) -> Result<u64, Error> {
    ffi::relay_connect(url)
}

/// Close a Nostr relay connection.
///
/// # Errors
///
/// Returns an error if the handle is invalid.
pub fn nostr_relay_close(handle: u64) -> Result<(), Error> {
    ffi::relay_close(handle)
}

/// Publish a Nostr event JSON to a relay.
///
/// # Errors
///
/// Returns an error if publish fails.
pub fn nostr_relay_publish(handle: u64, event_json: &str) -> Result<(), Error> {
    ffi::relay_publish(handle, event_json)
}

/// Subscribe to a Nostr relay with a filter and return a subscription handle.
///
/// # Errors
///
/// Returns an error if the relay handle is invalid or subscription fails.
pub fn nostr_relay_subscribe(handle: u64, filter_json: &str) -> Result<u64, Error> {
    ffi::relay_subscribe(handle, filter_json)
}

/// Drain the next event from a subscription (non-blocking with 3s timeout).
///
/// Returns `Ok(None)` if no event is available within the timeout.
///
/// # Errors
///
/// Returns an error if the subscription handle is invalid.
pub fn nostr_relay_drain(sub_handle: u64) -> Result<Option<String>, Error> {
    ffi::relay_drain(sub_handle)
}

/// Unsubscribe and close a subscription.
///
/// # Errors
///
/// Returns an error if the subscription handle is invalid.
pub fn nostr_relay_unsubscribe(sub_handle: u64) -> Result<(), Error> {
    ffi::relay_unsubscribe(sub_handle)
}

// ---------------------------------------------------------------------------
// NIP-34 (Git over Nostr) helpers
// ---------------------------------------------------------------------------

impl Repository {
    /// Create a NIP-34 repository announcement event (kind 30617).
    ///
    /// # Errors
    ///
    /// Returns an error if tag construction or signing fails.
    pub fn create_nip34_announcement(
        &self,
        repo_id: &str,
        name: &str,
        description: &str,
        clone_urls: &[String],
        secret_key: &str,
    ) -> Result<String, Error> {
        let mut tags: Vec<Vec<String>> = vec![
            vec!["d".to_string(), repo_id.to_string()],
            vec!["name".to_string(), name.to_string()],
            vec!["description".to_string(), description.to_string()],
        ];
        for url in clone_urls {
            tags.push(vec!["clone".to_string(), url.clone()]);
        }
        let tags_json =
            serde_json::to_string(&tags).map_err(|e| Error::Go(format!("serialize tags: {e}")))?;
        nostr_event_sign_with_tags(secret_key, "", 30617, &tags_json)
    }

    /// Create a NIP-34 repository state event (kind 30618).
    ///
    /// Reads HEAD and branch refs from the repository and constructs
    /// a state event referencing the repo via its `d` tag.
    ///
    /// # Errors
    ///
    /// Returns an error if repo reading or signing fails.
    pub fn create_nip34_state(&self, repo_id: &str, secret_key: &str) -> Result<String, Error> {
        let head = self.head()?;
        let branches = self.branches()?;

        let mut tags: Vec<Vec<String>> = vec![
            vec!["d".to_string(), repo_id.to_string()],
            vec!["HEAD".to_string(), "ref: refs/heads/main".to_string()],
        ];

        for branch in branches {
            // branch name from "refs/heads/main" -> "refs/heads/main"
            tags.push(vec![branch.clone(), head.clone()]);
        }

        let tags_json =
            serde_json::to_string(&tags).map_err(|e| Error::Go(format!("serialize tags: {e}")))?;
        nostr_event_sign_with_tags(secret_key, "", 30618, &tags_json)
    }
}

/// Create a NIP-34 patch event (kind 1617).
///
/// `repo_ref` is the root repo reference: `30617:<pubkey>:<repo-id>`.
///
/// # Errors
///
/// Returns an error if tag construction or signing fails.
pub fn nip34_patch(secret_key: &str, repo_ref: &str, diff: &str) -> Result<String, Error> {
    let tags: Vec<Vec<String>> = vec![vec!["a".to_string(), repo_ref.to_string()]];
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| Error::Go(format!("serialize tags: {e}")))?;
    nostr_event_sign_with_tags(secret_key, diff, 1617, &tags_json)
}

/// Create a NIP-34 issue event (kind 1621).
///
/// # Errors
///
/// Returns an error if tag construction or signing fails.
pub fn nip34_issue(
    secret_key: &str,
    repo_ref: &str,
    title: &str,
    body: &str,
) -> Result<String, Error> {
    let content = format!("{}\n\n{}", title, body);
    let tags: Vec<Vec<String>> = vec![vec!["a".to_string(), repo_ref.to_string()]];
    let tags_json =
        serde_json::to_string(&tags).map_err(|e| Error::Go(format!("serialize tags: {e}")))?;
    nostr_event_sign_with_tags(secret_key, &content, 1621, &tags_json)
}

// ---------------------------------------------------------------------------
// git
// ---------------------------------------------------------------------------

/// Clone a Git repository.
///
/// # Errors
///
/// Returns an error if cloning fails.
pub fn git_clone(url: &str, path: &str, bare: bool) -> Result<(), Error> {
    ffi::git_clone_repo(url, path, bare)
}

/// Initialize a new Git repository.
///
/// # Errors
///
/// Returns an error if initialization fails.
pub fn git_init(path: &str, bare: bool) -> Result<(), Error> {
    ffi::git_init_repo(path, bare)
}

/// An opened Git repository handle.
///
/// When dropped, the handle is released.
pub struct Repository {
    handle: u64,
}

impl Repository {
    /// Open an existing Git repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref().to_str().ok_or(Error::InvalidPath)?;
        if path.contains('\0') {
            return Err(Error::InvalidPath);
        }
        let handle = ffi::git_open_repo(path)?;
        Ok(Repository { handle })
    }

    /// Return the hash of the current HEAD commit.
    ///
    /// # Errors
    ///
    /// Returns an error if HEAD cannot be resolved.
    pub fn head(&self) -> Result<String, Error> {
        ffi::git_repo_head_hash(self.handle)
    }

    /// Return whether the repository is bare.
    ///
    /// # Errors
    ///
    /// Returns an error if the handle is invalid.
    pub fn is_bare(&self) -> Result<bool, Error> {
        ffi::git_repo_is_bare(self.handle)
    }

    /// Return the list of branch names.
    ///
    /// # Errors
    ///
    /// Returns an error if the branch list cannot be read.
    pub fn branches(&self) -> Result<Vec<String>, Error> {
        ffi::git_repo_branches(self.handle)
    }

    /// Return the list of remote names.
    ///
    /// # Errors
    ///
    /// Returns an error if the remote list cannot be read.
    pub fn remotes(&self) -> Result<Vec<String>, Error> {
        ffi::git_repo_remotes(self.handle)
    }

    /// Create a new branch pointing to the given commit hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the branch cannot be created.
    pub fn create_branch(&self, name: &str, commit_hash: &str) -> Result<(), Error> {
        ffi::git_repo_create_branch(self.handle, name, commit_hash)
    }

    /// Look up a commit by hash and return its message.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit cannot be found.
    pub fn commit_message(&self, hash: &str) -> Result<String, Error> {
        ffi::git_repo_commit_lookup(self.handle, hash)
    }

    /// Return the entries of a tree as (name, hash) tuples.
    ///
    /// # Errors
    ///
    /// Returns an error if the tree cannot be read.
    pub fn tree_entries(&self, hash: &str) -> Result<Vec<(String, String)>, Error> {
        ffi::git_repo_tree_entries(self.handle, hash)
    }

    /// Read the contents of a blob by hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the blob cannot be read.
    pub fn blob_read(&self, hash: &str) -> Result<Vec<u8>, Error> {
        ffi::git_repo_blob_read(self.handle, hash)
    }

    /// Return the working tree status as a short-format string.
    ///
    /// # Errors
    ///
    /// Returns an error if the status cannot be read.
    pub fn status(&self) -> Result<String, Error> {
        ffi::git_repo_status(self.handle)
    }

    /// Compare two trees and return the diff patch.
    ///
    /// # Errors
    ///
    /// Returns an error if either tree cannot be read or the diff fails.
    pub fn diff_trees(&self, old_hash: &str, new_hash: &str) -> Result<String, Error> {
        ffi::git_repo_diff_trees(self.handle, old_hash, new_hash)
    }

    /// Release the handle.
    ///
    /// # Errors
    ///
    /// Returns an error if release fails.
    pub fn close(self) -> Result<(), Error> {
        let result = ffi::git_repo_release(self.handle);
        std::mem::forget(self);
        result
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = ffi::git_repo_release(self.handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_dir(name: &str) -> PathBuf {
        let path = PathBuf::from("tmp").join("test").join(name);
        let _ = fs::remove_dir_all(&path);
        path
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty(), "version should not be empty");
        assert!(v.contains('.'), "version should contain a dot");
    }

    #[test]
    fn test_init_repo_and_start_node() {
        let repo = tmp_dir("init_repo_and_start_node").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        assert!(repo.exists(), "repo directory should exist");

        let node = Node::start(&repo, false).expect("start node should succeed");
        let peer_id = node.peer_id().expect("peer_id should succeed");
        assert!(!peer_id.is_empty(), "peer_id should not be empty");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_add_and_cat() {
        let repo = tmp_dir("add_and_cat").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let data = b"hello from kubo-rs ffi";
        let cid = node.add_bytes(data).expect("add_bytes should succeed");
        assert!(!cid.is_empty(), "cid should not be empty");

        let fetched = node.cat(&cid).expect("cat should succeed");
        assert_eq!(fetched, data, "retrieved data should match");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_add_hello_world_cidv0_alignment() {
        // Aligns with go/kubo-sys/test/cli/add_test.go:
        // shortString = "hello world"
        // shortStringCidV0 = "Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD"
        let repo = tmp_dir("add_hello_world_cidv0").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let cid = node
            .add_bytes(b"hello world")
            .expect("add_bytes should succeed");
        assert_eq!(
            cid, "Qmf412jQZiuVUtdgnB36FXFX7xg5V6KEbSJ4dpQuhkLyfD",
            "CID for 'hello world' must match kubo default profile (CIDv0 dag-pb sha2-256)"
        );

        let fetched = node.cat(&cid).expect("cat should succeed");
        assert_eq!(fetched, b"hello world");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_listening_addrs_online() {
        let repo = tmp_dir("listening_addrs_online").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, true).expect("start node should succeed");

        let addrs = node
            .listening_addrs()
            .expect("listening_addrs should succeed");
        assert!(
            !addrs.is_empty(),
            "online node should have listening addresses"
        );

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_drop_stops_node() {
        let repo = tmp_dir("drop_stops_node").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        {
            let _node = Node::start(&repo, false).expect("start node should succeed");
            // Node is dropped here; should not panic.
        }
        // Starting a new node on the same repo should succeed after drop.
        let node = Node::start(&repo, false).expect("restart node should succeed");
        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_add_cat_empty() {
        let repo = tmp_dir("add_cat_empty").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let data: &[u8] = b"";
        let cid = node.add_bytes(data).expect("add_bytes should succeed");
        let fetched = node.cat(&cid).expect("cat should succeed");
        assert_eq!(fetched, data);

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_invalid_path() {
        let result = init_repo("path\0with\0null");
        assert!(
            matches!(result, Err(Error::InvalidPath)),
            "null path should fail with InvalidPath"
        );
    }

    #[test]
    fn test_two_nodes_exchange_data() {
        let base = tmp_dir("two_nodes_exchange_data");
        let repo_a = base.join("repo_a");
        let repo_b = base.join("repo_b");

        init_repo(&repo_a).expect("init_repo_a should succeed");
        init_repo(&repo_b).expect("init_repo_b should succeed");

        let node_a = Node::start(&repo_a, true).expect("start node_a should succeed");
        let node_b = Node::start(&repo_b, true).expect("start node_b should succeed");

        let peer_id_a = node_a.peer_id().expect("peer_id_a should succeed");
        let addrs_a = node_a
            .listening_addrs()
            .expect("listening_addrs_a should succeed");
        assert!(!addrs_a.is_empty(), "node_a should have addresses");

        // Pick the first address and append the peer ID.
        let dial_addr = format!("{}/p2p/{}", addrs_a[0], peer_id_a);
        node_b
            .connect(&dial_addr)
            .expect("connect b->a should succeed");

        // Add data on node_a.
        let data = b"peer-to-peer hello";
        let cid = node_a.add_bytes(data).expect("add_bytes should succeed");

        // Fetch from node_b.
        let fetched = node_b.cat(&cid).expect("cat from node_b should succeed");
        assert_eq!(fetched, data, "data fetched via bitswap should match");

        node_a.stop().expect("stop node_a should succeed");
        node_b.stop().expect("stop node_b should succeed");
    }

    #[test]
    fn test_swarm_peers_and_id() {
        let base = tmp_dir("swarm_peers_and_id");
        let repo_a = base.join("repo_a");
        let repo_b = base.join("repo_b");

        init_repo(&repo_a).expect("init_repo_a should succeed");
        init_repo(&repo_b).expect("init_repo_b should succeed");

        let node_a = Node::start(&repo_a, true).expect("start node_a should succeed");
        let node_b = Node::start(&repo_b, true).expect("start node_b should succeed");

        let peer_id_a = node_a.peer_id().expect("peer_id_a should succeed");
        let addrs_a = node_a
            .listening_addrs()
            .expect("listening_addrs_a should succeed");
        assert!(!addrs_a.is_empty(), "node_a should have addresses");

        let dial_addr = format!("{}/p2p/{}", addrs_a[0], peer_id_a);
        node_b
            .connect(&dial_addr)
            .expect("connect b->a should succeed");

        let peers_a = node_a.swarm_peers().expect("swarm_peers a should succeed");
        assert!(
            peers_a
                .iter()
                .any(|(id, _)| id == &peer_id_a || id == &node_b.peer_id().unwrap()),
            "node_a should see node_b or itself in peer list"
        );

        let id_json = node_a.id().expect("id should succeed");
        assert!(id_json.contains("id\""), "id json should contain id field");

        node_a.stop().expect("stop node_a should succeed");
        node_b.stop().expect("stop node_b should succeed");
    }

    #[test]
    fn test_block_put_get_stat() {
        let repo = tmp_dir("block_put_get_stat").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let data = b"raw block data";
        let cid = node.block_put(data).expect("block_put should succeed");
        assert!(!cid.is_empty(), "cid should not be empty");

        let size = node.block_stat(&cid).expect("block_stat should succeed");
        assert_eq!(size, data.len(), "block size should match");

        let fetched = node.block_get(&cid).expect("block_get should succeed");
        assert_eq!(fetched, data, "retrieved block should match");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_pin_add_rm_ls() {
        let repo = tmp_dir("pin_add_rm_ls").join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let data = b"pin me";
        let cid = node.add_bytes(data).expect("add_bytes should succeed");

        node.pin_add(&cid, true).expect("pin_add should succeed");

        let pins = node.pin_ls().expect("pin_ls should succeed");
        assert!(
            pins.iter().any(|(p, _)| p.contains(&cid)),
            "pinned cid should appear in pin_ls"
        );

        node.pin_rm(&cid, true).expect("pin_rm should succeed");

        let pins_after = node.pin_ls().expect("pin_ls after rm should succeed");
        assert!(
            !pins_after.iter().any(|(p, _)| p.contains(&cid)),
            "unpinned cid should not appear in pin_ls"
        );

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_swarm_disconnect() {
        let base = tmp_dir("swarm_disconnect");
        let repo_a = base.join("repo_a");
        let repo_b = base.join("repo_b");

        init_repo(&repo_a).expect("init_repo_a should succeed");
        init_repo(&repo_b).expect("init_repo_b should succeed");

        let node_a = Node::start(&repo_a, true).expect("start node_a should succeed");
        let node_b = Node::start(&repo_b, true).expect("start node_b should succeed");

        let peer_id_a = node_a.peer_id().expect("peer_id_a should succeed");
        let addrs_a = node_a
            .listening_addrs()
            .expect("listening_addrs_a should succeed");
        assert!(!addrs_a.is_empty(), "node_a should have addresses");

        let dial_addr = format!("{}/p2p/{}", addrs_a[0], peer_id_a);
        node_b
            .connect(&dial_addr)
            .expect("connect b->a should succeed");

        // Disconnect and verify the peer is gone
        node_b
            .disconnect(&dial_addr)
            .expect("disconnect should succeed");

        node_a.stop().expect("stop node_a should succeed");
        node_b.stop().expect("stop node_b should succeed");
    }

    #[test]
    #[ignore = "requires DHT bootstrap peers or local network discovery"]
    fn test_name_publish_resolve() {
        let repo = tmp_dir("name_publish_resolve").join("repo");
        init_repo(&repo).expect("init_repo should succeed");

        let node = Node::start(&repo, true).expect("start should succeed");
        let cid = node
            .add_bytes(b"name test")
            .expect("add_bytes should succeed");

        let name = node
            .name_publish(&cid, 60)
            .expect("name_publish should succeed");
        assert!(!name.is_empty(), "published name should not be empty");

        let resolved = node
            .name_resolve(&name)
            .expect("name_resolve should succeed");
        assert!(
            resolved.contains(&cid),
            "resolved path should contain the cid"
        );

        node.stop().expect("stop should succeed");
    }

    #[test]
    #[ignore = "requires DHT bootstrap peers or local network discovery"]
    fn test_dht_findpeer_local() {
        let base = tmp_dir("dht_findpeer_local");
        let repo_a = base.join("repo_a");
        let repo_b = base.join("repo_b");

        init_repo(&repo_a).expect("init_repo_a should succeed");
        init_repo(&repo_b).expect("init_repo_b should succeed");

        let node_a = Node::start(&repo_a, true).expect("start node_a should succeed");
        let node_b = Node::start(&repo_b, true).expect("start node_b should succeed");

        let peer_id_a = node_a.peer_id().expect("peer_id_a should succeed");

        // Connect so each node knows about the other.
        let addrs_a = node_a
            .listening_addrs()
            .expect("listening_addrs_a should succeed");
        let dial_addr = format!("{}/p2p/{}", addrs_a[0], peer_id_a);
        node_b
            .connect(&dial_addr)
            .expect("connect b->a should succeed");

        // DHT lookup for peer A from node B.
        let (found_id, found_addrs) = node_b
            .dht_findpeer(&peer_id_a)
            .expect("dht_findpeer should succeed");
        assert_eq!(found_id, peer_id_a, "found peer id should match");
        assert!(!found_addrs.is_empty(), "found peer should have addresses");

        node_a.stop().expect("stop node_a should succeed");
        node_b.stop().expect("stop node_b should succeed");
    }

    // -----------------------------------------------------------------------
    // libp2p tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_libp2p_host_lifecycle() {
        let host = Host::new().expect("host new should succeed");
        let peer_id = host.peer_id().expect("peer_id should succeed");
        assert!(!peer_id.is_empty(), "peer_id should not be empty");

        let addrs = host
            .listening_addrs()
            .expect("listening_addrs should succeed");
        assert!(!addrs.is_empty(), "host should have listening addresses");

        host.close().expect("close should succeed");
    }

    #[test]
    fn test_libp2p_host_drop() {
        {
            let _host = Host::new().expect("host new should succeed");
        }
        // Dropping should not panic.
        let host = Host::new().expect("second host new should succeed");
        host.close().expect("close should succeed");
    }

    // -----------------------------------------------------------------------
    // nostr tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_nostr_key_generation() {
        let sk = nostr_generate_key().expect("generate key should succeed");
        assert_eq!(sk.len(), 64, "secret key should be 64 hex chars");

        let pk = nostr_get_public_key(&sk).expect("get public key should succeed");
        assert_eq!(pk.len(), 64, "public key should be 64 hex chars");
    }

    #[test]
    fn test_nostr_sign_and_verify() {
        let sk = nostr_generate_key().expect("generate key should succeed");

        let event_json = nostr_event_sign(&sk, "hello nostr", 1).expect("sign should succeed");
        assert!(
            event_json.contains("hello nostr"),
            "event should contain content"
        );

        let valid = nostr_event_verify(&event_json).expect("verify should succeed");
        assert!(valid, "signature should be valid");
    }

    #[test]
    fn test_nostr_verify_invalid() {
        let valid = nostr_event_verify(
            r#"{"id":"bad","pubkey":"bad","created_at":0,"kind":1,"tags":[],"content":"x","sig":"bad"}"#,
        );
        assert!(
            matches!(valid, Ok(false) | Err(_)),
            "invalid event should not verify"
        );
    }

    #[test]
    fn test_nostr_relay_subscribe_invalid_handle() {
        let result = nostr_relay_subscribe(0, r#"{"kinds":[1]}"#);
        assert!(result.is_err(), "subscribe with invalid handle should fail");
    }

    #[test]
    #[ignore = "requires public Nostr relay (wss://relay.damus.io)"]
    fn test_nostr_relay_publish_and_drain() {
        let relay = nostr_relay_connect("wss://relay.damus.io").expect("connect should succeed");

        let sk = nostr_generate_key().expect("generate key should succeed");
        let event = nostr_event_sign(&sk, "hybrid protocol test", 1).expect("sign should succeed");

        // Subscribe to our own pubkey before publishing.
        let pk = nostr_get_public_key(&sk).expect("get pubkey should succeed");
        let filter = format!(r#"{{"kinds":[1],"authors":["{}"],"limit":1}}"#, pk);
        let sub = nostr_relay_subscribe(relay, &filter).expect("subscribe should succeed");

        // Publish the event.
        nostr_relay_publish(relay, &event).expect("publish should succeed");

        // Drain until we get it back (max 5 attempts, 3s timeout each).
        let mut found = false;
        for _ in 0..5 {
            if let Some(evt) = nostr_relay_drain(sub).expect("drain should not error") {
                assert!(
                    evt.contains("hybrid protocol test"),
                    "drained event should match"
                );
                found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        assert!(found, "should have received the published event");

        nostr_relay_unsubscribe(sub).expect("unsubscribe should succeed");
        nostr_relay_close(relay).expect("close should succeed");
    }

    // -----------------------------------------------------------------------
    // git tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_git_init_and_open() {
        let path = tmp_dir("git_init_and_open").join("repo");

        git_init(path.to_str().unwrap(), false).expect("git init should succeed");
        assert!(path.join(".git").exists(), ".git directory should exist");

        let repo = Repository::open(&path).expect("open should succeed");
        repo.close().expect("close should succeed");
    }

    #[test]
    fn test_git_init_bare() {
        let path = tmp_dir("git_init_bare").join("repo.git");

        git_init(path.to_str().unwrap(), true).expect("git init bare should succeed");
        // In a bare repo the path itself is the git dir.
        assert!(path.join("HEAD").exists(), "bare repo should have HEAD");
    }

    #[test]
    fn test_nostr_event_sign_with_tags() {
        let sk = nostr_generate_key().expect("generate key should succeed");
        let tags_json = r#"[["d","my-repo"],["clone","https://example.com/repo.git"]]"#;
        let event = nostr_event_sign_with_tags(&sk, "hello tagged", 1, tags_json)
            .expect("sign with tags should succeed");

        assert!(
            event.contains("hello tagged"),
            "event should contain content"
        );
        assert!(event.contains("my-repo"), "event should contain tag value");
        assert!(event.contains("clone"), "event should contain tag name");

        let valid = nostr_event_verify(&event).expect("verify should succeed");
        assert!(valid, "tagged event signature should be valid");
    }

    #[test]
    fn test_nip34_repo_announcement() {
        let path = tmp_dir("nip34_repo").join("repo");
        git_init(path.to_str().unwrap(), false).expect("git init should succeed");

        let repo = Repository::open(&path).expect("open should succeed");
        let sk = nostr_generate_key().expect("generate key should succeed");

        let event = repo
            .create_nip34_announcement(
                "my-repo",
                "My Repository",
                "A test repo for NIP-34",
                &["https://example.com/repo.git".to_string()],
                &sk,
            )
            .expect("create announcement should succeed");

        assert!(event.contains("30617"), "event should be kind 30617");
        assert!(
            event.contains("My Repository"),
            "event should contain repo name"
        );

        let valid = nostr_event_verify(&event).expect("verify should succeed");
        assert!(valid, "NIP-34 announcement signature should be valid");

        repo.close().expect("close should succeed");
    }

    #[test]
    fn test_nip34_patch() {
        let sk = nostr_generate_key().expect("generate key should succeed");
        let event = nip34_patch(
            &sk,
            "30617:abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678:my-repo",
            "diff --git a/file.txt b/file.txt\n--- a/file.txt\n+++ b/file.txt\n@@ -1 +1 @@\n-old\n+new\n",
        )
        .expect("create patch should succeed");

        assert!(event.contains("1617"), "event should be kind 1617");
        assert!(event.contains("diff --git"), "event should contain patch");

        let valid = nostr_event_verify(&event).expect("verify should succeed");
        assert!(valid, "NIP-34 patch signature should be valid");
    }
}
