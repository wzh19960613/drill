mod header;
mod sections;
mod text;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use crate::fsutil::{lock, log_warn};
use crate::model::{QOption, Question};

pub use text::nat_cmp;

pub fn load_questions_from(dir: &Path, source: &str) -> Vec<Question> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            log_warn(&format!(
                "[warn] cannot read directory {}: {e}",
                dir.display()
            ));
            return out;
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
        match parse_file_cached(&p) {
            Ok(mut q) => {
                q.source = source.to_string();
                out.push(q)
            }
            Err(e) => log_warn(&format!("[warn] failed to parse {}: {e:#}", name)),
        }
    }
    out.sort_by(|a, b| nat_cmp(&a.id, &b.id));
    out
}

fn is_question_file(name: &str) -> bool {
    let stem = name.strip_suffix(".md").unwrap_or(name);
    name.ends_with(".md")
        && !name.starts_with('.')
        && !stem.eq_ignore_ascii_case("template")
        && stem != "模板"
}

pub fn parse_file(path: &Path) -> Result<Question> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    parse(&content, Some(stem))
}

/// Parsed-question cache keyed by path; an entry stays valid while the file's
/// mtime and size are unchanged, so repeated bank reads skip both the disk
/// read and the parse. A size bound keeps the cache from growing without
/// limit across long-lived sessions.
type ParseCache = HashMap<PathBuf, (SystemTime, u64, Question)>;

const PARSE_CACHE_MAX: usize = 64;

fn cache() -> &'static Mutex<ParseCache> {
    static CACHE: OnceLock<Mutex<ParseCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn parse_file_cached(path: &Path) -> Result<Question> {
    let meta = fs::metadata(path);
    let (mtime, size) = match &meta {
        Ok(m) => (m.modified().unwrap_or(SystemTime::UNIX_EPOCH), m.len()),
        Err(e) => return Err(anyhow::anyhow!("stat {}: {e}", path.display())),
    };
    {
        let cache = lock(cache());
        if let Some((t, s, q)) = cache.get(path) {
            if *t == mtime && *s == size {
                return Ok(q.clone());
            }
        }
    }
    let q = parse_file(path)?;
    let mut cache = lock(cache());
    if cache.len() >= PARSE_CACHE_MAX {
        cache.clear();
    }
    cache.insert(path.to_path_buf(), (mtime, size, q.clone()));
    Ok(q)
}

pub fn parse_md(content: &str) -> Result<Question> {
    parse(content, None)
}

fn parse(content: &str, file_stem: Option<&str>) -> Result<Question> {
    let lines: Vec<&str> = content.lines().collect();
    let (header, body_start) = read_header(&lines);
    let meta = header::parse_header(&header);
    let parts = sections::split(&lines, body_start);
    let (stem, options) = split_stem_options(&parts.body);
    let answer_line = parts.answer.first().cloned().unwrap_or_default();
    let correct_ids = correct_ids(&options, &answer_line);
    Ok(Question {
        id: resolve_id(&meta.locate, file_stem)?,
        source: String::new(),
        subject: meta.subject,
        origin: meta.origin,
        locate: meta.locate,
        chapter: meta.chapter,
        qtype: infer_qtype(&meta.qtype, &options, &stem),
        stem,
        options,
        correct_id: correct_ids.first().copied(),
        correct_ids,
        answer_line,
        solution: parts.solution,
        notes: parts.notes,
    })
}

fn read_header(lines: &[&str]) -> (String, usize) {
    let i = lines
        .iter()
        .position(|l| !l.trim().is_empty())
        .unwrap_or(lines.len());
    if i < lines.len() && lines[i].trim_start().starts_with('>') {
        (
            lines[i].trim().trim_start_matches('>').trim().to_string(),
            i + 1,
        )
    } else {
        (String::new(), i)
    }
}

fn resolve_id(locate: &str, file_stem: Option<&str>) -> Result<String> {
    if !locate.is_empty() {
        return Ok(locate.to_string());
    }
    file_stem
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .context("header has no `@ locate`, cannot determine question id")
}

fn split_stem_options(body: &[String]) -> (Vec<String>, Vec<QOption>) {
    let mut stem = Vec::new();
    let mut options = Vec::new();
    for p in body {
        match parse_option(p) {
            Some(o) => options.push(o),
            None => stem.push(p.clone()),
        }
    }
    (stem, options)
}

fn parse_option(p: &str) -> Option<QOption> {
    let c = opt_re().captures(p.split('\n').next()?)?;
    let first = c[2].to_string();
    let rest = p
        .split_once('\n')
        .map(|(_, r)| r.to_string())
        .unwrap_or_default();
    let text = if rest.is_empty() {
        first
    } else {
        format!("{first}\n{rest}")
    };
    let letter = c[1].chars().next()?;
    Some(QOption {
        id: letter_id(letter),
        text: text.trim().to_string(),
    })
}

fn correct_ids(options: &[QOption], answer_line: &str) -> Vec<u8> {
    if options.is_empty() {
        return Vec::new();
    }
    let mut ids: Vec<u8> = Vec::new();
    let mut seen: HashSet<u8> = HashSet::new();
    for c in letter_re().captures_iter(answer_line) {
        let id = letter_id(c[1].chars().next().unwrap_or('A'));
        if seen.insert(id) {
            ids.push(id);
        }
    }
    ids
}

fn infer_qtype(qtype: &str, options: &[QOption], stem: &[String]) -> String {
    if !qtype.is_empty() {
        return qtype.to_string();
    }
    if !options.is_empty() {
        "选择题".into()
    } else if stem.iter().any(|p| p.contains("______")) {
        "填空题".into()
    } else {
        "证明题".into()
    }
}

fn letter_id(c: char) -> u8 {
    (c as u8 - b'A' + 1).min(26)
}

fn opt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[（(]([A-Z])[)）]\s*(.*)$").unwrap())
}

fn letter_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[（(]\s*([A-Z])\s*[)）]").unwrap())
}
