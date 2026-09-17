mod commands;
mod model;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Odd — a weird way to deal with the filesystem.
///
/// This binary intentionally exposes only the first vertical slice:
/// init, remember, show, resurrect. Everything else in the design notes
/// gets added once this slice is solid, one command at a time.
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
    /// Capture a file into the store.
    Remember {
        path: PathBuf,
        #[arg(long = "as")]
        as_name: Option<String>,
        #[arg(long = "ns")]
        namespace: Option<String>,
    },
    /// Show a remembered object's detail.
    Show {
        target: String,
        #[arg(long = "ns")]
        namespace: Option<String>,
    },
    /// Materialize a remembered object back to disk.
    Resurrect {
        target: String,
        #[arg(long)]
        dest: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long = "ns")]
        namespace: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { store } => commands::init::run(None, store),
        Commands::Remember {
            path,
            as_name,
            namespace,
        } => commands::remember::run(path, as_name, namespace),
        Commands::Show { target, namespace } => commands::show::run(target, namespace),
        Commands::Resurrect {
            target,
            dest,
            force,
            namespace,
        } => commands::resurrect::run(target, dest, force, namespace),
    }
}
