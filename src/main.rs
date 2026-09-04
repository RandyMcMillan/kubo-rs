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
        },
    }

    Ok(())
}
