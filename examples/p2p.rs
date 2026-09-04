use std::env;

use kubo_rs::{Node, init_repo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = env::temp_dir();
    let repo_a = tmp.join("kubo-rs-p2p-a");
    let repo_b = tmp.join("kubo-rs-p2p-b");

    if !repo_a.join("config").exists() {
        println!("initializing repo_a...");
        init_repo(&repo_a)?;
    }
    if !repo_b.join("config").exists() {
        println!("initializing repo_b...");
        init_repo(&repo_b)?;
    }

    println!("starting nodes...");
    let node_a = Node::start(&repo_a, true)?;
    let node_b = Node::start(&repo_b, true)?;

    let peer_id_a = node_a.peer_id()?;
    let addrs_a = node_a.listening_addrs()?;
    println!("node_a peer id: {peer_id_a}");
    println!("node_a addresses: {addrs_a:?}");

    let dial_addr = format!("{}/p2p/{}", addrs_a[0], peer_id_a);
    println!("connecting node_b -> node_a at {dial_addr}");
    node_b.connect(&dial_addr)?;
    println!("connected.");

    let data = b"hello from node_a via bitswap";
    let cid = node_a.add_bytes(data)?;
    println!("node_a added: cid={cid}");

    let fetched = node_b.cat(&cid)?;
    println!("node_b fetched: {}", String::from_utf8_lossy(&fetched));
    assert_eq!(fetched, data);

    println!("stopping nodes...");
    node_a.stop()?;
    node_b.stop()?;
    println!("done.");

    Ok(())
}
