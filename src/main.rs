mod commands;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Odd — a weird way to deal with the filesystem.
#[derive(Parser)]
#[command(name = "odd", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a workspace here, bound to a store.
    Init {
        #[arg(long)]
        store: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { store } => commands::init::run(None, store),
    }
}
