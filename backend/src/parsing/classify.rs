use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use super::parse::parse;
use crate::model::Question;

const KNOWN_H2: [&str; 3] = ["答案", "解析", "备注"];

#[derive(Clone, Debug)]
pub enum Classified {
    Question(Box<Question>),
    Rejected(String),
}

type ClassCache = HashMap<PathBuf, (SystemTime, u64, Classified)>;

const PARSE_CACHE_MAX: usize = 512;

fn cache() -> &'static Mutex<ClassCache> {
    static CACHE: OnceLock<Mutex<ClassCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn classify_file_cached(path: &Path) -> Classified {
    let meta = fs::metadata(path);
    let (mtime, size) = match &meta {
        Ok(m) => (m.modified().unwrap_or(SystemTime::UNIX_EPOCH), m.len()),
        Err(e) => return Classified::Rejected(format!("无法读取：{e}")),
    };
    {
        let cache = crate::fsutil::lock(cache());
        if let Some((t, s, c)) = cache.get(path) {
            if *t == mtime && *s == size {
                return c.clone();
            }
        }
    }
    let c = classify_path(path);
    let mut cache = crate::fsutil::lock(cache());
    if cache.len() >= PARSE_CACHE_MAX {
        cache.clear();
    }
    cache.insert(path.to_path_buf(), (mtime, size, c.clone()));
    c
}

fn classify_path(path: &Path) -> Classified {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let Ok(content) = fs::read_to_string(path) else {
        return Classified::Rejected("无法读取".to_string());
    };
    classify(name, &content)
}

pub fn classify(name: &str, content: &str) -> Classified {
    if let Some(reason) = reject_by_headings(content) {
        return Classified::Rejected(reason);
    }
    let stem = name.strip_suffix(".md").unwrap_or(name);
    match parse(content, Some(stem)) {
        Ok(q) => match question_defect(&q) {
            Some(defect) => Classified::Rejected(defect.to_string()),
            None => Classified::Question(Box::new(q)),
        },
        Err(_) => Classified::Rejected("缺少定位且无法从文件名推断".to_string()),
    }
}

fn reject_by_headings(content: &str) -> Option<String> {
    let mut in_fence = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if t.starts_with("# ") {
            return Some("含一级标题".to_string());
        }
        if let Some(rest) = t.strip_prefix("## ") {
            let title = rest.trim();
            if !KNOWN_H2.contains(&title) {
                return Some(format!("含非标二级标题「{title}」"));
            }
        }
    }
    None
}

pub fn question_defect(q: &Question) -> Option<&'static str> {
    if q.stem.is_empty() {
        return Some("题目为空");
    }
    if q.answer.is_empty() && q.solution.is_empty() {
        return Some("无答案与解析");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse_md;

    #[test]
    fn classify_file_cached_reparses_when_the_file_changes() {
        let dir = std::env::temp_dir().join(format!("drill-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("P1-1.md");
        std::fs::write(&path, "> @ P1-1\n\n题干一______。\n\n## 答案\n\n1。\n").unwrap();

        let q1 = match classify_file_cached(&path) {
            Classified::Question(q) => q,
            other => panic!("expected a question, got {other:?}"),
        };
        assert_eq!(q1.stem, vec!["题干一______。"]);
        assert!(matches!(classify_file_cached(&path), Classified::Question(_)));

        std::fs::write(
            &path,
            "> @ P1-1\n\n题干二（更长）______。\n\n## 答案\n\n2。\n",
        )
        .unwrap();
        match classify_file_cached(&path) {
            Classified::Question(q2) => assert_eq!(
                q2.stem,
                vec!["题干二（更长）______。"],
                "changed file must be reparsed"
            ),
            other => panic!("expected a question, got {other:?}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_cache_is_bounded() {
        let dir = std::env::temp_dir().join(format!("drill-cache-cap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..=PARSE_CACHE_MAX {
            std::fs::write(
                dir.join(format!("P{i}.md")),
                format!("> @ P{i}\n\n题干______。\n\n## 答案\n\n{i}。\n"),
            )
            .unwrap();
        }
        for i in 0..=PARSE_CACHE_MAX {
            match classify_file_cached(&dir.join(format!("P{i}.md"))) {
                Classified::Question(q) => assert_eq!(q.id, format!("P{i}")),
                other => panic!("expected a question, got {other:?}"),
            }
        }
        assert!(
            crate::fsutil::lock(cache()).len() <= PARSE_CACHE_MAX,
            "cache must stay bounded"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn rejection_of(content: &str) -> Option<String> {
        match classify("P1-1.md", content) {
            Classified::Rejected(reason) => Some(reason),
            Classified::Question(_) => None,
        }
    }

    #[test]
    fn classify_rejects_heading_violations() {
        let h1 = "# 总标题\n\n题干______。\n\n## 答案\n\n42。\n";
        assert_eq!(rejection_of(h1).as_deref(), Some("含一级标题"));

        let late_h1 = "题干______。\n\n## 答案\n\n42。\n\n# 附加章节\n";
        assert_eq!(rejection_of(late_h1).as_deref(), Some("含一级标题"));

        let unknown = "题干______。\n\n## 知识体系\n\n内容。\n\n## 答案\n\n42。\n";
        assert_eq!(
            rejection_of(unknown).as_deref(),
            Some("含非标二级标题「知识体系」")
        );

        assert_eq!(
            rejection_of("题干______。\n\n## 答案\n\n42。\n\n## 备注\n\nnote。\n"),
            None
        );

        let fenced = "题干______。\n\n```python\n# 注释\nn = 1\n```\n\n## 答案\n\n42。\n";
        assert_eq!(rejection_of(fenced), None);
    }

    #[test]
    fn classify_rejects_blank_question_and_missing_answer() {
        let blank = "## 答案\n\n42。\n";
        assert_eq!(rejection_of(blank).as_deref(), Some("题目为空"));

        let no_ans = "> @ P1-1\n\n题干______。\n\n## 备注\n\n只有备注。\n";
        assert_eq!(rejection_of(no_ans).as_deref(), Some("无答案与解析"));

        assert_eq!(rejection_of("题干______。\n\n## 解析\n\n步骤。\n"), None);
        assert_eq!(rejection_of("题干______。\n\n## 答案\n\n42。\n"), None);
    }

    #[test]
    fn classify_rejects_unparseable_documents() {
        match parse_md("随便什么内容") {
            Err(_) => {}
            Ok(q) => panic!("should not parse, got {q:?}"),
        }
        let reason = match classify("x.md", "随便什么内容") {
            Classified::Rejected(r) => r,
            other => panic!("expected rejection, got {other:?}"),
        };
        assert_eq!(reason, "无答案与解析");
    }
}
