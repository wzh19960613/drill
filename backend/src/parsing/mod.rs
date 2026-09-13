mod classify;
mod header;
mod parse;
mod scan;
mod sections;
mod text;

#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::fsutil::log_warn;
use crate::model::Question;

pub use classify::{classify, classify_file_cached, Classified};
pub use parse::parse_md;
pub use scan::{scan_dir, DirScan};
pub use text::nat_cmp;

pub(crate) use scan::{find_question_by_id, find_question_file, walk_dirs};

pub fn load_questions_from(
    dir: &Path,
    source: &str,
    recursive: bool,
    excluded: &HashSet<String>,
) -> Vec<Question> {
    let mut out = Vec::new();
    for sub in walk_dirs(dir, recursive) {
        let entries = match fs::read_dir(&sub) {
            Ok(e) => e,
            Err(e) => {
                log_warn(&format!(
                    "[warn] cannot read directory {}: {e}",
                    sub.display()
                ));
                continue;
            }
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if !is_question_file(name) {
                continue;
            }
            if rel_of(dir, &p).is_some_and(|rel| is_excluded(&rel, excluded)) {
                continue;
            }
            if let Classified::Question(mut q) = classify_file_cached(&p) {
                q.source = source.to_string();
                q.file = name.to_string();
                out.push(*q);
            }
        }
    }
    out.sort_by(|a, b| nat_cmp(&a.id, &b.id));
    out
}

pub(crate) fn rel_of(root: &Path, p: &Path) -> Option<String> {
    let rel = p.strip_prefix(root).ok()?;
    let rel = rel.to_string_lossy().replace('\\', "/");
    Some(rel.trim_start_matches('/').to_string())
}

pub(crate) fn is_question_file(name: &str) -> bool {
    name.ends_with(".md") && !name.starts_with('.')
}

/// A path is hidden when it — or any ancestor folder — carries the manual
/// non-question mark (folder marks cover their whole subtree).
pub(crate) fn is_excluded(rel: &str, excluded: &HashSet<String>) -> bool {
    let mut cur = rel;
    loop {
        if excluded.contains(cur) {
            return true;
        }
        match cur.rfind('/') {
            Some(i) => cur = &cur[..i],
            None => return false,
        }
    }
}
