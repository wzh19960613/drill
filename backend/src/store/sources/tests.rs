use super::*;
use std::fs;

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("drill-src-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn add_relocate_remove() {
    let root = tmpdir("ops");
    let a = root.join("bank-a");
    let b = root.join("bank-b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let store = SourceStore::load(root.join("sources.json"), None);

    let s1 = store.add(a.to_str().unwrap(), &root).unwrap();
    assert_eq!(s1.id, "s1");
    assert!(store.add(a.to_str().unwrap(), &root).is_err());
    assert!(store.add("missing-dir", &root).is_err());

    let moved = store.relocate(&s1.id, b.to_str().unwrap(), &root).unwrap();
    assert_eq!(moved.id, "s1");
    assert_eq!(store.get("s1").unwrap().path, b.canonicalize().unwrap());
    assert!(store.remove("nope").is_err());
    store.remove(&s1.id).unwrap();
    assert!(store.list().is_empty(), "the last source can be removed");
}

#[test]
fn recursive_flag_round_trips_and_defaults_off() {
    let root = tmpdir("recursive");
    let a = root.join("bank-a");
    fs::create_dir_all(&a).unwrap();
    let file = root.join("sources.json");
    let store = SourceStore::load(file.clone(), None);

    let s1 = store.add(a.to_str().unwrap(), &root).unwrap();
    assert!(!s1.recursive, "default is root-only");

    store.set_recursive(&s1.id, true).unwrap();
    assert!(store.get(&s1.id).unwrap().recursive);
    store.set_recursive(&s1.id, false).unwrap();
    assert!(!store.get(&s1.id).unwrap().recursive);
    assert!(store.set_recursive("nope", true).is_err());

    store.set_recursive(&s1.id, true).unwrap();
    let reloaded = SourceStore::load(file, None);
    assert!(reloaded.get(&s1.id).unwrap().recursive);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn load_with_existing_sources_does_not_rewrite_the_file() {
    let root = tmpdir("norewrite");
    let bank = root.join("bank");
    fs::create_dir_all(&bank).unwrap();
    let file = root.join("sources.json");

    let store = SourceStore::load(file.clone(), None);
    store.add(bank.to_str().unwrap(), &root).unwrap();
    let before = fs::metadata(&file).unwrap().modified().unwrap();
    let raw_before = fs::read_to_string(&file).unwrap();

    let _ = SourceStore::load(file.clone(), None);
    let raw_after = fs::read_to_string(&file).unwrap();
    assert_eq!(
        raw_before, raw_after,
        "reload must not rewrite sources.json"
    );
    let after = fs::metadata(&file).unwrap().modified().unwrap();
    assert_eq!(before, after);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn corrupted_sources_file_resets_and_still_works() {
    let root = tmpdir("corrupt");
    let bank = root.join("bank");
    fs::create_dir_all(&bank).unwrap();
    let file = root.join("sources.json");
    fs::write(&file, "{broken").unwrap();

    let store = SourceStore::load(file.clone(), None);
    assert!(store.list().is_empty(), "corrupted file resets the store");
    assert!(
        !file.exists() || fs::read_to_string(&file).unwrap() == "{broken",
        "no rewrite during load"
    );
    let s1 = store.add(bank.to_str().unwrap(), &root).unwrap();
    assert_eq!(s1.id, "s1");
    let reloaded = SourceStore::load(file.clone(), None);
    assert_eq!(reloaded.list().len(), 1);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn explicit_root_seeds_itself_no_root_seeds_nothing() {
    let root = tmpdir("seed");

    let store = SourceStore::load(root.join("sources.json"), None);
    assert!(store.list().is_empty());

    let store = SourceStore::load(root.join("sources.json"), Some(&root));
    let list = store.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].path, root.canonicalize().unwrap());

    let store = SourceStore::load(root.join("sources.json"), Some(&root));
    assert_eq!(store.list().len(), 1);

    let missing = root.join("missing");
    let store = SourceStore::load(root.join("sources2.json"), Some(&missing));
    assert!(store.list().is_empty());

    let _ = fs::remove_dir_all(&root);
}
