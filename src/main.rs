mod commands;
mod model;
mod resolve;
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
    /// Capture a file or directory into the store.
    Remember {
        path: PathBuf,
        #[arg(long = "as")]
        as_name: Option<String>,
        #[arg(long = "ns")]
        namespace: Option<String>,
        /// Required for a directory; recurses and binds every child too.
        #[arg(short = 'r', long = "recursive")]
        recursive: bool,
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
    /// Compose existing objects into a new one, without touching disk.
    Graft {
        target_name: String,
        /// One or more "entry=source" pairs.
        entries: Vec<String>,
        #[arg(long)]
        into: Option<String>,
        #[arg(long = "ns")]
        namespace: Option<String>,
    },
    /// Remove an entry from a composed object, producing a new one.
    Ungraft {
        composed_name: String,
        entry: String,
        #[arg(long = "as")]
        as_name: Option<String>,
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
            recursive,
        } => commands::remember::run(path, as_name, namespace, recursive),
        Commands::Show { target, namespace } => commands::show::run(target, namespace),
        Commands::Resurrect {
            target,
            dest,
            force,
            namespace,
        } => commands::resurrect::run(target, dest, force, namespace),
        Commands::Graft {
            target_name,
            entries,
            into,
            namespace,
        } => commands::graft::run(target_name, entries, into, namespace),
        Commands::Ungraft {
            composed_name,
            entry,
            as_name,
            namespace,
        } => commands::ungraft::run(composed_name, entry, as_name, namespace),
    }
}
