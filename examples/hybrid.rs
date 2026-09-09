use std::env;

use kubo_rs::{
    HybridNode, init_repo, nostr_generate_key, nostr_get_public_key,
    nostr_relay_connect, nostr_relay_subscribe,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = env::temp_dir();
    let repo = tmp.join("kubo-rs-hybrid");

    println!("repo path: {}", repo.display());

    if !repo.join("config").exists() {
        println!("initializing repo...");
        init_repo(&repo)?;
    }

    println!("starting hybrid node...");
    let node = HybridNode::start(&repo, true)?;

    println!("ipfs peer id: {}", node.ipfs.peer_id()?);
    println!("p2p peer id:  {}", node.p2p.peer_id()?);

    let sk = nostr_generate_key()?;
    let pk = nostr_get_public_key(&sk)?;
    println!("nostr pubkey: {pk}");

    // Create a test file and publish it as a NIP-94 event.
    let file = tmp.join("hybrid-demo.txt");
    std::fs::write(&file, b"hello from hybrid protocol")?;
    println!("publishing file: {}", file.display());

    let cid = node.publish_file(&file, &sk, None, None)?;
    println!("published to ipfs: cid={cid}");

    // Verify round-trip via IPFS.
    let fetched = node.ipfs.cat(&cid)?;
    assert_eq!(fetched, b"hello from hybrid protocol");
    println!("ipfs round-trip verified.");

    // Drain gossip for our own event.
    println!("draining gossip events...");
    let events = node.drain_events(None)?;
    println!("drained {} event(s)", events.len());
    for evt in &events {
        println!("  - {}", &evt[..evt.len().min(120)]);
    }

    // Optional: relay publish/drain demo (requires wss://relay.damus.io).
    if env::var("HYBRID_RELAY_DEMO").is_ok() {
        println!("relay demo enabled — connecting to wss://relay.damus.io...");
        let relay = nostr_relay_connect("wss://relay.damus.io")?;

        let filter = format!(r#"{{"kinds":[1063],"authors":["{}"],"limit":1}}"#, pk);
        let sub = nostr_relay_subscribe(relay, &filter)?;

        // Re-publish via relay so we can drain it back.
        let cid2 = node.publish_file(&file, &sk, Some(relay), None)?;
        println!("re-published via relay: cid={cid2}");

        let mut found = false;
        for _ in 0..5 {
            let relay_events = node.drain_events(Some(sub))?;
            if !relay_events.is_empty() {
                println!("received {} event(s) from relay+ gossip", relay_events.len());
                found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        if !found {
            println!("no relay event received within timeout (relay may be slow).");
        }
    }

    println!("stopping hybrid node...");
    node.stop()?;
    println!("done.");

    Ok(())
}
