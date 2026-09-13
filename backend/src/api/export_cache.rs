use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::fsutil::{atomic_write, lock, log_warn};

pub struct ExportCache {
    dir: PathBuf,
    entries: Mutex<BTreeMap<String, CacheEntry>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry {
    file: String,
    #[serde(default)]
    book: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct SaveFile {
    #[serde(default)]
    entries: BTreeMap<String, CacheEntry>,
}

impl ExportCache {
    pub fn new(dir: PathBuf) -> ExportCache {
        let entries = match fs::read_to_string(dir.join("cache.json")) {
            Ok(text) => {
                serde_json::from_str::<SaveFile>(&text)
                    .unwrap_or_else(|e| {
                        eprintln!(
                            "[warn] corrupted export cache {}, resetting: {e}",
                            dir.display()
                        );
                        Default::default()
                    })
                    .entries
            }
            Err(_) => BTreeMap::new(),
        };
        ExportCache {
            dir,
            entries: Mutex::new(entries),
        }
    }

    pub fn lookup(&self, hash: &str) -> Option<PathBuf> {
        let file = lock(&self.entries).get(hash)?.file.clone();
        let path = self.dir.join(&file);
        path.is_file().then_some(path)
    }

    pub fn insert(&self, hash: &str, file: &str, book: Option<&str>) {
        let mut entries = lock(&self.entries);
        entries.insert(
            hash.to_string(),
            CacheEntry {
                file: file.to_string(),
                book: book.map(str::to_string),
            },
        );
        self.save(&entries);
    }

    pub fn remove_book(&self, book_id: &str) {
        let mut entries = lock(&self.entries);
        let doomed: Vec<String> = entries
            .values()
            .filter(|e| e.book.as_deref() == Some(book_id))
            .map(|e| e.file.clone())
            .collect();
        if doomed.is_empty() {
            return;
        }
        for file in &doomed {
            let _ = fs::remove_file(self.dir.join(file));
        }
        entries.retain(|_, e| e.book.as_deref() != Some(book_id));
        self.save(&entries);
    }

    fn save(&self, entries: &BTreeMap<String, CacheEntry>) {
        let mut pruned = entries.clone();
        pruned.retain(|_, e| self.dir.join(&e.file).is_file());
        let file = self.dir.join("cache.json");
        let bytes = serde_json::to_vec_pretty(&SaveFile { entries: pruned }).unwrap_or_default();
        if let Err(e) = atomic_write(&file, &bytes) {
            log_warn(&format!(
                "[warn] cannot write export cache {}: {e}",
                file.display()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("drill-cache-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn insert_lookup_and_book_cleanup() {
        let dir = tmpdir("ops");
        fs::write(dir.join("a.pdf"), b"pdf").unwrap();
        fs::write(dir.join("b.pdf"), b"pdf").unwrap();

        let cache = ExportCache::new(dir.clone());
        cache.insert("h1", "a.pdf", Some("book1"));
        cache.insert("h2", "b.pdf", None);
        assert!(cache.lookup("h1").is_some());
        assert!(cache.lookup("missing").is_none());

        cache.remove_book("book1");
        assert!(cache.lookup("h1").is_none());
        assert!(!dir.join("a.pdf").exists(), "book1 pdf removed");
        assert!(dir.join("b.pdf").exists(), "bookless export kept");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dead_entries_are_pruned_on_save() {
        let dir = tmpdir("prune");
        let cache = ExportCache::new(dir.clone());
        cache.insert("h1", "gone.pdf", Some("b"));
        assert!(cache.lookup("h1").is_none(), "file never existed");
        let raw = fs::read_to_string(dir.join("cache.json")).unwrap();
        assert!(!raw.contains("gone.pdf"), "dead entry not persisted: {raw}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn survives_restart_and_corrupted_files() {
        let dir = tmpdir("restart");
        fs::write(dir.join("a.pdf"), b"pdf").unwrap();
        let cache = ExportCache::new(dir.clone());
        cache.insert("h1", "a.pdf", Some("b1"));

        let reloaded = ExportCache::new(dir.clone());
        assert!(reloaded.lookup("h1").is_some());

        fs::write(dir.join("cache.json"), "{broken").unwrap();
        let corrupted = ExportCache::new(dir.clone());
        assert!(corrupted.lookup("h1").is_none());
        fs::write(dir.join("c.pdf"), b"pdf").unwrap();
        corrupted.insert("h3", "c.pdf", None);
        assert!(ExportCache::new(dir.clone()).lookup("h3").is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn concurrent_inserts_are_not_lost() {
        let dir = tmpdir("concurrent");
        for i in 0..8 {
            fs::write(dir.join(format!("f{i}.pdf")), b"pdf").unwrap();
        }
        let cache = std::sync::Arc::new(ExportCache::new(dir.clone()));
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let cache = cache.clone();
                std::thread::spawn(move || {
                    cache.insert(&format!("h{i}"), &format!("f{i}.pdf"), None);
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        for i in 0..8 {
            assert!(cache.lookup(&format!("h{i}")).is_some(), "entry h{i} lost");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
