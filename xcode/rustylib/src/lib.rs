use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

uniffi::setup_scaffolding!();

static P2P_HOST: std::sync::OnceLock<Mutex<kubo_rs::Host>> = std::sync::OnceLock::new();

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

fn p2p_host() -> Result<&'static Mutex<kubo_rs::Host>, kubo_rs::Error> {
    P2P_HOST
        .get_or_try_init(|| kubo_rs::Host::new().map(Mutex::new))
        .map_err(|err| err)
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
pub fn p2p_start() -> Result<String, kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.peer_id()
}

#[uniffi::export]
pub fn p2p_peer_id() -> Result<String, kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.peer_id()
}

#[uniffi::export]
pub fn p2p_listening_addrs() -> Result<Vec<String>, kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.listening_addrs()
}

#[uniffi::export]
pub fn p2p_connect(addr: &str) -> Result<(), kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.connect(addr)
}

#[uniffi::export]
pub fn p2p_ping(peer_id: &str) -> Result<i64, kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.ping(peer_id)
}

#[uniffi::export]
pub fn p2p_protocols() -> Result<Vec<String>, kubo_rs::Error> {
    let host = p2p_host()?;
    let host = host
        .lock()
        .map_err(|err| kubo_rs::Error::Go(format!("p2p host lock poisoned: {err}")))?;
    host.protocols()
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
