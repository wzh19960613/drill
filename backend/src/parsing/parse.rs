use std::collections::HashSet;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use regex::Regex;

use super::header;
use super::sections;
use crate::model::{QOption, Question};

pub fn parse_md(content: &str) -> Result<Question> {
    parse(content, None)
}

pub(crate) fn parse(content: &str, file_stem: Option<&str>) -> Result<Question> {
    let lines: Vec<&str> = content.lines().collect();
    let (header, body_start) = read_header(&lines);
    let meta = header::parse_header(&header);
    let parts = sections::split(&lines, body_start);
    let (stem, options) = split_stem_options(&parts.body);
    let answer_line = parts.answer.first().cloned().unwrap_or_default();
    let correct_ids = correct_ids(&options, &answer_line);
    let file = file_stem.unwrap_or_default().to_string() + ".md";
    let file = if file == ".md" { String::new() } else { file };
    Ok(Question {
        id: resolve_id(&meta.locate, file_stem)?,
        source: String::new(),
        file,
        subject: meta.subject,
        origin: meta.origin,
        locate: meta.locate,
        chapter: meta.chapter,
        qtype: infer_qtype(&meta.qtype, &options, &stem),
        stem,
        options,
        correct_id: correct_ids.first().copied(),
        correct_ids,
        answer: parts.answer,
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

#[cfg(test)]
mod tests {
    use super::*;

    const CHOICE: &str = "> [数学] 示例题库 · 演示 | 第3章 一元函数微分学的概念 @ P29-10 > 选择题\n\n设 $f(x)$，则（　）。\n\n(A) 不连续\n\n(B) 连续，但不可导\n\n(C) 可导\n\n(D) 可导且连续\n\n## 答案\n\n**(D)**。\n\n## 解析\n\n连续性：$\\lim x=0$。\n\n可导性：其余步骤。\n";

    #[test]
    fn parse_choice() {
        let q = parse_md(CHOICE).unwrap();
        assert_eq!(q.id, "P29-10");
        assert_eq!(q.chapter, "第3章 一元函数微分学的概念");
        assert_eq!(q.qtype, "选择题");
        assert_eq!(q.stem, vec!["设 $f(x)$，则（　）。"]);
        assert_eq!(q.options.len(), 4);
        assert_eq!((q.options[0].id, q.options[0].text.as_str()), (1, "不连续"));
        assert_eq!(q.correct_id, Some(4));
        assert_eq!(q.correct_ids, vec![4]);
        assert_eq!(q.answer, vec!["**(D)**。"]);
        assert_eq!(q.solution.len(), 2);
    }

    #[test]
    fn parse_fill_in_infers_type_and_keeps_solution_under_its_heading() {
        let md = "> [数学] 示例题库 · 演示 | 第1章 函数极限与连续 @ P15-11\n\n求极限 $\\lim x=$______。\n\n## 答案\n\n$\\dfrac{1}{2}$。\n\n泰勒展开。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.qtype, "填空题");
        assert_eq!(q.chapter, "第1章 函数极限与连续");
        assert!(q.options.is_empty());
        assert_eq!(q.correct_id, None);
        assert_eq!(q.answer, vec!["$\\dfrac{1}{2}$。", "泰勒展开。"]);
        assert!(q.solution.is_empty(), "legacy inference is gone");
    }

    #[test]
    fn parse_proof_only_solution() {
        let md = "> [数学] 示例题库 · 演示 | 第6章 @ P50-6 > 证明题\n\n设 $x>0$，证明不等式。\n\n## 解析\n\n令 $f(t)=\\ln(1+t)$：\n\n> 关于式子的来历：构造辅助函数。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.qtype, "证明题");
        assert!(q.answer.is_empty());
        assert_eq!(q.solution.len(), 2);
        assert!(q.solution[1].starts_with("> 关于"));
    }

    #[test]
    fn parse_image_paragraph_and_extra_answer_text() {
        let md = "> [数学] 示例题库 · 演示 | 第5章 几何应用 @ P40-5 > 选择题\n\n拐点个数（　）。\n\n![[P40-5图.svg]]\n\n(A) $1$\n\n(B) $2$\n\n(C) $3$\n\n(D) $4$\n\n## 答案\n\n**(C) 3 个**。\n\n## 解析\n\n解析。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.correct_id, Some(3));
        assert_eq!(q.stem.len(), 2);
        assert_eq!(q.stem[1], "![[P40-5图.svg]]");
    }

    #[test]
    fn parse_multi_choice_with_notes() {
        let md = "> [数学] 某卷 | 第2章 @ P10-1 > 多选题\n\n下列正确的是（　）。\n\n(A) 甲\n\n(B) 乙\n\n(C) 丙\n\n(D) 丁\n\n## 答案\n\n**(A)(C)**。\n\n## 解析\n\n甲丙正确。\n\n## 备注\n\n本题易错。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.qtype, "多选题");
        assert_eq!(q.correct_ids, vec![1, 3]);
        assert_eq!(q.correct_id, Some(1));
        assert_eq!(q.solution, vec!["甲丙正确。"]);
        assert_eq!(q.notes, vec!["本题易错。"]);
    }

    #[test]
    fn parse_partial_header_fields() {
        let md = "> 2026 上海高考 @ P25-12 > 选择题\n\n题干（　）。\n\n(A) 甲\n\n(B) 乙\n\n## 答案\n\n**(B)**。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.id, "P25-12");
        assert_eq!(q.subject, "");
        assert_eq!(q.origin, "2026 上海高考");
        assert_eq!(q.chapter, "");
        assert_eq!(q.qtype, "选择题");
    }

    #[test]
    fn parse_without_header_uses_filename() {
        let dir = std::env::temp_dir().join("drill-parse-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("P9-9.md");
        std::fs::write(&path, "题干______。\n\n## 答案\n\n42。\n").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let q = parse(&content, Some("P9-9")).unwrap();
        assert_eq!(q.id, "P9-9");
        assert_eq!(q.stem, vec!["题干______。"]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn parse_without_id_fails() {
        assert!(parse_md("题干______。\n\n## 答案\n\n42。\n").is_err());
    }

    #[test]
    fn parse_crlf_line_endings() {
        let md = "> @ P1-1\r\n\r\n题干（　）。\r\n\r\n(A) 甲\r\n\r\n## 答案\r\n\r\n**(A)**。\r\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.stem, vec!["题干（　）。"]);
        assert_eq!(q.options[0].text, "甲");
        assert_eq!(q.answer, vec!["**(A)**。"]);
    }

    #[test]
    fn parse_header_only_file_has_no_content() {
        let q = parse_md("> @ P1-1\n").unwrap();
        assert!(q.stem.is_empty());
        assert!(q.options.is_empty());
        assert!(q.answer.is_empty());
    }

    #[test]
    fn answer_letters_dedup_and_keep_order() {
        let md = "> @ P1-1\n\n题（　）。\n\n(A) 甲\n\n(B) 乙\n\n## 答案\n\n**(B)(A)(B)**。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.correct_ids, vec![2, 1]);
        assert_eq!(q.correct_id, Some(2));
    }

    #[test]
    fn option_paragraph_continues_on_next_line() {
        let md = "> @ P1-1\n\n题（　）。\n\n(A) 第一行\n第二行\n\n## 答案\n\n**(A)**。\n";
        let q = parse_md(md).unwrap();
        assert_eq!(q.options[0].text, "第一行\n第二行");
    }

    #[test]
    fn lowercase_option_markers_are_not_options() {
        let md = "> @ P1-1\n\n题 (a) 不是选项。\n\n## 答案\n\n42。\n";
        let q = parse_md(md).unwrap();
        assert!(q.options.is_empty());
        assert_eq!(q.stem, vec!["题 (a) 不是选项。"]);
    }
}
