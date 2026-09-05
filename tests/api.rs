//! Integration tests for the Kubo HTTP API server (embedded node).

use std::path::PathBuf;
use std::time::Duration;

fn tmp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from("tmp").join("api-test").join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn set_cors_origins(repo: &std::path::Path, origins: &[&str]) {
    let config_path = repo.join("config");
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();

    let origins_json: Vec<serde_json::Value> = origins
        .iter()
        .map(|o| serde_json::Value::String(o.to_string()))
        .collect();

    config["API"]["HTTPHeaders"]["Access-Control-Allow-Origin"] =
        serde_json::Value::Array(origins_json);
    config["API"]["HTTPHeaders"]["Access-Control-Allow-Methods"] =
        serde_json::json!(["PUT", "POST", "GET"]);

    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
}

fn wait_for_api(api_url: &str) {
    for i in 0..120 {
        match ureq::post(&format!("{}/api/v0/id", api_url)).send_empty() {
            Ok(_) => return,
            Err(e) => {
                if i >= 119 {
                    panic!("API did not start in time: {e:?}");
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn test_api_server_starts_and_id_endpoint_works() {
    let repo = tmp_dir("api_server_starts").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");

    // Convert multiaddr to HTTP URL.
    // e.g. /ip4/127.0.0.1/tcp/5001 -> http://127.0.0.1:5001
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let resp = ureq::post(&format!("{}/api/v0/id", api_url))
        .send_empty()
        .expect("id request should succeed");
    let body = resp.into_body().read_to_string().unwrap();
    assert!(
        body.contains("ID"),
        "id response should contain ID field: {}",
        body
    );

    node.stop().expect("stop should succeed");
}

#[test]
fn test_api_cors_chrome_localhost() {
    let repo = tmp_dir("api_cors_chrome").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");
    set_cors_origins(
        &repo,
        &[
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ],
    );

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let resp = ureq::post(&format!("{}/api/v0/id", api_url))
        .header("Origin", "http://localhost:8080")
        .send_empty()
        .expect("CORS request should succeed");

    let acao = resp
        .headers()
        .get("Access-Control-Allow-Origin")
        .map(|v| v.to_str().unwrap());
    assert_eq!(
        acao,
        Some("http://localhost:8080"),
        "Chrome Origin header should be reflected in ACAO"
    );

    node.stop().expect("stop should succeed");
}

#[test]
fn test_api_cors_firefox_127() {
    let repo = tmp_dir("api_cors_firefox").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");
    set_cors_origins(
        &repo,
        &[
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ],
    );

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let resp = ureq::post(&format!("{}/api/v0/id", api_url))
        .header("Origin", "http://127.0.0.1:8080")
        .send_empty()
        .expect("CORS request should succeed");

    let acao = resp
        .headers()
        .get("Access-Control-Allow-Origin")
        .map(|v| v.to_str().unwrap());
    assert_eq!(
        acao,
        Some("http://127.0.0.1:8080"),
        "Firefox Origin header should be reflected in ACAO"
    );

    node.stop().expect("stop should succeed");
}

#[test]
fn test_api_cors_safari_ipv6() {
    let repo = tmp_dir("api_cors_safari").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");
    set_cors_origins(
        &repo,
        &[
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ],
    );

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let resp = ureq::post(&format!("{}/api/v0/id", api_url))
        .header("Origin", "http://[::1]:8080")
        .send_empty()
        .expect("CORS request should succeed");

    let acao = resp
        .headers()
        .get("Access-Control-Allow-Origin")
        .map(|v| v.to_str().unwrap());
    assert_eq!(
        acao,
        Some("http://[::1]:8080"),
        "Safari IPv6 Origin header should be reflected in ACAO"
    );

    node.stop().expect("stop should succeed");
}

#[test]
fn test_api_cors_preflight_options() {
    let repo = tmp_dir("api_cors_preflight").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");
    set_cors_origins(
        &repo,
        &[
            "http://localhost:8080",
            "http://127.0.0.1:8080",
            "http://[::1]:8080",
        ],
    );

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let resp = ureq::options(&format!("{}/api/v0/id", api_url))
        .header("Origin", "http://localhost:8080")
        .header("Access-Control-Request-Method", "POST")
        .call()
        .expect("preflight request should succeed");

    let acao = resp
        .headers()
        .get("Access-Control-Allow-Origin")
        .map(|v| v.to_str().unwrap());
    let acam = resp
        .headers()
        .get("Access-Control-Allow-Methods")
        .map(|v| v.to_str().unwrap());
    assert_eq!(
        acao,
        Some("http://localhost:8080"),
        "preflight should reflect Origin in ACAO"
    );
    assert!(
        acam.map(|m| m.contains("POST")).unwrap_or(false),
        "preflight should allow POST method, got {:?}",
        acam
    );

    node.stop().expect("stop should succeed");
}

#[test]
fn test_api_cors_blocked_without_config() {
    // When CORS is not configured, the API should still respond but
    // without ACAO for cross-origin requests.
    let repo = tmp_dir("api_cors_blocked").join("repo");
    kubo_rs::init_repo(&repo).expect("init repo should succeed");
    // Do NOT set CORS origins.

    let node = kubo_rs::Node::start(&repo, false).expect("start node should succeed");
    let api_maddr = node
        .start_api("/ip4/127.0.0.1/tcp/0")
        .expect("start api should succeed");
    let api_url = maddr_to_http_url(&api_maddr);
    wait_for_api(&api_url);

    let result = ureq::post(&format!("{}/api/v0/id", api_url))
        .header("Origin", "http://localhost:8080")
        .send_empty();

    // Without explicit CORS config, cross-origin requests from a different
    // port should be rejected (403).
    assert!(
        result.is_err(),
        "cross-origin request without CORS config should be blocked"
    );
    let err = result.unwrap_err();
    assert!(
        format!("{err:?}").contains("403"),
        "expected 403 Forbidden, got {err:?}"
    );

    node.stop().expect("stop should succeed");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn maddr_to_http_url(maddr: &str) -> String {
    // Parse /ip4/127.0.0.1/tcp/5001 -> http://127.0.0.1:5001
    // Parse /ip6/::1/tcp/5001 -> http://[::1]:5001
    let parts: Vec<&str> = maddr.trim_start_matches('/').split('/').collect();
    let mut host = String::new();
    let mut port = String::new();
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "ip4" => {
                i += 1;
                host = parts[i].to_string();
            }
            "ip6" => {
                i += 1;
                host = format!("[{}]", parts[i]);
            }
            "tcp" => {
                i += 1;
                port = parts[i].to_string();
            }
            _ => {}
        }
        i += 1;
    }
    format!("http://{}:{}", host, port)
}
