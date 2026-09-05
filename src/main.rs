use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use kubo_rs::{Node, init_repo, version};

#[derive(Parser)]
#[command(name = "kubo-rs")]
#[command(about = "Rust CLI for Kubo via FFI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// IPFS commands
    Ipfs {
        #[command(subcommand)]
        command: IpfsCommands,
    },
    /// P2P networking commands (via Kubo node)
    P2p {
        #[command(subcommand)]
        command: P2pCommands,
    },
    /// Standalone libp2p commands
    Libp2p {
        #[command(subcommand)]
        command: Libp2pCommands,
    },
    /// Nostr bridge and relay commands
    Nostr {
        #[command(subcommand)]
        command: NostrCommands,
    },
    /// Git commands
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
}

#[derive(Subcommand)]
enum IpfsCommands {
    /// Initialize a new IPFS repo
    Init {
        /// Path to the repo
        #[arg(default_value = ".ipfs")]
        path: PathBuf,
    },
    /// Print the Kubo version
    Version,
    /// Start a node and print its peer ID
    PeerId {
        /// Path to the repo
        #[arg(default_value = ".ipfs")]
        path: PathBuf,
        /// Start in online mode
        #[arg(long)]
        online: bool,
    },
    /// Add a file to IPFS and print the CID
    Add {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// File to add
        file: PathBuf,
    },
    /// Retrieve IPFS content by CID and write to stdout
    Cat {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID to retrieve
        cid: String,
    },
    /// Put a raw block into the blockstore
    BlockPut {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// File to store as a raw block
        file: PathBuf,
    },
    /// Get a raw block from the blockstore
    BlockGet {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID of the block
        cid: String,
    },
    /// Print the size of a raw block
    BlockStat {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID of the block
        cid: String,
    },
    /// Run a persistent IPFS node (daemon)
    Daemon {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Run in online mode
        #[arg(long)]
        online: bool,
        /// Start the HTTP API server on this multiaddr
        #[arg(long, default_value = "/ip4/127.0.0.1/tcp/5001")]
        api: String,
    },
    /// Read or write config values
    Config {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Interpret value as JSON
        #[arg(long)]
        json: bool,
        /// Config key path, e.g. API.HTTPHeaders.Access-Control-Allow-Origin
        key: String,
        /// Value to set (omit to read current value)
        value: Option<String>,
    },
    /// Pin a CID or path
    PinAdd {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID or /ipfs/… path to pin
        cid: String,
        /// Pin recursively (default)
        #[arg(long, default_value = "true")]
        recursive: bool,
    },
    /// Unpin a CID or path
    PinRm {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID or /ipfs/… path to unpin
        cid: String,
        /// Unpin recursively
        #[arg(long, default_value = "true")]
        recursive: bool,
    },
    /// List pinned objects
    PinLs {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
    },
    /// Publish an IPNS name
    NamePublish {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID or /ipfs/… path to publish
        cid: String,
        /// Record lifetime in seconds
        #[arg(short, long, default_value = "86400")]
        lifetime: i64,
    },
    /// Resolve an IPNS name
    NameResolve {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// IPNS name to resolve (e.g. /ipns/… or peer ID)
        name: String,
    },
}

#[derive(Subcommand)]
enum P2pCommands {
    /// Print the peer ID
    PeerId {
        /// Path to the repo
        #[arg(default_value = ".ipfs")]
        path: PathBuf,
        /// Start in online mode
        #[arg(long)]
        online: bool,
    },
    /// Connect to a peer by multiaddr
    Connect {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Multiaddr of the peer to connect to
        addr: String,
    },
    /// Print listening addresses
    Listen {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
    },
    /// Disconnect from a peer by multiaddr
    Disconnect {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Multiaddr of the peer to disconnect from
        addr: String,
    },
    /// Find a peer's addresses via DHT
    DhtFindpeer {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Peer ID to look up
        peer_id: String,
    },
    /// Find providers for a CID via DHT
    DhtFindprovs {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// CID to find providers for
        cid: String,
    },
}

#[derive(Subcommand)]
enum Libp2pCommands {
    /// Create a new libp2p host and print its peer ID
    Host,
    /// Print listening addresses of a host
    Listen,
    /// Connect to a peer by multiaddr
    Connect {
        /// Multiaddr of the peer to connect to
        addr: String,
    },
}

