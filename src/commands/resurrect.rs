use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::model::{ContentRef, Object};
use crate::resolve::resolve;
use crate::storage::{BlobStore, Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

pub fn run(
    target: String,
    dest: Option<PathBuf>,
    force: bool,
    namespace: Option<String>,
) -> Result<()> {
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let blobs = BlobStore::new(ws.store.objects_dir());
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    // Default destination comes from the *binding*, per OPERATIONS.md §1 —
    // never from the object, since a deduped object may be reachable from
    // several bindings with different original paths.
    let (object_id, binding) = resolve(&db, &namespace_id, &target)?;
    let default_dest = binding.and_then(|b| b.original_path).map(PathBuf::from);

    let dest = dest
        .or(default_dest)
        .ok_or_else(|| anyhow::anyhow!("no destination known for {target}; pass --dest"))?;

    let obj = db
        .get_object(&object_id)?
        .ok_or_else(|| anyhow::anyhow!("object referenced but missing: {}", object_id.as_str()))?;

    materialize(&db, &blobs, &obj, &dest, force)?;

    println!("Resurrected {} to {}", object_id.as_str(), dest.display());
    Ok(())
}

/// Blob content is gated by `force` like any single-file write. Directory
/// content is always allowed to descend into an existing directory —
/// composing into a place that already exists is fine; overwriting an
/// unrelated file there is what `force` actually guards against, and that
/// check happens per-file at the leaves, not once at the top.
fn materialize(db: &Db, blobs: &BlobStore, obj: &Object, dest: &Path, force: bool) -> Result<()> {
    match &obj.content {
        ContentRef::Blob { hash } => {
            if dest.exists() && !force {
                bail!("destination exists: {} (use --force)", dest.display());
            }
            let data = blobs.get_bytes(hash)?;
            if let Some(parent) = dest.parent() {
                fs_create_dir_all(parent)?;
            }
            std::fs::write(dest, data).context("write resurrected file")?;
        }
        ContentRef::Directory { entries } => {
            if dest.exists() && !dest.is_dir() {
                bail!(
                    "destination exists and is not a directory: {}",
                    dest.display()
                );
            }
            fs_create_dir_all(dest)?;
            for (name, child_id) in entries {
                let child_obj = db.get_object(child_id)?.ok_or_else(|| {
                    anyhow::anyhow!("object referenced but missing: {}", child_id.as_str())
                })?;
                materialize(db, blobs, &child_obj, &dest.join(name), force)?;
            }
        }
    }
    Ok(())
}

fn fs_create_dir_all(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .with_context(|| format!("create destination directory {}", path.display()))
}
