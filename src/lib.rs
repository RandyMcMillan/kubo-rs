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
pub mod nostr_url;

pub use error::Error;
pub use ffi::version;
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
}
