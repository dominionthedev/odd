use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::model::Object;
use crate::storage::{Binding, BlobStore, Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

/// v1 scope: a single file. Directories, recursion, and `--ref` are the
/// next slice — see OPERATIONS.md build order. Not adding the flags for
/// them until they do something.
pub fn run(path: PathBuf, as_name: Option<String>, namespace: Option<String>) -> Result<()> {
    if !path.is_file() {
        bail!(
            "remember (v1) only supports a single file: {}",
            path.display()
        );
    }
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let blobs = BlobStore::new(ws.store.objects_dir());

    let (hash, data) = crate::storage::hash_file(&path)?;
    blobs.put_bytes(&data)?;
    let obj = Object::new_file(hash);
    db.put_object(&obj)?;

    let name = as_name.unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string())
    });
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    let meta = std::fs::metadata(&path).context("stat remembered file")?;
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        Some(meta.permissions().mode())
    };
    #[cfg(not(unix))]
    let mode: Option<u32> = None;

    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);

    db.bind(&Binding {
        namespace_id,
        name: name.clone(),
        object_id: obj.id.clone(),
        original_path: Some(path.display().to_string()),
        mode,
        mtime,
    })?;

    println!(
        "Remembered {} as {} ({})",
        path.display(),
        name,
        obj.id.as_str()
    );
    Ok(())
}
