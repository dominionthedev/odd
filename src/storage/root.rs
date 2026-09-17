use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

fn odd_home() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("ODD_HOME") {
        return Ok(PathBuf::from(p));
    }
    let base = dirs_next_data_dir()?;
    Ok(base.join("odd"))
}

// Minimal stand-in for a dirs crate — kept in-house since this is the only
// thing we need it for right now. Revisit if a second use case shows up.
fn dirs_next_data_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    Ok(PathBuf::from(home).join(".local").join("share"))
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub struct Store {
    pub id: String,
    pub path: PathBuf,
}

impl Store {
    fn path_for_id(id: &str) -> Result<PathBuf> {
        Ok(odd_home()?.join("stores").join(id))
    }

    pub fn open(id: &str) -> Result<Self> {
        let path = Self::path_for_id(id)?;
        if !path.is_dir() {
            bail!("store not found: {id}");
        }
        Ok(Self {
            id: id.to_string(),
            path,
        })
    }

    /// Default id, when the caller didn't ask for a specific name: a slug
    /// of `hint` (typically the workspace directory name) rather than a
    /// raw UUID — a readable default costs nothing.
    pub fn create(id: Option<String>, hint: &str) -> Result<Self> {
        let id = id.unwrap_or_else(|| {
            let slug = slugify(hint);
            if slug.is_empty() {
                uuid::Uuid::now_v7().to_string()
            } else {
                slug
            }
        });
        let path = Self::path_for_id(&id)?;
        if path.is_dir() {
            return Self::open(&id);
        }
        fs::create_dir_all(path.join("objects")).context("create store objects/")?;
        Ok(Self { id, path })
    }

    // Called by remember/show/resurrect once they exist — not this commit.
    #[allow(dead_code)]
    pub fn db_path(&self) -> PathBuf {
        self.path.join("metadata.db")
    }

    #[allow(dead_code)]
    pub fn objects_dir(&self) -> PathBuf {
        self.path.join("objects")
    }
}

pub struct Workspace {
    pub odd_dir: PathBuf,
    pub store: Store,
}

impl Workspace {
    pub fn init(at: &Path, store_id: Option<String>) -> Result<Self> {
        let odd_dir = at.join(".odd");
        if odd_dir.exists() {
            bail!(".odd already exists at {}", at.display());
        }
        let hint = at
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());
        let store = Store::create(store_id, &hint)?;
        fs::create_dir_all(&odd_dir).context("create .odd")?;
        fs::write(odd_dir.join("store_id"), &store.id).context("write store_id locator")?;
        Ok(Self { odd_dir, store })
    }

    // Used once remember/show/resurrect need to locate an existing
    // workspace — init doesn't, since it's the one command that creates
    // a workspace rather than finding one.
    #[allow(dead_code)]
    pub fn find(start: Option<PathBuf>) -> Result<Self> {
        let mut dir = start.unwrap_or(std::env::current_dir()?);
        loop {
            let candidate = dir.join(".odd");
            if candidate.is_dir() {
                let store_id = fs::read_to_string(candidate.join("store_id"))
                    .context("read store_id locator")?
                    .trim()
                    .to_string();
                let store = Store::open(&store_id)?;
                return Ok(Self {
                    odd_dir: candidate,
                    store,
                });
            }
            if !dir.pop() {
                bail!("not inside an odd workspace (no .odd found)");
            }
        }
    }
}
