use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::model::{ContentRef, Object, ObjectId, ObjectKind};

pub struct Db {
    conn: Connection,
}

/// A binding: name -> object, plus the path-specific facts that belong to
/// *this* binding, not to the shared object. Two names bound to the same
/// deduped object each keep their own row here.
#[derive(Debug, Clone)]
pub struct Binding {
    pub namespace_id: String,
    pub name: String,
    pub object_id: ObjectId,
    pub original_path: Option<String>,
    pub mode: Option<u32>,
    pub mtime: Option<i64>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("open sqlite db")?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS objects (
                id       TEXT PRIMARY KEY,
                kind     TEXT NOT NULL,
                content  TEXT NOT NULL,
                parents  TEXT NOT NULL,
                created  TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS bindings (
                namespace_id  TEXT NOT NULL,
                name          TEXT NOT NULL,
                object_id     TEXT NOT NULL,
                original_path TEXT,
                mode          INTEGER,
                mtime         INTEGER,
                PRIMARY KEY (namespace_id, name)
            );
            ",
        )
        .context("create schema")?;
        Ok(Self { conn })
    }

    pub fn put_object(&self, obj: &Object) -> Result<()> {
        let content = serde_json::to_string(&obj.content)?;
        let parents = serde_json::to_string(&obj.parents)?;
        let kind = match obj.kind {
            ObjectKind::File => "file",
            ObjectKind::Directory => "directory",
            ObjectKind::Composed => "composed",
        };
        self.conn.execute(
            "INSERT OR REPLACE INTO objects (id, kind, content, parents, created)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                obj.id.as_str(),
                kind,
                content,
                parents,
                obj.created.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_object(&self, id: &ObjectId) -> Result<Option<Object>> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, content, parents, created FROM objects WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.as_str()])?;
        if let Some(row) = rows.next()? {
            let kind_s: String = row.get(0)?;
            let content_s: String = row.get(1)?;
            let parents_s: String = row.get(2)?;
            let created_s: String = row.get(3)?;
            let kind = match kind_s.as_str() {
                "file" => ObjectKind::File,
                "directory" => ObjectKind::Directory,
                "composed" => ObjectKind::Composed,
                other => anyhow::bail!("unknown object kind in db: {other}"),
            };
            let content: ContentRef = serde_json::from_str(&content_s)?;
            let parents: Vec<ObjectId> = serde_json::from_str(&parents_s)?;
            let created = chrono::DateTime::parse_from_rfc3339(&created_s)?.into();
            Ok(Some(Object {
                id: id.clone(),
                kind,
                content,
                parents,
                created,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn bind(&self, binding: &Binding) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO bindings
                (namespace_id, name, object_id, original_path, mode, mtime)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                binding.namespace_id,
                binding.name,
                binding.object_id.as_str(),
                binding.original_path,
                binding.mode,
                binding.mtime,
            ],
        )?;
        Ok(())
    }

    pub fn lookup_binding(&self, namespace_id: &str, name: &str) -> Result<Option<Binding>> {
        let mut stmt = self.conn.prepare(
            "SELECT object_id, original_path, mode, mtime
             FROM bindings WHERE namespace_id = ?1 AND name = ?2",
        )?;
        let mut rows = stmt.query(params![namespace_id, name])?;
        if let Some(row) = rows.next()? {
            let object_id: String = row.get(0)?;
            Ok(Some(Binding {
                namespace_id: namespace_id.to_string(),
                name: name.to_string(),
                object_id: ObjectId(object_id),
                original_path: row.get(1)?,
                mode: row.get(2)?,
                mtime: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    // Not called yet — wired in when `odd ls` is built, the next
    // introspection command after this slice.
    #[allow(dead_code)]
    pub fn list_names(&self, namespace_id: &str, prefix: &str) -> Result<Vec<(String, ObjectId)>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, object_id FROM bindings
             WHERE namespace_id = ?1 AND name LIKE ?2 || '%'
             ORDER BY name",
        )?;
        let rows = stmt.query_map(params![namespace_id, prefix], |row| {
            let name: String = row.get(0)?;
            let object_id: String = row.get(1)?;
            Ok((name, ObjectId(object_id)))
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }
}
