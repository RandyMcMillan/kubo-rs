use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

uniffi::setup_scaffolding!();

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
