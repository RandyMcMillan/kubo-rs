use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

uniffi::setup_scaffolding!();

/// UniFFI-compatible wrapper for `kubo_rs::p2p_messages::HybridMessage`.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct HybridMessage {
    pub kind: u16,
    pub content: String,
    pub tags: Vec<Vec<String>>,
}

impl From<kubo_rs::p2p_messages::HybridMessage> for HybridMessage {
    fn from(msg: kubo_rs::p2p_messages::HybridMessage) -> Self {
        HybridMessage {
            kind: msg.kind,
            content: msg.content,
            tags: msg.tags,
        }
    }
}

impl From<HybridMessage> for kubo_rs::p2p_messages::HybridMessage {
    fn from(msg: HybridMessage) -> Self {
        kubo_rs::p2p_messages::HybridMessage {
            kind: msg.kind,
            content: msg.content,
            tags: msg.tags,
        }
    }
}

/// UniFFI-compatible wrapper for `kubo_rs::p2p_messages::MessageCategory`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum MessageCategory {
    File,
    Repo,
    Patch,
    Issue,
    Other,
}

impl From<kubo_rs::p2p_messages::MessageCategory> for MessageCategory {
    fn from(cat: kubo_rs::p2p_messages::MessageCategory) -> Self {
        match cat {
            kubo_rs::p2p_messages::MessageCategory::File => MessageCategory::File,
            kubo_rs::p2p_messages::MessageCategory::Repo => MessageCategory::Repo,
            kubo_rs::p2p_messages::MessageCategory::Patch => MessageCategory::Patch,
            kubo_rs::p2p_messages::MessageCategory::Issue => MessageCategory::Issue,
            kubo_rs::p2p_messages::MessageCategory::Other => MessageCategory::Other,
        }
    }
}

/// Error type exposed to Swift via UniFFI `throws`.
#[derive(Debug, uniffi::Error)]
pub enum RustyError {
    Ipfs { msg: String },
    P2p { msg: String },
    Nostr { msg: String },
    Git { msg: String },
    Generic { msg: String },
}

impl std::fmt::Display for RustyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RustyError::Ipfs { msg } => write!(f, "IPFS error: {msg}"),
            RustyError::P2p { msg } => write!(f, "P2P error: {msg}"),
            RustyError::Nostr { msg } => write!(f, "Nostr error: {msg}"),
            RustyError::Git { msg } => write!(f, "Git error: {msg}"),
            RustyError::Generic { msg } => write!(f, "{msg}"),
        }
    }
}

impl From<kubo_rs::Error> for RustyError {
    fn from(err: kubo_rs::Error) -> Self {
        let msg = format!("{err}");
        RustyError::Generic { msg }
    }
}

fn map_ipfs<T>(result: Result<T, kubo_rs::Error>) -> Result<T, RustyError> {
    result.map_err(|e| RustyError::Ipfs { msg: format!("{e}") })
}

fn map_p2p<T>(result: Result<T, kubo_rs::Error>) -> Result<T, RustyError> {
    result.map_err(|e| RustyError::P2p { msg: format!("{e}") })
}

fn map_nostr<T>(result: Result<T, kubo_rs::Error>) -> Result<T, RustyError> {
    result.map_err(|e| RustyError::Nostr { msg: format!("{e}") })
}

fn map_git<T>(result: Result<T, kubo_rs::Error>) -> Result<T, RustyError> {
    result.map_err(|e| RustyError::Git { msg: format!("{e}") })
}

static P2P_HOST: Mutex<Option<kubo_rs::Host>> = Mutex::new(None);
static P2P_LAST_ERROR: Mutex<String> = Mutex::new(String::new());

static HYBRID_NODE: Mutex<Option<kubo_rs::HybridNode>> = Mutex::new(None);
static HYBRID_LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_hybrid_error(msg: &str) {
    if let Ok(mut last) = HYBRID_LAST_ERROR.lock() {
        *last = msg.to_string();
    }
}

