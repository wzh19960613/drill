#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::store::json_store::JsonStore;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Inner {
    #[serde(default = "default_next_id")]
    next_id: u32,
    #[serde(default)]
    sources: Vec<Source>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            next_id: default_next_id(),
            sources: Vec::new(),
        }
    }
}

fn default_next_id() -> u32 {
    1
}

pub struct SourceStore {
    store: JsonStore<Inner>,
}

impl SourceStore {
    /// `root` is only a first-run convenience: when `Some` and the store
    /// comes up empty, the directory itself is registered as the default
    /// source. With `None` (running from source, `DRILL_ROOT` not set)
    /// nothing is seeded and sources are managed entirely in the app.
    pub fn load(file: PathBuf, root: Option<&Path>) -> SourceStore {
        let store: JsonStore<Inner> = JsonStore::load(file);
        // seed only when the store comes up empty, and skip the rewrite
        // entirely when nothing changed
        let seeded = store.edit(|inner| {
            if inner.sources.is_empty() {
                root.is_some_and(|r| seed_default(inner, r))
            } else {
                false
            }
        });
        if seeded {
            store.save();
        }
        SourceStore { store }
    }

    pub fn list(&self) -> Vec<Source> {
        self.store.view(|inner| inner.sources.clone())
    }

    pub fn get(&self, id: &str) -> Option<Source> {
        self.store
            .view(|inner| inner.sources.iter().find(|s| s.id == id).cloned())
    }

    pub fn add(&self, input: &str, root: &Path) -> Result<Source, String> {
        let path = resolve_dir(input, root)?;
        self.store.mutate(|inner| {
            if inner.sources.iter().any(|s| s.path == path) {
                return Err(format!("source already exists: {}", path.display()));
            }
            let src = Source {
                id: format!("s{}", inner.next_id),
                name: source_name(&path),
                path,
            };
            inner.next_id += 1;
            inner.sources.push(src.clone());
            Ok(src)
        })
    }

    pub fn relocate(&self, id: &str, input: &str, root: &Path) -> Result<Source, String> {
        let path = resolve_dir(input, root)?;
        self.store.mutate(|inner| {
            if inner.sources.iter().any(|s| s.id != id && s.path == path) {
                return Err(format!("another source already uses {}", path.display()));
            }
            let src = inner
                .sources
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| "unknown source id".to_string())?;
            src.path = path;
            Ok(src.clone())
        })
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        self.store.mutate(|inner| {
            if !inner.sources.iter().any(|s| s.id == id) {
                return Err("unknown source id".to_string());
            }
            if inner.sources.len() <= 1 {
                return Err("at least one source is required".to_string());
            }
            inner.sources.retain(|s| s.id != id);
            Ok(())
        })
    }
}

fn seed_default(inner: &mut Inner, root: &Path) -> bool {
    let Ok(path) = root.canonicalize() else {
        return false;
    };
    inner.sources.push(Source {
        id: format!("s{}", inner.next_id),
        name: source_name(&path),
        path,
    });
    inner.next_id += 1;
    true
}

fn resolve_dir(input: &str, root: &Path) -> Result<PathBuf, String> {
    let p = Path::new(input.trim().trim_end_matches('/'));
    let p = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    p.canonicalize()
        .ok()
        .filter(|c| c.is_dir())
        .ok_or_else(|| format!("directory not found: {input}"))
}

fn source_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "source".to_string())
}
