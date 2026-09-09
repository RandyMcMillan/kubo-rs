use std::env;
use std::thread;
use std::time::Duration;

use kubo_rs::{HybridNode, init_repo, nostr_event_sign, nostr_generate_key, nostr_get_public_key};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = env::temp_dir();
    let repo_a = tmp.join("kubo-rs-hybrid-mt-a");
    let repo_b = tmp.join("kubo-rs-hybrid-mt-b");

    for repo in [&repo_a, &repo_b] {
        if !repo.join("config").exists() {
            println!("initialising repo at {} ...", repo.display());
            init_repo(repo)?;
        }
    }

    println!("starting hybrid node A ...");
    let node_a = HybridNode::start(&repo_a, true)?;
    let peer_id_a = node_a.p2p.peer_id()?;
    let addrs_a = node_a.p2p.listening_addrs()?;
    println!("node A p2p peer id: {peer_id_a}");
    println!("node A p2p addrs:   {addrs_a:?}");

    println!("starting hybrid node B ...");
    let node_b = HybridNode::start(&repo_b, true)?;
    let peer_id_b = node_b.p2p.peer_id()?;
    println!("node B p2p peer id: {peer_id_b}");

    // Connect B -> A so they can gossip
    if let Some(addr) = addrs_a.first() {
        let dial = format!("{}/p2p/{}", addr, peer_id_a);
        println!("connecting B -> A at {dial} ...");
        node_b.p2p.connect(&dial)?;
        println!("connected.");
    }

    // Join a custom topic on both nodes
    let topic = "kubo-hybrid-demo";
    println!("joining topic '{topic}' on both nodes ...");
    node_a.p2p.gossip_join(topic)?;
    node_b.p2p.gossip_join(topic)?;

    // Give GossipSub a moment to establish mesh
    thread::sleep(Duration::from_millis(500));

    // Generate a Nostr key and sign an event
    let sk = nostr_generate_key()?;
    let pk = nostr_get_public_key(&sk)?;
    let event = nostr_event_sign(&sk, "hello multi-topic hybrid", 1)?;
    println!("signed event (kind 1) from {pk}");

    // Publish to the custom topic from node A
    println!("publishing event to topic '{topic}' from node A ...");
    node_a.p2p.gossip_publish_to(topic, &event)?;

    // Drain on node B until we see it
    println!("draining on node B ...");
    let mut found = false;
    for attempt in 1..=10 {
        let events = node_b.p2p.gossip_drain_nostr()?;
        if !events.is_empty() {
            for evt in &events {
                println!(
                    "  received event on attempt {attempt}: {}",
                    &evt[..evt.len().min(80)]
                );
            }
            found = true;
            break;
        }
        thread::sleep(Duration::from_millis(300));
    }

    if !found {
        println!("warning: did not receive event within timeout (gossip may need more time).");
    }

    println!("stopping nodes ...");
    node_a.stop()?;
    node_b.stop()?;
    println!("done.");

    Ok(())
}
