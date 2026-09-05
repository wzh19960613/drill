use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::store::json_store::JsonStore;

pub struct PausedStore {
    store: JsonStore<Option<Value>>,
}

impl PausedStore {
    pub fn load(path: PathBuf) -> PausedStore {
        PausedStore {
            store: JsonStore::load(path),
        }
    }

    pub fn get(&self) -> Option<Value> {
        self.store.view(|v| v.clone())
    }

    pub fn set(&self, v: Value) {
        self.store.mutate(|inner| *inner = Some(v));
    }

    /// Clear the session snapshot: drop it in memory and delete the file so a
    /// restart does not resurrect a stale pause.
    pub fn clear(&self) {
        self.store.edit(|inner| *inner = None);
        let _ = fs::remove_file(self.store.path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_get_and_clear_roundtrip() {
        let dir = std::env::temp_dir().join(format!("drill-paused-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("paused.json");
        let store = PausedStore::load(path.clone());
        assert_eq!(store.get(), None);

        let snapshot = json!({ "qid": "P1", "index": 3 });
        store.set(snapshot.clone());
        assert_eq!(store.get(), Some(snapshot.clone()));
        assert_eq!(PausedStore::load(path.clone()).get(), Some(snapshot));

        store.clear();
        assert_eq!(store.get(), None);
        assert!(!path.exists(), "cleared store deletes the file");
        assert_eq!(PausedStore::load(path.clone()).get(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn null_file_loads_as_none() {
        let dir = std::env::temp_dir().join(format!("drill-paused-null-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("paused.json");
        std::fs::write(&path, "null").unwrap();
        assert_eq!(PausedStore::load(path.clone()).get(), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
