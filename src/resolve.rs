use anyhow::{bail, Result};

use crate::model::ObjectId;
use crate::storage::{Binding, Db};

/// The one place a target string turns into an object id. Exact-length
/// rule only (see ObjectId::looks_like_id) — never a threshold guess.
/// Returns the binding too when resolution went through a name, since
/// some callers (resurrect) need the binding's own recorded facts, not
/// just the id it points at.
pub fn resolve(db: &Db, namespace_id: &str, target: &str) -> Result<(ObjectId, Option<Binding>)> {
    if ObjectId::looks_like_id(target) {
        Ok((ObjectId(target.to_string()), None))
    } else if let Some(binding) = db.lookup_binding(namespace_id, target)? {
        let id = binding.object_id.clone();
        Ok((id, Some(binding)))
    } else {
        bail!("not remembered in namespace {namespace_id}: {target}")
    }
}