fn hybrid_with<R>(f: impl FnOnce(&kubo_rs::HybridNode) -> R) -> Option<R> {
    let guard = HYBRID_NODE.lock().ok()?;
    let node = guard.as_ref()?;
    Some(f(node))
}

fn demo_repo_path() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("kubo-rs-xcode-{}-{}", std::process::id(), stamp))
}

fn kubo_roundtrip_summary(message: &str) -> Result<String, kubo_rs::Error> {
    let repo_path = demo_repo_path();
    let result = (|| {
        kubo_rs::init_repo(&repo_path)?;
        let node = kubo_rs::Node::start(&repo_path, false)?;

        let cid = node.add_bytes(message.as_bytes())?;
        let fetched = node.cat(&cid)?;
        let peer_id = node.peer_id()?;
        node.stop()?;

        let roundtrip = String::from_utf8(fetched).map_err(|err| {
            kubo_rs::Error::Go(format!("round-trip data was not valid UTF-8: {err}"))
        })?;

        Ok(format!(
            "kubo-rs {} | peer {} | cid {} | round-trip {}",
            kubo_rs::version(),
            peer_id,
            cid,
            roundtrip
        ))
    })();

    let _ = fs::remove_dir_all(&repo_path);
    result
}

fn p2p_host<R>(f: impl FnOnce(&kubo_rs::Host) -> R) -> Option<R> {
    let mut guard = P2P_HOST.lock().ok()?;
    if guard.is_none() {
        let host = match kubo_rs::Host::new() {
            Ok(host) => host,
            Err(err) => {
                let msg = format!("{err}");
                eprintln!("p2p host start failed: {msg}");
                if let Ok(mut last) = P2P_LAST_ERROR.lock() {
                    *last = msg;
                }
                return None;
            }
        };
        *guard = Some(host);
    }

    let host = guard.as_ref()?;
    Some(f(host))
}

#[uniffi::export]
pub fn p2p_last_error() -> String {
    P2P_LAST_ERROR.lock().map(|s| s.clone()).unwrap_or_default()
}

