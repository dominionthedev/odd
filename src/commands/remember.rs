use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{Object, ObjectId};
use crate::storage::{Binding, BlobStore, Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

pub fn run(
    path: PathBuf,
    as_name: Option<String>,
    namespace: Option<String>,
    recursive: bool,
) -> Result<()> {
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let blobs = BlobStore::new(ws.store.objects_dir());
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    let name = as_name.unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string())
    });

    if path.is_dir() {
        if !recursive {
            bail!(
                "{} is a directory; pass -r/--recursive to remember it",
                path.display()
            );
        }
        let obj = remember_dir(&db, &blobs, &namespace_id, &name, &path)?;
        println!(
            "Remembered {} as {} ({})",
            path.display(),
            name,
            obj.id.as_str()
        );
    } else if path.is_file() {
        let obj = remember_file(&db, &blobs, &namespace_id, &name, &path)?;
        println!(
            "Remembered {} as {} ({})",
            path.display(),
            name,
            obj.id.as_str()
        );
    } else {
        bail!("not a file or directory: {}", path.display());
    }

    Ok(())
}

fn stat_binding_fields(path: &Path) -> Result<(Option<u32>, Option<i64>)> {
    let meta = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
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

    Ok((mode, mtime))
}

/// Capture one file as a blob, bind `name` to it. Path-specific facts go
/// on the binding, never on the object — see model::Object's own docs.
fn remember_file(
    db: &Db,
    blobs: &BlobStore,
    namespace_id: &str,
    name: &str,
    path: &Path,
) -> Result<Object> {
    let (hash, data) = crate::storage::hash_file(path)?;
    blobs.put_bytes(&data)?;
    let obj = Object::new_file(hash);
    db.put_object(&obj)?;

    let (mode, mtime) = stat_binding_fields(path)?;
    db.bind(&Binding {
        namespace_id: namespace_id.to_string(),
        name: name.to_string(),
        object_id: obj.id.clone(),
        original_path: Some(path.display().to_string()),
        mode,
        mtime,
    })?;

    Ok(obj)
}

/// Capture a directory recursively. Each child (file or subdirectory) is
/// bound in its own right under `<name>/<child>`, in addition to the
/// directory itself being bound under `name` — so any nested file can be
/// `show`n or `resurrect`ed on its own later, not only as part of the
/// whole tree.
///
/// Directory entries are built from what's actually on disk right now,
/// not from prior bindings — a directory object's structural id has to
/// reflect its real current children.
fn remember_dir(
    db: &Db,
    blobs: &BlobStore,
    namespace_id: &str,
    name: &str,
    path: &Path,
) -> Result<Object> {
    let mut dir_entries: Vec<_> = fs::read_dir(path)
        .with_context(|| format!("read dir {}", path.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("read dir entries {}", path.display()))?;
    dir_entries.sort_by_key(|e| e.file_name());

    let mut entries: BTreeMap<String, ObjectId> = BTreeMap::new();
    for entry in dir_entries {
        let child_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let child_name = format!("{name}/{file_name}");

        let child_obj = if child_path.is_dir() {
            remember_dir(db, blobs, namespace_id, &child_name, &child_path)?
        } else if child_path.is_file() {
            remember_file(db, blobs, namespace_id, &child_name, &child_path)?
        } else {
            // Symlinks and other special files are out of scope for this
            // slice — skip rather than guess at semantics for them.
            continue;
        };
        entries.insert(file_name, child_obj.id);
    }

    let obj = Object::new_directory(entries);
    db.put_object(&obj)?;

    let (mode, mtime) = stat_binding_fields(path)?;
    db.bind(&Binding {
        namespace_id: namespace_id.to_string(),
        name: name.to_string(),
        object_id: obj.id.clone(),
        original_path: Some(path.display().to_string()),
        mode,
        mtime,
    })?;

    Ok(obj)
}
