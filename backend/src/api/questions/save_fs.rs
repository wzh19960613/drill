use std::fs;
use std::path::Path;

pub(super) fn split_rel(original: &str) -> (String, String) {
    match original.rfind('/') {
        Some(i) => (original[..i].to_string(), original[i + 1..].to_string()),
        None => (String::new(), original.to_string()),
    }
}

pub(super) fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(from, to)?;
            fs::remove_file(from)
        }
    }
}
