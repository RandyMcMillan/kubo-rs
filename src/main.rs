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
    /// P2P networking commands
    P2p {
        #[command(subcommand)]
        command: P2pCommands,
    },
    /// Nostr bridge and relay commands
    Nostr {
        #[command(subcommand)]
        command: NostrCommands,
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
}

#[derive(Subcommand)]
enum NostrCommands {
    /// Start a hybrid Nostr relay with IPFS backend
    Relay {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Run in online mode
        #[arg(long)]
        online: bool,
    },
    /// Publish a Nostr event (requires nostr FFI extension)
    Publish {
        /// Path to the repo
        #[arg(short, long, default_value = ".ipfs")]
        repo: PathBuf,
        /// Event content to publish
        content: String,
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
            IpfsCommands::Daemon { repo, online } => {
                let node = Node::start(&repo, online)?;
                println!("daemon started");
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
        },
        Commands::Nostr { command } => match command {
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
                repo: _,
                content: _,
            } => {
                eprintln!("nostr publish is not yet implemented");
                eprintln!(
                    "it requires extending the FFI layer with nostr event signing/publishing"
                );
                std::process::exit(1);
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
