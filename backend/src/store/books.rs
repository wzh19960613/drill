use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::model::BookDef;
use crate::store::json_store::JsonStore;

/// Current book per subject ('' = all subjects).
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct ActiveMap(BTreeMap<String, String>);

impl ActiveMap {
    pub fn get(&self, subject: &str) -> Option<String> {
        self.0.get(subject).cloned()
    }

    pub fn set(&mut self, subject: &str, id: Option<String>) {
        match id {
            Some(id) => {
                self.0.insert(subject.to_string(), id);
            }
            None => {
                self.0.remove(subject);
            }
        }
    }

    fn retain_books(&mut self, kept: &[BookDef]) {
        let ids: std::collections::BTreeSet<&str> = kept.iter().map(|b| b.id.as_str()).collect();
        self.0.retain(|_, id| ids.contains(id.as_str()));
    }
}

#[derive(Serialize, Deserialize, Default)]
struct SaveFile {
    #[serde(default)]
    books: Vec<BookDef>,
    #[serde(default, rename = "activeBySubject")]
    active: ActiveMap,
    #[serde(
        default,
        rename = "favoriteId",
        skip_serializing_if = "Option::is_none"
    )]
    favorite: Option<String>,
}

pub struct BookStore {
    store: JsonStore<SaveFile>,
    max: usize,
}

const MAX_BOOKS: usize = 65535;

impl BookStore {
    /// Upper bound on stored books, exposed for tests and API messages.
    #[cfg(test)]
    pub fn max_books() -> usize {
        MAX_BOOKS
    }

    pub fn load(path: PathBuf) -> BookStore {
        BookStore::load_with_cap(path, MAX_BOOKS)
    }

    /// Same store with an explicit cap; tests use a tiny one because every
    /// upsert rewrites the whole books file — filling the production cap
    /// would be a quadratic, hundred-GB write storm.
    pub fn load_with_cap(path: PathBuf, max: usize) -> BookStore {
        BookStore {
            store: JsonStore::load(path),
            max,
        }
    }

    pub fn list(&self) -> Vec<BookDef> {
        self.store.view(|f| f.books.clone())
    }

    /// Insert (or replace) one book at the top of the list. Adding a new
    /// book once the list is full is rejected instead of silently dropping
    /// the oldest entry.
    pub fn upsert(&self, def: BookDef) -> Result<(), String> {
        self.store.mutate(|f| {
            if !f.books.iter().any(|b| b.id == def.id) && f.books.len() >= self.max {
                return Err(format!("题本数量已达上限（{}）", self.max));
            }
            f.books.retain(|b| b.id != def.id);
            f.books.insert(0, def);
            f.active.retain_books(&f.books);
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> bool {
        self.store.mutate(|f| {
            let n = f.books.len();
            f.books.retain(|b| b.id != id);
            if f.books.len() == n {
                return false;
            }
            if f.favorite.as_deref() == Some(id) {
                f.favorite = None;
            }
            f.active.retain_books(&f.books);
            true
        })
    }

    pub fn get_active(&self, subject: &str) -> Option<String> {
        self.store.view(|f| f.active.get(subject))
    }

    pub fn set_active(&self, subject: &str, id: Option<String>) {
        self.store.mutate(|f| f.active.set(subject, id));
    }

    pub fn favorite(&self) -> Option<String> {
        self.store.view(|f| f.favorite.clone())
    }

    pub fn set_favorite(&self, id: Option<String>) {
        self.store.mutate(|f| f.favorite = id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BookItemDef;

    fn def(id: &str, name: &str) -> BookDef {
        BookDef {
            id: id.into(),
            name: name.into(),
            seed: String::new(),
            date: "2026-08-26".into(),
            created_at: 1,
            items: vec![BookItemDef {
                id: "P1-1".into(),
                source: String::new(),
                option_order: None,
            }],
        }
    }

    fn store(tag: &str) -> (PathBuf, BookStore) {
        let dir = std::env::temp_dir().join(format!("drill-book-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("books.json");
        let _ = std::fs::remove_file(&path);
        let s = BookStore::load(path.clone());
        (path, s)
    }

    fn store_capped(tag: &str, cap: usize) -> (PathBuf, BookStore) {
        let dir = std::env::temp_dir().join(format!("drill-book-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("books.json");
        let _ = std::fs::remove_file(&path);
        (path.clone(), BookStore::load_with_cap(path, cap))
    }

    #[test]
    fn active_book_is_per_subject() {
        let (path, store) = store("subject");
        store.upsert(def("a", "one")).unwrap();
        store.upsert(def("b", "two")).unwrap();

        store.set_active("", Some("a".into()));
        store.set_active("数学", Some("b".into()));
        assert_eq!(store.get_active("").as_deref(), Some("a"));
        assert_eq!(store.get_active("数学").as_deref(), Some("b"));
        assert_eq!(store.get_active("物理"), None);

        assert!(store.delete("b"));
        assert_eq!(store.get_active("数学"), None);
        assert_eq!(store.get_active("").as_deref(), Some("a"));

        let reloaded = BookStore::load(path.clone());
        assert_eq!(reloaded.get_active("").as_deref(), Some("a"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn favorite_book_roundtrip_and_clear_on_delete() {
        let (path, store) = store("favorite");
        assert_eq!(store.favorite(), None);
        store.upsert(def("a", "one")).unwrap();
        store.set_favorite(Some("a".into()));
        assert_eq!(store.favorite().as_deref(), Some("a"));
        assert!(store.delete("a"));
        assert_eq!(store.favorite(), None);
        let reloaded = BookStore::load(path.clone());
        assert_eq!(reloaded.favorite(), None);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn upsert_rejects_a_new_book_at_the_cap_but_allows_replacing() {
        // capped at 3: filling the production cap would rewrite the file
        // quadratically (hundreds of GB of writes)
        let (_path, store) = store_capped("cap", 3);
        for i in 0..3 {
            store.upsert(def(&format!("b{i}"), "x")).unwrap();
        }
        assert_eq!(store.list().len(), 3);
        let err = store.upsert(def("new", "y")).unwrap_err();
        assert!(err.contains("上限"), "{err}");
        assert_eq!(store.list().len(), 3, "rejected book is not stored");
        assert_eq!(store.list()[0].id, "b2");
        // replacing an existing book at the cap is still allowed
        store.upsert(def("b0", "replaced")).unwrap();
        assert_eq!(store.list().len(), 3);
        assert!(store
            .list()
            .iter()
            .any(|b| b.id == "b0" && b.name == "replaced"));
    }

    #[test]
    fn corrupted_file_resets_and_keeps_format() {
        let (path, _store) = store("corrupt");
        std::fs::write(&path, "not json").unwrap();
        let fresh = BookStore::load(path.clone());
        assert!(fresh.list().is_empty());
        fresh.upsert(def("a", "one")).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(
            raw.contains("\"activeBySubject\""),
            "wire format preserved: {raw}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
