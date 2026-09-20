use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// A content-addressed identifier. Never constructed from a filesystem
/// path or a timestamp — only from content or structure.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub String);

impl ObjectId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The one place a reference is treated as a raw id rather than a name:
    /// exact hash length, never a threshold guess.
    pub fn looks_like_id(s: &str) -> bool {
        s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// v1 kinds only — File and Directory. State, Composed, Historical,
/// Projection and Namespace are named in the design doc but deliberately
/// not implemented yet; adding them here ahead of a command that
/// constructs them is exactly the mistake we're not repeating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentRef {
    Blob { hash: String },
    Directory { entries: BTreeMap<String, ObjectId> },
}

/// Content only. No original path, no mtime, no mode — those are
/// binding-level facts (see storage::db), never object-level ones.
/// An object that dedupes across two different captured paths must not
/// have to choose whose metadata wins, because it never holds any.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub content: ContentRef,
    pub parents: Vec<ObjectId>,
    pub created: DateTime<Utc>,
}

impl Object {
    pub fn new_file(hash: String) -> Self {
        let content = ContentRef::Blob { hash: hash.clone() };
        Self {
            id: ObjectId(hash),
            kind: ObjectKind::File,
            content,
            parents: Vec::new(),
            created: Utc::now(),
        }
    }

    pub fn new_directory(entries: BTreeMap<String, ObjectId>) -> Self {
        let id = structural_id(ObjectKind::Directory, &entries);
        Self {
            id,
            kind: ObjectKind::Directory,
            content: ContentRef::Directory { entries },
            parents: Vec::new(),
            created: Utc::now(),
        }
    }
}

/// Structural identity: derived from kind + sorted child map only.
/// Never from path, mtime, or any other filesystem fact.
fn structural_id(kind: ObjectKind, entries: &BTreeMap<String, ObjectId>) -> ObjectId {
    let prefix = match kind {
        ObjectKind::Directory => "directory\n",
        ObjectKind::File => unreachable!("files are addressed by blob hash, not structural_id"),
    };
    let mut buf = String::from(prefix);
    for (name, id) in entries {
        buf.push_str(name);
        buf.push('\0');
        buf.push_str(id.as_str());
        buf.push('\n');
    }
    ObjectId(sha256_hex(buf.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_file_content_dedupes() {
        let a = Object::new_file(sha256_hex(b"hello"));
        let b = Object::new_file(sha256_hex(b"hello"));
        assert_eq!(
            a.id, b.id,
            "identical blob content must produce the same id"
        );
    }

    #[test]
    fn different_file_content_differs() {
        let a = Object::new_file(sha256_hex(b"hello"));
        let b = Object::new_file(sha256_hex(b"world"));
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn directory_id_ignores_insertion_order() {
        let a_id = ObjectId(sha256_hex(b"a"));
        let b_id = ObjectId(sha256_hex(b"b"));

        let mut entries1 = BTreeMap::new();
        entries1.insert("a.txt".to_string(), a_id.clone());
        entries1.insert("b.txt".to_string(), b_id.clone());

        let mut entries2 = BTreeMap::new();
        entries2.insert("b.txt".to_string(), b_id);
        entries2.insert("a.txt".to_string(), a_id);

        let dir1 = Object::new_directory(entries1);
        let dir2 = Object::new_directory(entries2);
        assert_eq!(
            dir1.id, dir2.id,
            "BTreeMap already sorts by key, so insertion order must not matter"
        );
    }

    #[test]
    fn empty_directory_is_a_stable_id() {
        let d1 = Object::new_directory(BTreeMap::new());
        let d2 = Object::new_directory(BTreeMap::new());
        assert_eq!(d1.id, d2.id);
    }

    #[test]
    fn directory_and_file_never_collide_by_construction() {
        // A file's id is its blob hash directly. A directory's id is
        // sha256("directory\n" + ...), which can never equal a bare
        // blob hash unless a preimage happens to start with that exact
        // prefix and hash to itself — not a real collision path, but
        // worth a test documenting the assumption if this ever changes.
        let f = Object::new_file(sha256_hex(b"same-bytes"));
        let mut entries = BTreeMap::new();
        entries.insert("x".to_string(), f.id.clone());
        let d = Object::new_directory(entries);
        assert_ne!(f.id, d.id);
    }
}
