use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = path.with_extension(format!("tmp{}-{seq}", std::process::id()));
    if let Err(e) = fs::write(&tmp, bytes) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

pub fn is_safe_component(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\', '\0'])
}

pub fn is_safe_rel_path(rel: &str) -> bool {
    rel.is_empty()
        || (!rel.starts_with('/')
            && !rel.starts_with('\\')
            && rel.split('/').all(is_safe_component))
}

pub fn find_recursive(root: &Path, name: &str) -> Option<PathBuf> {
    if !is_safe_component(name) {
        return None;
    }
    if root.join(name).is_file() {
        return Some(root.join(name));
    }
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    const MAX_DEPTH: usize = 16;
    const MAX_DIRS: usize = 5000;
    let mut visited = 0usize;
    while let Some((dir, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH || visited >= MAX_DIRS {
            break;
        }
        visited += 1;
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let p = entry.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .is_none_or(|n| n.starts_with('.'))
            {
                continue;
            }
            let hit = p.join(name);
            if hit.is_file() {
                return Some(hit);
            }
            queue.push_back((p, depth + 1));
        }
    }
    None
}

pub fn find_all_recursive(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !is_safe_component(name) {
        return out;
    }
    if root.join(name).is_file() {
        out.push(root.join(name));
    }
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    const MAX_DEPTH: usize = 16;
    const MAX_DIRS: usize = 5000;
    let mut visited = 0usize;
    while let Some((dir, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH || visited >= MAX_DIRS {
            break;
        }
        visited += 1;
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let p = entry.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .is_none_or(|n| n.starts_with('.'))
            {
                continue;
            }
            let hit = p.join(name);
            if hit.is_file() {
                out.push(hit);
            }
            queue.push_back((p, depth + 1));
        }
    }
    out
}

pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn log_line(line: &str) {
    let stdout = std::io::stdout();
    let _ = writeln!(stdout.lock(), "{line}");
}

pub fn log_warn(line: &str) {
    let stderr = std::io::stderr();
    let _ = writeln!(stderr.lock(), "{line}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_into_missing_directories() {
        let dir = std::env::temp_dir().join(format!("drill-fsutil-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let target = dir.join("nested").join("file.json");
        atomic_write(&target, b"{\"a\":1}").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"{\"a\":1}");
        atomic_write(&target, b"{}").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"{}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reports_errors_and_cleans_up_tmp() {
        let dir = std::env::temp_dir().join(format!("drill-fsutil-err-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let blocker = dir.join("blocker");
        std::fs::write(&blocker, b"").unwrap();
        let target = blocker.join("nested").join("file.json");
        assert!(atomic_write(&target, b"x").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn safe_component_bounds() {
        assert!(is_safe_component("P1-1.md"));
        assert!(is_safe_component("题本 A4.pdf"));
        assert!(!is_safe_component(""), "empty");
        assert!(!is_safe_component("."), "current dir");
        assert!(!is_safe_component(".."), "parent dir");
        assert!(!is_safe_component(".hidden"), "dotfile");
        assert!(!is_safe_component("a/b.md"), "slash");
        assert!(!is_safe_component("a\\b.md"), "backslash");
        assert!(!is_safe_component("a\0b"), "NUL");
    }

    #[test]
    fn safe_rel_path_bounds() {
        assert!(is_safe_rel_path(""), "root");
        assert!(is_safe_rel_path("数学/张宇1000"));
        assert!(!is_safe_rel_path("/abs"), "absolute");
        assert!(!is_safe_rel_path("../escape"), "traversal");
        assert!(!is_safe_rel_path("a//b"), "empty component");
        assert!(!is_safe_rel_path("a/.hidden"), "hidden component");
    }

    #[test]
    fn finds_files_recursively_skipping_hidden() {
        let dir = std::env::temp_dir().join(format!("drill-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::create_dir_all(dir.join(".git/objects")).unwrap();
        std::fs::write(dir.join("a/b/deep.svg"), "<svg/>").unwrap();
        std::fs::write(dir.join(".git/stash.svg"), "<svg/>").unwrap();

        assert_eq!(
            find_recursive(&dir, "deep.svg"),
            Some(dir.join("a/b/deep.svg"))
        );
        assert_eq!(find_recursive(&dir, "stash.svg"), None, "hidden dirs skipped");
        assert_eq!(find_recursive(&dir, "missing.svg"), None);
        assert_eq!(find_recursive(&dir, "../escape"), None, "unsafe name");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_all_returns_every_match_in_bfs_order() {
        let dir = std::env::temp_dir().join(format!("drill-findall-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("dup.md"), "x").unwrap();
        std::fs::write(dir.join("a/dup.md"), "x").unwrap();
        std::fs::write(dir.join("a/b/dup.md"), "x").unwrap();
        assert_eq!(
            find_all_recursive(&dir, "dup.md"),
            vec![
                dir.join("dup.md"),
                dir.join("a/dup.md"),
                dir.join("a/b/dup.md")
            ]
        );
        assert!(find_all_recursive(&dir, "none.md").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
