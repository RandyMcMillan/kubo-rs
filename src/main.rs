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
        Commands::Init { path } => {
            init_repo(&path)?;
            println!("initialized repo at {}", path.display());
        }
        Commands::Version => {
            println!("{}", version());
        }
        Commands::PeerId { path, online } => {
            let node = Node::start(&path, online)?;
            println!("{}", node.peer_id()?);
            node.stop()?;
        }
        Commands::Add { repo, file } => {
            let data = fs::read(&file)?;
            let node = Node::start(&repo, false)?;
            let cid = node.add_bytes(&data)?;
            println!("{cid}");
            node.stop()?;
        }
        Commands::Cat { repo, cid } => {
            let node = Node::start(&repo, false)?;
            let data = node.cat(&cid)?;
            std::io::Write::write_all(&mut std::io::stdout(), &data)?;
            node.stop()?;
        }
        Commands::BlockPut { repo, file } => {
            let data = fs::read(&file)?;
            let node = Node::start(&repo, false)?;
            let cid = node.block_put(&data)?;
            println!("{cid}");
            node.stop()?;
        }
        Commands::BlockGet { repo, cid } => {
            let node = Node::start(&repo, false)?;
            let data = node.block_get(&cid)?;
            std::io::Write::write_all(&mut std::io::stdout(), &data)?;
            node.stop()?;
        }
        Commands::BlockStat { repo, cid } => {
            let node = Node::start(&repo, false)?;
            let size = node.block_stat(&cid)?;
            println!("{size}");
            node.stop()?;
        }
    }

    Ok(())
}
