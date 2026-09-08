use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

uniffi::setup_scaffolding!();

static P2P_HOST: Mutex<Option<kubo_rs::Host>> = Mutex::new(None);

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
                eprintln!("p2p host start failed: {err}");
                return None;
            }
        };
        *guard = Some(host);
    }

    let host = guard.as_ref()?;
    Some(f(host))
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
    p2p_host(|host| host.peer_id().ok()).flatten().unwrap_or_default()
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
pub fn p2p_gossip_publish(message: &str) -> bool {
    p2p_host(|host| host.gossip_publish(message).is_ok()).unwrap_or(false)
}

#[uniffi::export]
pub fn p2p_gossip_drain() -> Vec<String> {
    p2p_host(|host| host.gossip_drain().ok())
        .flatten()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::rust_hello;

    #[test]
    fn rust_hello_reports_kubo_roundtrip() {
        let message = rust_hello();
        assert!(message.contains("kubo-rs"), "message should mention kubo-rs");
        assert!(message.contains("round-trip"), "message should mention the round-trip");
    }
}
