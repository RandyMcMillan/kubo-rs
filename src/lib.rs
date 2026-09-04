mod ffi;

use std::path::Path;

pub use ffi::version;

/// Initialize a new IPFS repo at the given path.
///
/// # Errors
///
/// Returns an error string if the repo cannot be initialized.
pub fn init_repo<P: AsRef<Path>>(path: P) -> Result<(), String> {
    ffi::init_repo(path.as_ref().to_str().ok_or("invalid utf-8 path")?)
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
    /// Returns an error string if the node cannot be started.
    pub fn start<P: AsRef<Path>>(path: P, online: bool) -> Result<Self, String> {
        let path = path.as_ref().to_str().ok_or("invalid utf-8 path")?;
        let handle = ffi::node_start(path, online)?;
        Ok(Node { handle })
    }

    /// Return the node's peer ID.
    ///
    /// # Errors
    ///
    /// Returns an error string if the peer ID cannot be read.
    pub fn peer_id(&self) -> Result<String, String> {
        ffi::node_peer_id(self.handle)
    }

    /// Add a byte slice to IPFS and return the resulting CID.
    ///
    /// # Errors
    ///
    /// Returns an error string if the add operation fails.
    pub fn add_bytes(&self, data: &[u8]) -> Result<String, String> {
        ffi::unixfs_add_bytes(self.handle, data)
    }

    /// Retrieve the contents of a UnixFS file by CID.
    ///
    /// # Errors
    ///
    /// Returns an error string if the content cannot be retrieved.
    pub fn cat(&self, cid: &str) -> Result<Vec<u8>, String> {
        ffi::unixfs_cat(self.handle, cid)
    }

    /// Shut the node down and consume the handle.
    ///
    /// # Errors
    ///
    /// Returns an error string if shutdown fails.
    pub fn stop(self) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty(), "version should not be empty");
        // Kubo version strings typically look like "0.44.0-dev" or similar.
        assert!(v.contains('.'), "version should contain a dot");
    }

    #[test]
    fn test_init_repo_and_start_node() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");

        init_repo(&repo).expect("init repo should succeed");
        assert!(repo.exists(), "repo directory should exist");

        let node = Node::start(&repo, false).expect("start node should succeed");
        let peer_id = node.peer_id().expect("peer_id should succeed");
        assert!(!peer_id.is_empty(), "peer_id should not be empty");

        node.stop().expect("stop should succeed");
    }

    #[test]
    fn test_add_and_cat() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");

        init_repo(&repo).expect("init repo should succeed");
        let node = Node::start(&repo, false).expect("start node should succeed");

        let data = b"hello from kubo-rs ffi";
        let cid = node.add_bytes(data).expect("add_bytes should succeed");
        assert!(!cid.is_empty(), "cid should not be empty");

        let fetched = node.cat(&cid).expect("cat should succeed");
        assert_eq!(fetched, data, "retrieved data should match");

        node.stop().expect("stop should succeed");
    }
}
