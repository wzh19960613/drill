use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::store::json_store::JsonStore;

pub struct MasteryStore {
    store: JsonStore<BTreeSet<String>>,
}

impl MasteryStore {
    pub fn load(path: PathBuf) -> MasteryStore {
        MasteryStore {
            store: JsonStore::load(path),
        }
    }

    pub fn set(&self, ids: Vec<String>) {
        self.store
            .mutate(|inner| *inner = ids.into_iter().collect());
    }

    pub fn list(&self) -> Vec<String> {
        self.store.view(|s| s.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_roundtrips_deduped_and_sorted() {
        let dir = std::env::temp_dir().join(format!("drill-mastery-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mastery.json");
        let store = MasteryStore::load(path.clone());
        assert!(store.list().is_empty());

        store.set(vec!["P2".into(), "P1".into(), "P2".into()]);
        assert_eq!(store.list(), vec!["P1", "P2"]);

        let reloaded = MasteryStore::load(path.clone());
        assert_eq!(reloaded.list(), vec!["P1", "P2"]);

        store.set(vec![]);
        assert!(store.list().is_empty());
        assert_eq!(
            MasteryStore::load(path.clone()).list(),
            Vec::<String>::new()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