#[derive(Subcommand)]
enum NostrCommands {
    /// Generate a new Nostr keypair
    Keygen,
    /// Sign a Nostr event and print the JSON
    Sign {
        /// Event content
        content: String,
        /// Event kind
        #[arg(short, long, default_value = "1")]
        kind: i32,
        /// Secret key (hex). If omitted, a new key is generated.
        #[arg(short, long)]
        secret: Option<String>,
    },
    /// Verify a Nostr event JSON string
    Verify {
        /// Event JSON
        event: String,
    },
    /// Start a hybrid Nostr relay with IPFS backend
    Relay {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Run in online mode
        #[arg(long)]
        online: bool,
    },
    /// Publish a Nostr event (signs and outputs JSON)
    Publish {
        /// Event content
        content: String,
        /// Event kind
        #[arg(short, long, default_value = "1")]
        kind: i32,
        /// Secret key (hex). If omitted, a new key is generated.
        #[arg(short, long)]
        secret: Option<String>,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// Initialize a new Git repository
    Init {
        /// Path to the repository
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Create a bare repository
        #[arg(long)]
        bare: bool,
    },
    /// Clone a remote repository
    Clone {
        /// URL to clone from
        url: String,
        /// Destination path
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Clone as a bare repository
        #[arg(long)]
        bare: bool,
    },
    /// Open a repository and print HEAD
    Head {
        /// Path to the repository
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ipfs { command } => match command {
            IpfsCommands::Init { path } => {
                init_repo(&path)?;
                println!("initialized repo at {}", path.display());
            }
            IpfsCommands::Version => {
                println!("{}", version());
            }
            IpfsCommands::PeerId { path, online } => {
                let node = Node::start(&path, online)?;
                println!("{}", node.peer_id()?);
                node.stop()?;
            }
            IpfsCommands::Add { repo, file } => {
                let data = fs::read(&file)?;
                let node = Node::start(&repo, false)?;
                let cid = node.add_bytes(&data)?;
                println!("{cid}");
                node.stop()?;
            }
            IpfsCommands::Cat { repo, cid } => {
                let node = Node::start(&repo, false)?;
                let data = node.cat(&cid)?;
                std::io::Write::write_all(&mut std::io::stdout(), &data)?;
                node.stop()?;
            }
            IpfsCommands::BlockPut { repo, file } => {
                let data = fs::read(&file)?;
                let node = Node::start(&repo, false)?;
                let cid = node.block_put(&data)?;
                println!("{cid}");
                node.stop()?;
            }
            IpfsCommands::BlockGet { repo, cid } => {
                let node = Node::start(&repo, false)?;
                let data = node.block_get(&cid)?;
                std::io::Write::write_all(&mut std::io::stdout(), &data)?;
                node.stop()?;
            }
            IpfsCommands::BlockStat { repo, cid } => {
                let node = Node::start(&repo, false)?;
                let size = node.block_stat(&cid)?;
                println!("{size}");
                node.stop()?;
            }
            IpfsCommands::Daemon { repo, online, api } => {
                let node = Node::start(&repo, online)?;
                let api_addr = node.start_api(&api)?;
                println!("daemon started");
                println!("peer id: {}", node.peer_id()?);
                println!("api: {api_addr}");
                println!("listening addrs:");
                for addr in node.listening_addrs()? {
                    println!("  {addr}");
                }
                println!("press Ctrl+C to stop");

                let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let r = running.clone();
                ctrlc::set_handler(move || {
                    r.store(false, std::sync::atomic::Ordering::SeqCst);
                })?;

                while running.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                println!("shutting down...");
                node.stop()?;
                println!("daemon stopped");
            }
            IpfsCommands::Config {
                repo,
                json,
                key,
                value,
            } => {
                let config_path = repo.join("config");
                if !config_path.exists() {
                    return Err(format!(
                        "config not found at {}. run `init` first.",
                        config_path.display()
                    )
                    .into());
                }

                let mut config: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&config_path)?)?;

                if let Some(val_str) = value {
                    let new_val = if json {
                        serde_json::from_str(&val_str)?
                    } else {
                        serde_json::Value::String(val_str)
                    };
                    config_set(&mut config, &key, new_val)?;
                    fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
                } else {
                    let current = config_get(&config, &key)
                        .ok_or_else(|| format!("config key '{}' not found", key))?;
                    println!("{}", serde_json::to_string_pretty(current)?);
                }
            }
            IpfsCommands::PinAdd {
                repo,
                cid,
                recursive,
            } => {
                let node = Node::start(&repo, false)?;
                node.pin_add(&cid, recursive)?;
                println!("pinned {cid}");
                node.stop()?;
            }
            IpfsCommands::PinRm {
                repo,
                cid,
                recursive,
            } => {
                let node = Node::start(&repo, false)?;
                node.pin_rm(&cid, recursive)?;
                println!("unpinned {cid}");
                node.stop()?;
            }
            IpfsCommands::PinLs { repo } => {
                let node = Node::start(&repo, false)?;
                for (path, typ) in node.pin_ls()? {
                    println!("{path}\t{typ}");
                }
                node.stop()?;
            }
            IpfsCommands::NamePublish {
                repo,
                cid,
                lifetime,
            } => {
                let node = Node::start(&repo, true)?;
                let name = node.name_publish(&cid, lifetime)?;
                println!("{name}");
                node.stop()?;
            }
            IpfsCommands::NameResolve { repo, name } => {
                let node = Node::start(&repo, true)?;
                let resolved = node.name_resolve(&name)?;
                println!("{resolved}");
                node.stop()?;
            }
        },
        Commands::P2p { command } => match command {
            P2pCommands::PeerId { path, online } => {
                let node = Node::start(&path, online)?;
                println!("{}", node.peer_id()?);
                node.stop()?;
            }
            P2pCommands::Connect { repo, addr } => {
                let node = Node::start(&repo, true)?;
                node.connect(&addr)?;
                println!("connected to {addr}");
                node.stop()?;
            }
            P2pCommands::Listen { repo } => {
                let node = Node::start(&repo, true)?;
                for addr in node.listening_addrs()? {
                    println!("{addr}");
                }
                node.stop()?;
            }
            P2pCommands::Disconnect { repo, addr } => {
                let node = Node::start(&repo, true)?;
                node.disconnect(&addr)?;
                println!("disconnected from {addr}");
                node.stop()?;
            }
            P2pCommands::DhtFindpeer { repo, peer_id } => {
                let node = Node::start(&repo, true)?;
                let (id, addrs) = node.dht_findpeer(&peer_id)?;
                println!("{id}");
                for addr in addrs {
                    println!("  {addr}");
                }
                node.stop()?;
            }
            P2pCommands::DhtFindprovs { repo, cid } => {
                let node = Node::start(&repo, true)?;
                for (id, addrs) in node.dht_findprovs(&cid)? {
                    println!("{id}");
                    for addr in addrs {
                        println!("  {addr}");
                    }
                }
                node.stop()?;
            }
        },
        Commands::Libp2p { command } => {
            use kubo_rs::Host;
            match command {
                Libp2pCommands::Host => {
                    let host = Host::new()?;
                    println!("{}", host.peer_id()?);
                    host.close()?;
                }
                Libp2pCommands::Listen => {
                    let host = Host::new()?;
                    for addr in host.listening_addrs()? {
                        println!("{addr}");
                    }
                    host.close()?;
                }
                Libp2pCommands::Connect { addr } => {
                    let host = Host::new()?;
                    host.connect(&addr)?;
                    println!("connected to {addr}");
                    host.close()?;
                }
            }
        }
        Commands::Nostr { command } => match command {
            NostrCommands::Keygen => {
                let sk = kubo_rs::nostr_generate_key()?;
                let pk = kubo_rs::nostr_get_public_key(&sk)?;
                println!("secret: {sk}");
                println!("public: {pk}");
            }
            NostrCommands::Sign {
                content,
                kind,
                secret,
            } => {
                let sk = secret.unwrap_or_else(|| {
                    kubo_rs::nostr_generate_key().expect("key generation failed")
                });
                let event = kubo_rs::nostr_event_sign(&sk, &content, kind)?;
                println!("{event}");
            }
            NostrCommands::Verify { event } => {
                let valid = kubo_rs::nostr_event_verify(&event)?;
                println!("{valid}");
            }
            NostrCommands::Relay { repo, online } => {
                let node = Node::start(&repo, online)?;
                println!("nostr relay started");
                println!("peer id: {}", node.peer_id()?);
                println!("listening addrs:");
                for addr in node.listening_addrs()? {
                    println!("  {addr}");
                }
                println!("press Ctrl+C to stop");

                let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let r = running.clone();
                ctrlc::set_handler(move || {
                    r.store(false, std::sync::atomic::Ordering::SeqCst);
                })?;

                while running.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                println!("shutting down...");
                node.stop()?;
                println!("nostr relay stopped");
            }
            NostrCommands::Publish {
                content,
                kind,
                secret,
            } => {
                let sk = secret.unwrap_or_else(|| {
                    kubo_rs::nostr_generate_key().expect("key generation failed")
                });
                let event = kubo_rs::nostr_event_sign(&sk, &content, kind)?;
                println!("{event}");
            }
        },
        Commands::Git { command } => match command {
            GitCommands::Init { path, bare } => {
                kubo_rs::git_init(path.to_str().ok_or("invalid path")?, bare)?;
                if bare {
                    println!("initialized bare repo at {}", path.display());
                } else {
                    println!("initialized repo at {}", path.display());
                }
            }
            GitCommands::Clone { url, path, bare } => {
                kubo_rs::git_clone(&url, path.to_str().ok_or("invalid path")?, bare)?;
                println!("cloned into {}", path.display());
            }
            GitCommands::Head { path } => {
                let repo = kubo_rs::Repository::open(&path)?;
                let head = repo.head()?;
                println!("{head}");
                repo.close()?;
            }
        },
    }

    Ok(())
}

fn config_get<'a>(config: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    let mut current = config;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn config_set(
    config: &mut serde_json::Value,
    key: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() {
        return Err("empty key".to_string());
    }

    let mut current = config;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            current[part] = value;
            return Ok(());
        }
        match current {
            serde_json::Value::Object(map) => {
                if !map.get(*part).map(|v| v.is_object()).unwrap_or(false) {
                    map.insert(part.to_string(), serde_json::json!({}));
                }
                current = map.get_mut(*part).unwrap();
            }
            _ => return Err(format!("cannot navigate into non-object at '{}'", part)),
        }
    }
    Ok(())
}