#[uniffi::export]
fn rust_hello() -> String {
    match kubo_roundtrip_summary("Hello from Rust!") {
        Ok(summary) => summary,
        Err(err) => format!("Hello from Rust! (kubo-rs demo failed: {err})"),
    }
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

#[uniffi::export]
pub fn p2p_start() -> String {
    p2p_peer_id()
}

#[uniffi::export]
pub fn p2p_peer_id() -> String {
    p2p_host(|host| host.peer_id().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_listening_addrs() -> Vec<String> {
    p2p_host(|host| host.listening_addrs().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_connect(addr: &str) -> bool {
    p2p_host(|host| host.connect(addr).is_ok()).unwrap_or(false)
}

#[uniffi::export]
pub fn p2p_ping(peer_id: &str) -> i64 {
    p2p_host(|host| host.ping(peer_id).ok())
        .flatten()
        .unwrap_or(-1)
}

#[uniffi::export]
pub fn p2p_protocols() -> Vec<String> {
    p2p_host(|host| host.protocols().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_gossip_topic() -> String {
    p2p_host(|host| host.gossip_topic().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_gossip_error() -> String {
    p2p_host(|host| host.gossip_error().ok())
        .flatten()
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_gossip_publish(message: &str) -> bool {
    p2p_host(|host| host.gossip_publish(message).is_ok()).unwrap_or(false)
}

#[uniffi::export]
pub fn p2p_gossip_drain() -> Vec<String> {
    p2p_host(|host| host.gossip_drain().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn p2p_gossip_join(topic: &str) -> bool {
    p2p_host(|host| host.gossip_join(topic).is_ok()).unwrap_or(false)
}

#[uniffi::export]
pub fn p2p_gossip_leave(topic: &str) -> bool {
    p2p_host(|host| host.gossip_leave(topic).is_ok()).unwrap_or(false)
}

#[uniffi::export]
pub fn p2p_gossip_publish_to(topic: &str, message: &str) -> bool {
    p2p_host(|host| host.gossip_publish_to(topic, message).is_ok()).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// HybridNode
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn hybrid_start(online: bool) -> bool {
    let repo_path = demo_repo_path();
    match kubo_rs::HybridNode::start(&repo_path, online) {
        Ok(node) => {
            if let Ok(mut guard) = HYBRID_NODE.lock() {
                *guard = Some(node);
            }
            true
        }
        Err(err) => {
            set_hybrid_error(&format!("{err}"));
            false
        }
    }
}

#[uniffi::export]
pub fn hybrid_stop() -> bool {
    let mut guard = match HYBRID_NODE.lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    if let Some(node) = guard.take() {
        if let Err(err) = node.stop() {
            set_hybrid_error(&format!("{err}"));
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// HybridNode — throwing variants (Phase 14)
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn hybrid_start_try(online: bool) -> Result<(), RustyError> {
    let repo_path = demo_repo_path();
    let node = kubo_rs::HybridNode::start(&repo_path, online)?;
    if let Ok(mut guard) = HYBRID_NODE.lock() {
        *guard = Some(node);
    }
    Ok(())
}

#[uniffi::export]
pub fn hybrid_stop_try() -> Result<(), RustyError> {
    let mut guard = HYBRID_NODE.lock().map_err(|_| RustyError::Generic {
        msg: "mutex poisoned".to_string(),
    })?;
    if let Some(node) = guard.take() {
        node.stop()?;
    }
    Ok(())
}

#[uniffi::export]
pub fn ipfs_add_try(data: Vec<u8>) -> Result<String, RustyError> {
    let cid = hybrid_with(|node| node.ipfs.add_bytes(&data))
        .ok_or_else(|| RustyError::Ipfs {
            msg: "hybrid node not started".to_string(),
        })??;
    Ok(cid)
}

#[uniffi::export]
pub fn ipfs_cat_try(cid: &str) -> Result<Vec<u8>, RustyError> {
    let data = hybrid_with(|node| node.ipfs.cat(cid))
        .ok_or_else(|| RustyError::Ipfs {
            msg: "hybrid node not started".to_string(),
        })??;
    Ok(data)
}

#[uniffi::export]
pub fn p2p_connect_try(addr: &str) -> Result<(), RustyError> {
    p2p_host(|host| host.connect(addr))
        .ok_or_else(|| RustyError::P2p {
            msg: "p2p host not started".to_string(),
        })??;
    Ok(())
}

#[uniffi::export]
pub fn hybrid_publish_file_try(
    file_path: &str,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> Result<String, RustyError> {
    let cid = hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.publish_file(file_path, secret_key, relay_handle, topic)
    })
    .ok_or_else(|| RustyError::Generic {
        msg: "hybrid node not started".to_string(),
    })??;
    Ok(cid)
}

#[uniffi::export]
pub fn hybrid_ipfs_peer_id() -> String {
    hybrid_with(|node| node.ipfs.peer_id().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_p2p_peer_id() -> String {
    hybrid_with(|node| node.p2p.peer_id().ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_broadcast_event(
    event_json: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> bool {
    hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.broadcast_event(event_json, relay_handle, topic)
            .is_ok()
    })
    .unwrap_or(false)
}

#[uniffi::export]
pub fn hybrid_drain_events(relay_sub_handle: Option<u64>) -> Vec<String> {
    hybrid_with(|node| node.drain_events(relay_sub_handle).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_drain_by_kind(relay_sub_handle: Option<u64>, kind: u16) -> Vec<String> {
    hybrid_with(|node| node.drain_by_kind(relay_sub_handle, kind).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_publish_file(
    file_path: &str,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> String {
    hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.publish_file(file_path, secret_key, relay_handle, topic)
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_resolve_nip94(event_json: &str) -> String {
    hybrid_with(|node| {
        node.resolve_nip94(event_json)
            .map(|(cid, content)| format!("{}", String::from_utf8_lossy(&content)))
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_publish_repo(
    repo_path: &str,
    repo_id: &str,
    name: &str,
    description: &str,
    clone_urls: Vec<String>,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> String {
    hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.publish_repo(
            repo_path,
            repo_id,
            name,
            description,
            &clone_urls,
            secret_key,
            relay_handle,
            topic,
        )
        .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_publish_patch(
    repo_path: &str,
    repo_ref: &str,
    old_hash: &str,
    new_hash: &str,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> String {
    hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.publish_patch(
            repo_path,
            repo_ref,
            old_hash,
            new_hash,
            secret_key,
            relay_handle,
            topic,
        )
        .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn hybrid_publish_issue(
    repo_ref: &str,
    title: &str,
    body: &str,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> String {
    hybrid_with(|node| {
        let topic = gossip_topic.as_deref();
        node.publish_issue(repo_ref, title, body, secret_key, relay_handle, topic)
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// IPFS Node API (via HybridNode singleton)
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn ipfs_add(data: Vec<u8>) -> String {
    hybrid_with(|node| node.ipfs.add_bytes(&data).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_cat(cid: &str) -> Vec<u8> {
    hybrid_with(|node| node.ipfs.cat(cid).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_pin_add(cid: &str, recursive: bool) -> bool {
    hybrid_with(|node| node.ipfs.pin_add(cid, recursive).is_ok())
        .unwrap_or(false)
}

#[uniffi::export]
pub fn ipfs_pin_rm(cid: &str, recursive: bool) -> bool {
    hybrid_with(|node| node.ipfs.pin_rm(cid, recursive).is_ok())
        .unwrap_or(false)
}

#[uniffi::export]
pub fn ipfs_pin_ls() -> Vec<String> {
    hybrid_with(|node| {
        node.ipfs
            .pin_ls()
            .map(|pins| pins.into_iter().map(|(path, _typ)| path).collect())
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_block_put(data: Vec<u8>) -> String {
    hybrid_with(|node| node.ipfs.block_put(&data).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_block_get(cid: &str) -> Vec<u8> {
    hybrid_with(|node| node.ipfs.block_get(cid).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_block_stat(cid: &str) -> u64 {
    hybrid_with(|node| node.ipfs.block_stat(cid).ok())
        .flatten()
        .map(|s| s as u64)
        .unwrap_or(0)
}

#[uniffi::export]
pub fn ipfs_dht_findpeer(peer_id: &str) -> Vec<String> {
    hybrid_with(|node| {
        node.ipfs
            .dht_findpeer(peer_id)
            .map(|(_id, addrs)| addrs)
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_dht_findprovs(cid: &str) -> Vec<String> {
    hybrid_with(|node| {
        node.ipfs
            .dht_findprovs(cid)
            .map(|provs| {
                provs
                    .into_iter()
                    .map(|(id, addrs)| format!("{}: {}", id, addrs.join(", ")))
                    .collect()
            })
            .ok()
    })
    .flatten()
    .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_name_publish(cid: &str, lifetime_sec: i64) -> String {
    hybrid_with(|node| node.ipfs.name_publish(cid, lifetime_sec).ok())
        .flatten()
        .unwrap_or_default()
}

#[uniffi::export]
pub fn ipfs_name_resolve(name: &str) -> String {
    hybrid_with(|node| node.ipfs.name_resolve(name).ok())
        .flatten()
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Git API
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn git_clone(url: &str, path: &str, bare: bool) -> bool {
    kubo_rs::git_clone(url, path, bare).is_ok()
}

#[uniffi::export]
pub fn git_init(path: &str, bare: bool) -> bool {
    kubo_rs::git_init(path, bare).is_ok()
}

#[uniffi::export]
pub fn git_head(path: &str) -> String {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.head().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[uniffi::export]
pub fn git_branches(path: &str) -> Vec<String> {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.branches().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[uniffi::export]
pub fn git_remotes(path: &str) -> Vec<String> {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.remotes().unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

#[uniffi::export]
pub fn git_is_bare(path: &str) -> bool {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.is_bare().unwrap_or(false),
        Err(_) => false,
    }
}

#[uniffi::export]
pub fn git_create_branch(path: &str, name: &str, commit_hash: &str) -> bool {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.create_branch(name, commit_hash).is_ok(),
        Err(_) => false,
    }
}

#[uniffi::export]
pub fn git_commit_message(path: &str, hash: &str) -> String {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.commit_message(hash).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[uniffi::export]
pub fn git_status(path: &str) -> String {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.status().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[uniffi::export]
pub fn git_diff_trees(path: &str, old_hash: &str, new_hash: &str) -> String {
    match kubo_rs::Repository::open(path) {
        Ok(repo) => repo.diff_trees(old_hash, new_hash).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Nostr helpers
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn nostr_generate_key() -> String {
    kubo_rs::nostr_generate_key().unwrap_or_default()
}

#[uniffi::export]
pub fn nostr_get_public_key(sk: &str) -> String {
    kubo_rs::nostr_get_public_key(sk).unwrap_or_default()
}

#[uniffi::export]
pub fn nostr_event_sign(sk: &str, content: &str, kind: i32) -> String {
    kubo_rs::nostr_event_sign(sk, content, kind).unwrap_or_default()
}

#[uniffi::export]
pub fn nostr_event_verify(event_json: &str) -> bool {
    kubo_rs::nostr_event_verify(event_json).unwrap_or(false)
}

#[uniffi::export]
pub fn nostr_relay_connect(url: &str) -> u64 {
    kubo_rs::nostr_relay_connect(url).unwrap_or(0)
}

#[uniffi::export]
pub fn nostr_relay_close(handle: u64) -> bool {
    kubo_rs::nostr_relay_close(handle).is_ok()
}

#[uniffi::export]
pub fn nostr_relay_publish(handle: u64, event_json: &str) -> bool {
    kubo_rs::nostr_relay_publish(handle, event_json).is_ok()
}

#[uniffi::export]
pub fn nostr_relay_subscribe(handle: u64, filter_json: &str) -> u64 {
    kubo_rs::nostr_relay_subscribe(handle, filter_json).unwrap_or(0)
}

#[uniffi::export]
pub fn nostr_relay_drain(sub_handle: u64) -> Option<String> {
    kubo_rs::nostr_relay_drain(sub_handle).ok().flatten()
}

#[uniffi::export]
pub fn nostr_relay_unsubscribe(sub_handle: u64) -> bool {
    kubo_rs::nostr_relay_unsubscribe(sub_handle).is_ok()
}

// ---------------------------------------------------------------------------
// Phase 15: Typed GossipSub Messages
// ---------------------------------------------------------------------------

#[uniffi::export]
pub fn hybrid_broadcast_typed(
    msg: HybridMessage,
    secret_key: &str,
    relay_handle: Option<u64>,
    gossip_topic: Option<String>,
) -> Result<String, RustyError> {
    let inner: kubo_rs::p2p_messages::HybridMessage = msg.into();
    let event = hybrid_with(|node| {
        node.broadcast_typed(inner, secret_key, relay_handle, gossip_topic.as_deref())
    })
    .ok_or_else(|| RustyError::Generic {
        msg: "hybrid node not started".to_string(),
    })??;
    Ok(event)
}

#[uniffi::export]
pub fn hybrid_drain_typed(relay_sub_handle: Option<u64>) -> Result<Vec<HybridMessage>, RustyError> {
    let messages: Vec<HybridMessage> = hybrid_with(|node| node.drain_typed(relay_sub_handle))
        .ok_or_else(|| RustyError::Generic {
            msg: "hybrid node not started".to_string(),
        })??
        .into_iter()
        .map(|m| m.into())
        .collect();
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::rust_hello;

    #[test]
    fn rust_hello_reports_kubo_roundtrip() {
        let message = rust_hello();
        assert!(
            message.contains("kubo-rs"),
            "message should mention kubo-rs"
        );
        assert!(
            message.contains("round-trip"),
            "message should mention the round-trip"
        );
    }
}
