use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

/// Write via temp file + rename so a crash never truncates the target.
/// Fails (instead of silently dropping the data) when the directory cannot
/// be created, the temp file cannot be written, or the rename fails; the
/// temp file is cleaned up on failure.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
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

/// A name is safe to join onto a directory as a single path component when
/// it is non-empty, not a dotfile (`.`/`..`/`.hidden`), and contains no
/// separators or NUL bytes.
pub fn is_safe_component(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\', '\0'])
}

/// Lock a std Mutex, healing a poisoned lock instead of panicking: the
/// guarded data is never left structurally invalid, so the next holder can
/// safely carry on after a previous holder panicked.
pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Print a line to stdout, ignoring write failures: `println!` panics when the
/// stdout pipe is closed (e.g. the service was started under `| head`), which
/// would take down every request that merely tried to log.
pub fn log_line(line: &str) {
    let stdout = std::io::stdout();
    let _ = writeln!(stdout.lock(), "{line}");
}

/// Same as [`log_line`] for stderr diagnostics.
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
        // a plain file where a directory would be needed blocks the write
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
}
