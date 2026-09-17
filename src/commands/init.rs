use anyhow::Result;
use std::path::PathBuf;

use crate::storage::Workspace;

pub fn run(at: Option<PathBuf>, store_id: Option<String>) -> Result<()> {
    let at = at.unwrap_or(std::env::current_dir()?);
    let ws = Workspace::init(&at, store_id)?;
    println!("Initialized odd workspace at {}", ws.odd_dir.display());
    println!("  store: {}", ws.store.id);
    Ok(())
}
