use anyhow::{anyhow, bail, Result};
use std::collections::BTreeMap;

use crate::model::{ContentRef, Object, ObjectId, ObjectKind};
use crate::resolve::resolve;
use crate::storage::{Binding, Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

/// `entries` are "name=source" pairs, source resolved the same way any
/// other target is (exact-length id, or a binding in this namespace).
/// `into` extends an existing composed object — this never mutates it;
/// it produces a *new* composed object (existing entries plus the new
/// ones) and rebinds `target_name` to that new object.
pub fn run(
    target_name: String,
    entries: Vec<String>,
    into: Option<String>,
    namespace: Option<String>,
) -> Result<()> {
    if entries.is_empty() {
        bail!("graft needs at least one entry=source pair");
    }
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    let mut result_entries: BTreeMap<String, ObjectId> = BTreeMap::new();

    if let Some(into_name) = &into {
        let (base_id, _) = resolve(&db, &namespace_id, into_name)?;
        let base_obj = db
            .get_object(&base_id)?
            .ok_or_else(|| anyhow!("object referenced but missing: {}", base_id.as_str()))?;
        if base_obj.kind != ObjectKind::Composed {
            bail!(
                "--into target must be a composed object (it's a {:?}): {into_name}",
                base_obj.kind
            );
        }
        match base_obj.content {
            ContentRef::Directory { entries } => result_entries = entries,
            ContentRef::Blob { .. } => {
                unreachable!("Composed objects always hold Directory-shaped content")
            }
        }
    }

    for raw in &entries {
        let (entry_name, source) = raw
            .split_once('=')
            .ok_or_else(|| anyhow!("expected entry=source, got: {raw}"))?;
        if entry_name.is_empty() {
            bail!("empty entry name in: {raw}");
        }
        let (source_id, _) = resolve(&db, &namespace_id, source)?;
        result_entries.insert(entry_name.to_string(), source_id);
    }

    let obj = Object::new_composed(result_entries);
    db.put_object(&obj)?;

    // A grafted object was never on disk directly — no original path,
    // mode, or mtime to record. The binding's metadata fields are all
    // `None`, which is exactly what they're for.
    db.bind(&Binding {
        namespace_id,
        name: target_name.clone(),
        object_id: obj.id.clone(),
        original_path: None,
        mode: None,
        mtime: None,
    })?;

    println!("Grafted {} ({})", target_name, obj.id.as_str());
    Ok(())
}
