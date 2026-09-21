use anyhow::{anyhow, bail, Result};

use crate::model::{ContentRef, Object, ObjectKind};
use crate::resolve::resolve;
use crate::storage::{Binding, Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

/// Removes `entry` from a composed object. Objects are immutable, so this
/// never edits the original in place — it produces a *new* composed
/// object without that entry. By default the result is rebound to
/// `composed_name`, which is what makes this read like an edit even
/// though underneath it's the same "always produce something new" rule
/// as `graft`. `--as` names the result separately instead, leaving the
/// original binding untouched.
pub fn run(
    composed_name: String,
    entry: String,
    as_name: Option<String>,
    namespace: Option<String>,
) -> Result<()> {
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    let (base_id, _) = resolve(&db, &namespace_id, &composed_name)?;
    let base_obj = db
        .get_object(&base_id)?
        .ok_or_else(|| anyhow!("object referenced but missing: {}", base_id.as_str()))?;
    if base_obj.kind != ObjectKind::Composed {
        bail!(
            "ungraft target must be a composed object (it's a {:?}): {composed_name}",
            base_obj.kind
        );
    }

    let mut entries = match base_obj.content {
        ContentRef::Directory { entries } => entries,
        ContentRef::Blob { .. } => {
            unreachable!("Composed objects always hold Directory-shaped content")
        }
    };
    if entries.remove(&entry).is_none() {
        bail!("no such entry '{entry}' in {composed_name}");
    }

    let obj = Object::new_composed(entries);
    db.put_object(&obj)?;

    let target_name = as_name.unwrap_or_else(|| composed_name.clone());
    db.bind(&Binding {
        namespace_id,
        name: target_name.clone(),
        object_id: obj.id.clone(),
        original_path: None,
        mode: None,
        mtime: None,
    })?;

    println!(
        "Ungrafted {entry} from {composed_name} -> {target_name} ({})",
        obj.id.as_str()
    );
    Ok(())
}
