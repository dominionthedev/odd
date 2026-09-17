use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path_for_hash(&self, hash: &str) -> PathBuf {
        if hash.len() < 2 {
            return self.root.join(hash);
        }
        self.root.join(&hash[0..2]).join(hash)
    }

    /// Hash, check existence, write-to-tmp-then-rename. Returns the hash.
    pub fn put_bytes(&self, data: &[u8]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());

        let path = self.path_for_hash(&hash);
        if path.exists() {
            return Ok(hash);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("create blob shard dir")?;
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, data).context("write tmp blob")?;
        fs::rename(&tmp, &path).context("rename tmp blob into place")?;
        Ok(hash)
    }

    // Used by resurrect, landing in the next commit.
    #[allow(dead_code)]
    pub fn get_bytes(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.path_for_hash(hash);
        fs::read(path).with_context(|| format!("read blob {hash}"))
    }

    // Not called by any command in this slice yet — planned for `doctor`
    // and for `graft`'s dedup checks once those exist.
    #[allow(dead_code)]
    pub fn contains(&self, hash: &str) -> bool {
        self.path_for_hash(hash).exists()
    }
}

pub fn hash_file(path: &Path) -> Result<(String, Vec<u8>)> {
    let data = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok((hex::encode(hasher.finalize()), data))
}
