use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::model::{ContentRef, ObjectId};
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
    let (object_id, default_dest) = if ObjectId::looks_like_id(&target) {
        (ObjectId(target.clone()), None)
    } else if let Some(binding) = db.lookup_binding(&namespace_id, &target)? {
        let d = binding.original_path.clone().map(PathBuf::from);
        (binding.object_id, d)
    } else {
        bail!("not remembered in namespace {namespace_id}: {target}");
    };

    let dest = dest
        .or(default_dest)
        .ok_or_else(|| anyhow::anyhow!("no destination known for {target}; pass --dest"))?;

    if dest.exists() && !force {
        bail!("destination exists: {} (use --force)", dest.display());
    }

    let obj = db
        .get_object(&object_id)?
        .ok_or_else(|| anyhow::anyhow!("object referenced but missing: {}", object_id.as_str()))?;

    match &obj.content {
        ContentRef::Blob { hash } => {
            let data = blobs.get_bytes(hash)?;
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).context("create destination parent dir")?;
            }
            std::fs::write(&dest, data).context("write resurrected file")?;
        }
        ContentRef::Directory { .. } => {
            bail!("resurrecting directories is not part of this first slice yet");
        }
    }

    println!("Resurrected {} to {}", object_id.as_str(), dest.display());
    Ok(())
}
