use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{de::DeserializeOwned, Serialize};

use crate::fsutil::{atomic_write, lock, log_warn};

pub struct JsonStore<T> {
    path: PathBuf,
    inner: Mutex<T>,
}

impl<T: Serialize + DeserializeOwned + Default> JsonStore<T> {
    pub fn load(path: PathBuf) -> JsonStore<T> {
        let value = match fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                eprintln!(
                    "[warn] corrupted store file {}, resetting to defaults: {e}",
                    path.display()
                );
                T::default()
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => T::default(),
            Err(e) => {
                log_warn(&format!(
                    "[warn] cannot read store file {}, starting empty: {e}",
                    path.display()
                ));
                T::default()
            }
        };
        JsonStore {
            path,
            inner: Mutex::new(value),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn view<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&*lock(&self.inner))
    }

    pub fn mutate<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = lock(&self.inner);
        let out = f(&mut guard);
        self.persist(&guard);
        out
    }

    pub fn edit<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut *lock(&self.inner))
    }

    pub fn persist(&self, t: &T) {
        let bytes = match serde_json::to_vec_pretty(t) {
            Ok(b) => b,
            Err(e) => {
                log_warn(&format!(
                    "[warn] cannot serialize store {}: {e}",
                    self.path.display()
                ));
                return;
            }
        };
        if let Err(e) = atomic_write(&self.path, &bytes) {
            log_warn(&format!(
                "[warn] cannot write store {}: {e}",
                self.path.display()
            ));
        }
    }

    pub fn save(&self) {
        let guard = lock(&self.inner);
        self.persist(&guard);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    struct Data {
        #[serde(default)]
        items: Vec<u32>,
    }

    fn tmpfile(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("drill-jsonstore-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("store.json")
    }

    #[test]
    fn missing_file_loads_default_and_mutate_persists() {
        let path = tmpfile("fresh");
        let store = JsonStore::<Data>::load(path.clone());
        store.mutate(|d| d.items.push(7));
        let reloaded = JsonStore::<Data>::load(path.clone());
        assert_eq!(reloaded.view(|d| d.items.clone()), vec![7]);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn corrupted_file_resets_to_default() {
        let path = tmpfile("corrupt");
        fs::write(&path, "{broken").unwrap();
        let store = JsonStore::<Data>::load(path.clone());
        assert_eq!(store.view(|d| d.items.clone()), Vec::<u32>::new());
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn edit_does_not_persist_until_save() {
        let path = tmpfile("edit");
        let store = JsonStore::<Data>::load(path.clone());
        store.edit(|d| d.items.push(1));
        assert_eq!(
            JsonStore::<Data>::load(path.clone()).view(|d| d.items.clone()),
            Vec::<u32>::new(),
            "edit alone must not touch the disk"
        );
        store.save();
        assert_eq!(
            JsonStore::<Data>::load(path.clone()).view(|d| d.items.clone()),
            vec![1]
        );
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
