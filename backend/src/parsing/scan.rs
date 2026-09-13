use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::classify::{classify_file_cached, Classified};
use super::text::nat_cmp;
use super::{is_excluded, is_question_file, rel_of};

pub(crate) fn walk_dirs(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut out = vec![dir.to_path_buf()];
    if !recursive {
        return out;
    }
    let mut queue = std::collections::VecDeque::from(vec![(dir.to_path_buf(), 0usize)]);
    const MAX_DEPTH: usize = 16;
    const MAX_DIRS: usize = 5000;
    while let Some((d, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH || out.len() >= MAX_DIRS {
            break;
        }
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        let mut subdirs: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| !n.starts_with('.'))
            })
            .collect();
        subdirs.sort_by(|a, b| nat_cmp(&a.to_string_lossy(), &b.to_string_lossy()));
        for sd in subdirs {
            queue.push_back((sd.clone(), depth + 1));
            out.push(sd);
        }
    }
    out
}

#[derive(Default, Debug)]
pub struct DirScan {
    pub questions: Vec<QuestionFile>,
    pub other_md: Vec<OtherFile>,
    pub dirs: Vec<String>,
    pub files: Vec<String>,
    pub dir_info: Vec<DirInfo>,
}

#[derive(Debug)]
pub struct DirInfo {
    pub name: String,
    pub questions: usize,
    pub other_md: usize,
    /// questions in this folder's whole subtree (own + nested folders)
    pub questions_subtree: usize,
}

#[derive(Debug)]
pub struct QuestionFile {
    pub file: String,
    pub id: String,
}

#[derive(Debug)]
pub struct OtherFile {
    pub file: String,
    pub reason: String,
}

pub fn scan_dir(root: &Path, dir: &Path, excluded: &HashSet<String>) -> DirScan {
    scan_dir_at(root, dir, excluded, 0)
}

const SCAN_MAX_DEPTH: usize = 16;

fn scan_dir_at(root: &Path, dir: &Path, excluded: &HashSet<String>, depth: usize) -> DirScan {
    let mut scan = DirScan::default();
    let Ok(entries) = fs::read_dir(dir) else {
        return scan;
    };
    let mut names: Vec<(String, bool)> = entries
        .flatten()
        .filter_map(|e| {
            let ft = e.file_type().ok()?;
            if ft.is_symlink() {
                return None;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            (!name.starts_with('.')).then_some((name, ft.is_dir()))
        })
        .collect();
    names.sort_by(|a, b| nat_cmp(&a.0, &b.0));
    for (name, is_dir) in names {
        scan_entry(&mut scan, root, dir, &name, is_dir, excluded, depth);
    }
    scan
}

#[allow(clippy::too_many_arguments)]
fn scan_entry(
    scan: &mut DirScan,
    root: &Path,
    dir: &Path,
    name: &str,
    is_dir: bool,
    excluded: &HashSet<String>,
    depth: usize,
) {
    let p = dir.join(name);
    if is_dir {
        scan_subdir(scan, root, &p, name, excluded, depth);
        return;
    }
    scan.files.push(name.to_string());
    if !is_question_file(name) {
        return;
    }
    if rel_of(root, &p).is_some_and(|rel| is_excluded(&rel, excluded)) {
        scan.other_md.push(OtherFile {
            file: name.to_string(),
            reason: "已标记为非题目".to_string(),
        });
        return;
    }
    match classify_file_cached(&p) {
        Classified::Question(q) => scan.questions.push(QuestionFile {
            file: name.to_string(),
            id: q.id,
        }),
        Classified::Rejected(reason) => scan.other_md.push(OtherFile {
            file: name.to_string(),
            reason,
        }),
    }
}

fn scan_subdir(
    scan: &mut DirScan,
    root: &Path,
    p: &Path,
    name: &str,
    excluded: &HashSet<String>,
    depth: usize,
) {
    if depth >= SCAN_MAX_DEPTH {
        return;
    }
    scan.dirs.push(name.to_string());
    let child = scan_dir_at(root, p, excluded, depth + 1);
    let subtree =
        child.questions.len() + child.dir_info.iter().map(|d| d.questions_subtree).sum::<usize>();
    scan.dir_info.push(DirInfo {
        name: name.to_string(),
        questions: child.questions.len(),
        other_md: child.other_md.len(),
        questions_subtree: subtree,
    });
}

pub(crate) fn find_question_file(
    root: &Path,
    id: &str,
    excluded: &HashSet<String>,
) -> Option<PathBuf> {
    let name = format!("{id}.md");
    crate::fsutil::find_all_recursive(root, &name)
        .into_iter()
        .find(|p| rel_of(root, p).is_none_or(|rel| !is_excluded(&rel, excluded)))
}

pub(crate) fn find_question_by_id(
    root: &Path,
    id: &str,
    excluded: &HashSet<String>,
) -> Option<PathBuf> {
    for dir in walk_dirs(root, true) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_name().and_then(|s| s.to_str()) else {
                continue;
            };
            if !is_question_file(name) {
                continue;
            }
            if rel_of(root, &p).is_some_and(|rel| is_excluded(&rel, excluded)) {
                continue;
            }
            if let Classified::Question(q) = classify_file_cached(&p) {
                if q.id == id {
                    return Some(p);
                }
            }
        }
    }
    None
}
