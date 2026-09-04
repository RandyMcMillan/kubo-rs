use std::env;
use std::path::PathBuf;

use kubo_rs::{Node, init_repo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        let tmp = std::env::temp_dir();
        tmp.join("kubo-rs-example")
    });

    println!("repo path: {}", repo.display());

    if !repo.join("config").exists() {
        println!("initializing repo...");
        init_repo(&repo)?;
    }

    println!("starting node...");
    let node = Node::start(&repo, true)?;

    println!("version: {}", kubo_rs::version());
    println!("peer id: {}", node.peer_id()?);

    let addrs = node.listening_addrs()?;
    println!("listening addresses:");
    for addr in &addrs {
        println!("  {addr}");
    }

    let data = b"hello from kubo-rs";
    let cid = node.add_bytes(data)?;
    println!("added bytes: cid = {cid}");

    let fetched = node.cat(&cid)?;
    println!("fetched bytes: {}", String::from_utf8_lossy(&fetched));
    assert_eq!(fetched, data);

    println!("stopping node...");
    node.stop()?;
    println!("done.");

    Ok(())
}
