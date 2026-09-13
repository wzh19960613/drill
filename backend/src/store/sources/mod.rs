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

    #[serde(default)]
    pub recursive: bool,

    #[serde(default)]
    pub excluded: Vec<String>,
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

    pub fn load(file: PathBuf, root: Option<&Path>) -> SourceStore {
        let store: JsonStore<Inner> = JsonStore::load(file);

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
                recursive: false,
                excluded: Vec::new(),
            };
            inner.next_id += 1;
            inner.sources.push(src.clone());
            Ok(src)
        })
    }

    pub fn set_recursive(&self, id: &str, recursive: bool) -> Result<Source, String> {
        self.store.mutate(|inner| {
            let src = inner
                .sources
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| "unknown source id".to_string())?;
            src.recursive = recursive;
            Ok(src.clone())
        })
    }

    pub fn mark_excluded(&self, id: &str, rel: &str, on: bool) -> Result<Source, String> {
        self.store.mutate(|inner| {
            let src = inner
                .sources
                .iter_mut()
                .find(|s| s.id == id)
                .ok_or_else(|| "unknown source id".to_string())?;
            if on {
                if !src.excluded.iter().any(|p| p == rel) {
                    let pos = src
                        .excluded
                        .iter()
                        .position(|p| p.as_str() > rel)
                        .unwrap_or(src.excluded.len());
                    src.excluded.insert(pos, rel.to_string());
                }
            } else {
                src.excluded.retain(|p| p != rel);
            }
            Ok(src.clone())
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
        recursive: false,
        excluded: Vec::new(),
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
