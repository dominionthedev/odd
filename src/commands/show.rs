use anyhow::{bail, Result};

use crate::model::{ContentRef, ObjectId};
use crate::storage::{Db, Workspace};

const DEFAULT_NAMESPACE: &str = "manual:default";

pub fn run(target: String, namespace: Option<String>) -> Result<()> {
    let ws = Workspace::find(None)?;
    let db = Db::open(&ws.store.db_path())?;
    let namespace_id = namespace.unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    // Exact-length rule, not a threshold guess — see OPERATIONS.md §1.
    let object_id = if ObjectId::looks_like_id(&target) {
        ObjectId(target.clone())
    } else if let Some(binding) = db.lookup_binding(&namespace_id, &target)? {
        binding.object_id
    } else {
        bail!("not remembered in namespace {namespace_id}: {target}");
    };

    let obj = db
        .get_object(&object_id)?
        .ok_or_else(|| anyhow::anyhow!("object referenced but missing: {}", object_id.as_str()))?;

    println!("id:      {}", obj.id.as_str());
    println!("kind:    {:?}", obj.kind);
    match &obj.content {
        ContentRef::Blob { hash } => println!("blob:    {hash}"),
        ContentRef::Directory { entries } => {
            println!("entries:");
            for (name, id) in entries {
                println!("  {name} -> {}", id.as_str());
            }
        }
    }
    println!("created: {}", obj.created.to_rfc3339());
    Ok(())
}
