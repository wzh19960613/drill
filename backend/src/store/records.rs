use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::fsutil::{atomic_write, log_warn};
use crate::model::{now_ms, Record};
use crate::store::json_store::JsonStore;

#[derive(Serialize, Deserialize, Default)]
struct SaveFile {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    records: Vec<Record>,
}

pub struct RecordStore {
    store: JsonStore<SaveFile>,
}

impl RecordStore {
    pub fn load(path: PathBuf) -> RecordStore {
        RecordStore {
            store: JsonStore::load(path),
        }
    }

    pub fn list(&self) -> Vec<Record> {
        self.store.view(|f| f.records.clone())
    }

    pub fn add(
        &self,
        source: String,
        question_id: String,
        correct: bool,
        ms: Option<u64>,
    ) -> Record {
        let (record, bytes) = self.store.edit(|f| {
            let record = Record {
                id: f.next_id.max(1),
                question_id,
                correct,
                at: now_ms(),
                ms,
                source,
            };
            f.next_id = record.id + 1;
            f.records.push(record.clone());
            (record, serde_json::to_vec(&*f).unwrap_or_default())
        });
        self.write(&bytes);
        record
    }

    pub fn update(&self, id: u64, correct: bool, ms: Option<u64>) -> bool {
        let bytes = self.store.edit(|f| {
            f.records.iter_mut().find(|r| r.id == id).map(|r| {
                r.correct = correct;
                r.ms = ms;
            })?;
            Some(serde_json::to_vec(&*f).unwrap_or_default())
        });
        match bytes {
            Some(bytes) => {
                self.write(&bytes);
                true
            }
            None => false,
        }
    }

    pub fn delete(&self, id: u64) -> bool {
        let bytes = self.store.edit(|f| {
            let before = f.records.len();
            f.records.retain(|r| r.id != id);
            (before != f.records.len()).then(|| serde_json::to_vec(&*f).unwrap_or_default())
        });
        match bytes {
            Some(bytes) => {
                self.write(&bytes);
                true
            }
            None => false,
        }
    }

    fn write(&self, bytes: &[u8]) {
        if let Err(e) = atomic_write(self.store.path(), bytes) {
            log_warn(&format!(
                "[warn] cannot persist {}: {e}",
                self.store.path().display()
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> RecordStore {
        let dir = std::env::temp_dir().join(format!("drill-rec-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        RecordStore::load(dir.join("records.json"))
    }

    fn add1(s: &RecordStore, qid: &str, tag: u64) -> Record {
        s.add(format!("s{tag}"), qid.into(), true, Some(1200))
    }

    #[test]
    fn add_assigns_incrementing_ids_and_persists() {
        let dir = std::env::temp_dir().join(format!("drill-rec-add-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let s = RecordStore::load(dir.join("records.json"));
        let a = s.add("s1".into(), "P1".into(), true, Some(1200));
        let b = s.add("s2".into(), "P2".into(), false, None);
        assert_eq!((a.id, b.id), (1, 2));
        assert!(a.at > 0);
        assert_eq!(a.source, "s1");

        let reloaded = RecordStore::load(dir.join("records.json"));
        let records = reloaded.list();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].question_id, "P1");
        assert_eq!(records[1].source, "s2");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_and_delete_report_misses() {
        let s = store("miss");
        add1(&s, "P1", 1);
        assert!(!s.update(99, false, None), "unknown id");
        assert!(!s.delete(99), "unknown id");
        assert!(s.update(1, false, Some(5)));
        assert!(!s.list()[0].correct);
        assert!(s.delete(1));
        assert!(s.list().is_empty());
    }

    #[test]
    fn corrupted_file_starts_fresh() {
        let dir = std::env::temp_dir().join(format!("drill-rec-bad-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("records.json"), "{broken").unwrap();
        let s = RecordStore::load(dir.join("records.json"));
        assert!(s.list().is_empty());
        assert_eq!(add1(&s, "P1", 1).id, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
