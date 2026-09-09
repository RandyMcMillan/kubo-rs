use nostr::prelude::*;

fn main() {
    println!("=== Hybrid Protocol — WASI Demo ===");
    println!("Compiles native Rust nostr crate (with C secp256k1) to wasm32-wasip1\n");

    let keys = Keys::generate();
    let pk = keys.public_key();
    println!("Public key: {}", pk.to_hex());

    let event = EventBuilder::new(Kind::TextNote, "Hello from wasm32-wasip1!")
        .finalize(&keys)
        .unwrap();
    println!("Event ID: {}", event.id.to_hex());
    println!("Signature: {}", event.sig);
    println!("Valid: {}", event.verify().is_ok());

    let nip94 = EventBuilder::new(Kind::FileMetadata, "demo.txt")
        .tag(
            Tag::parse([
                "url",
                "ipfs://QmeYpfdsesMd4U1MRMyfwot21KHGSBvHHEzPVgDBR5d2EJ",
            ])
            .unwrap(),
        )
        .tag(Tag::parse(["m", "text/plain"]).unwrap())
        .finalize(&keys)
        .unwrap();
    println!("\nNIP-94 event ID: {}", nip94.id.to_hex());
}
