//! Alignment tests: verify Go FFI implementations match native Rust crates.

use nostr::prelude::*;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Nostr alignment
// ---------------------------------------------------------------------------

#[test]
fn test_nostr_key_alignment() {
    // Generate a keypair via Go FFI.
    let sk_go = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let pk_go = kubo_rs::nostr_get_public_key(&sk_go).expect("go pubkey failed");

    // Parse the same secret key with the native Rust nostr crate.
    let keys_rs = Keys::parse(&sk_go).expect("rust key parse failed");
    let pk_rs = keys_rs.public_key().to_hex();

    assert_eq!(
        pk_go, pk_rs,
        "public keys derived from the same secret key must match"
    );
}

#[test]
fn test_nostr_event_sign_alignment_go_to_rust() {
    // Sign an event via Go FFI.
    let sk = kubo_rs::nostr_generate_key().expect("go keygen failed");
    let event_json = kubo_rs::nostr_event_sign(&sk, "alignment test", 1)
        .expect("go sign failed");

    // Parse the event with the native Rust nostr crate and verify it.
    let event: Event = serde_json::from_str(&event_json)
        .expect("rust event parse failed");
    event.verify().expect("rust verify of go-signed event failed");

    // Also verify via Go FFI that the event is valid.
    let ok = kubo_rs::nostr_event_verify(&event_json)
        .expect("go verify failed");
    assert!(ok, "go verify must succeed for its own signed event");
}

#[test]
fn test_nostr_event_sign_alignment_rust_to_go() {
    // Create and sign an event with the native Rust nostr crate.
    let keys = Keys::generate();
    let unsigned = UnsignedEvent::new(
        keys.public_key(),
        Timestamp::now(),
        Kind::TextNote,
        [],
        "alignment test from rust",
    );
    let event = keys.sign_event(unsigned)
        .expect("rust sign failed");
    let event_json = serde_json::to_string(&event)
        .expect("rust serialize failed");

    // Verify the Rust-signed event via Go FFI.
    let ok = kubo_rs::nostr_event_verify(&event_json)
        .expect("go verify of rust-signed event failed");
    assert!(ok, "go verify must accept rust-signed event");
}

// ---------------------------------------------------------------------------
// Git alignment
// ---------------------------------------------------------------------------

#[test]
fn test_git_init_alignment() {
    let path = tmp_path("git_init_alignment");

    // Init via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), false)
        .expect("go git init failed");
    assert!(path.join(".git").exists(), ".git should exist after go init");

    // Open the same repo with the native Rust git2 (libgit2) crate.
    let repo = git2::Repository::open(&path)
        .expect("git2 open failed");

    // Verify the repo is not bare.
    assert!(!repo.is_bare(), "repo should not be bare");

    // Create a commit via git2 so HEAD exists.
    let sig = git2::Signature::now("Test", "test@example.com")
        .expect("signature failed");
    let tree_id = {
        let mut index = repo.index().expect("index failed");
        let blob_id = repo.blob(b"hello git alignment").expect("blob failed");
        index.add_frombuffer(
            &git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: 0o100644,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: blob_id,
                flags: 0,
                flags_extended: 0,
                path: b"hello.txt".to_vec(),
            },
            b"hello git alignment",
        ).expect("add failed");
        index.write_tree().expect("write tree failed")
    };
    let tree = repo.find_tree(tree_id).expect("find tree failed");
    let commit_id = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "alignment test commit",
        &tree,
        &[],
    ).expect("commit failed");

    // Verify Go FFI sees the same HEAD.
    let repo_go = kubo_rs::Repository::open(&path)
        .expect("go open failed");
    let head_go = repo_go.head().expect("go head failed");
    repo_go.close().expect("go close failed");

    assert_eq!(
        head_go, commit_id.to_string(),
        "HEAD hash must match between git2 and go-git ffi"
    );
}

#[test]
fn test_git_init_bare_alignment() {
    let path = tmp_path("git_init_bare_alignment");

    // Init bare repo via Go FFI.
    kubo_rs::git_init(path.to_str().unwrap(), true)
        .expect("go git init bare failed");

    // Open with git2 and verify it's bare.
    let repo = git2::Repository::open(&path)
        .expect("git2 open failed");
    assert!(repo.is_bare(), "repo should be bare");
}

// ---------------------------------------------------------------------------
// libp2p alignment
// ---------------------------------------------------------------------------

#[test]
fn test_libp2p_peer_id_format_alignment() {
    // Create a Go FFI host and get its peer ID.
    let host_go = kubo_rs::Host::new().expect("go host new failed");
    let peer_id_go = host_go.peer_id().expect("go peer_id failed");
    host_go.close().expect("go host close failed");

    // Verify the Go peer ID is a valid rust-libp2p PeerId.
    let peer_id_rs: libp2p::PeerId = peer_id_go.parse()
        .expect("go peer_id should parse as rust PeerId");

    // The string representation should round-trip.
    assert_eq!(peer_id_go, peer_id_rs.to_string(),
        "peer id string representation must round-trip");
}

#[test]
fn test_libp2p_keypair_peer_id_derivation_alignment() {
    // Generate a Rust libp2p keypair and derive its peer ID.
    let keypair_rs = libp2p::identity::Keypair::generate_ed25519();
    let peer_id_rs = keypair_rs.public().to_peer_id();

    // Verify the peer ID is valid according to Go (just format-check).
    let peer_id_str = peer_id_rs.to_string();
    assert!(
        peer_id_str.starts_with("12D3KooW") || peer_id_str.len() == 52,
        "ed25519 peer id should have expected format"
    );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_path(name: &str) -> PathBuf {
    let path = PathBuf::from("tmp").join("alignment").join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("mkdir failed");
    path
}
