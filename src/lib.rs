mod error;
mod ffi;

pub use error::Error;
pub use ffi::version;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty(), "version should not be empty");
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

    #[test]
    fn test_listening_addrs_online() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");

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
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");

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
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path().join("repo");

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
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo_a = tmp.path().join("repo_a");
        let repo_b = tmp.path().join("repo_b");

        init_repo(&repo_a).expect("init repo_a should succeed");
        init_repo(&repo_b).expect("init repo_b should succeed");

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
}
