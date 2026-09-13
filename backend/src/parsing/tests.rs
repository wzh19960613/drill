use super::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[test]
fn question_file_filter_only_checks_visibility() {
    assert!(is_question_file("P1-1.md"));
    assert!(!is_question_file(".hidden.md"));
    assert!(!is_question_file("notes.txt"));
    assert!(is_question_file("题目模板.md"));
}

#[test]
fn missing_source_directory_yields_empty_list() {
    let qs = load_questions_from(Path::new("/nonexistent-drill-dir"), "s1", false, &HashSet::new());
    assert!(qs.is_empty());
}

#[test]
fn real_bank_smoke() {
    let Ok(dir) = std::env::var("DRILL_BANK_DIR") else {
        return;
    };
    let qs = load_questions_from(Path::new(&dir), &dir, false, &HashSet::new());
    assert!(!qs.is_empty(), "bank is empty");
    for q in &qs {

        if !q.locate.is_empty() {
            assert_eq!(q.id, q.locate, "{}: locate != id", q.id);
        }
        assert!(!q.stem.is_empty(), "{}: no stem", q.id);
        assert!(
            !q.answer.is_empty() || !q.solution.is_empty(),
            "{}: no answer and no solution",
            q.id
        );
        if !q.options.is_empty() {
            assert!(q.correct_id.is_some(), "{}: choice without answer id", q.id);
        }
    }
    println!("{} questions passed validation", qs.len());
}

fn bank(dir: &Path) {
    fs::write(
        dir.join("P1-1.md"),
        "> @ P1-1\n\n题干______。\n\n## 答案\n\n1。\n",
    )
    .unwrap();
    fs::write(
        dir.join("note.md"),
        "# 笔记\n\n## 知识体系\n\n内容。\n\n## 答案\n\n42。\n",
    )
    .unwrap();
    fs::write(dir.join("noans.md"), "> @ N\n\n只有题干。\n").unwrap();
    fs::write(dir.join("img.svg"), "<svg/>").unwrap();
}

fn recursive_bank_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("drill-recursive-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("子卷")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    bank(&root);
    fs::write(
        root.join("子卷/P2-1.md"),
        "> @ P2-1\n\n子题______。\n\n## 答案\n\n2。\n",
    )
    .unwrap();
    fs::write(root.join(".git/x.md"), "> @ X\n\n题______。\n\n## 答案\n\n9。\n").unwrap();
    root
}

fn loaded_ids(root: &Path, recursive: bool, excluded: &HashSet<String>) -> Vec<String> {
    load_questions_from(root, "s1", recursive, excluded)
        .into_iter()
        .map(|q| q.id)
        .collect()
}

#[test]
fn load_recursive_vs_root_only() {
    let root = recursive_bank_root();
    let empty: HashSet<String> = HashSet::new();
    assert_eq!(
        loaded_ids(&root, false, &empty),
        vec!["P1-1"],
        "root only by default"
    );
    assert_eq!(
        loaded_ids(&root, true, &empty),
        vec!["P1-1", "P2-1"],
        "subfolders when recursive"
    );
    scan_reports_classification(&root);
    find_paths(&root);
    template_and_marks(&root);
    let _ = fs::remove_dir_all(&root);
}

fn scan_reports_classification(root: &Path) {
    let scan = scan_dir(root, root, &HashSet::new());
    assert_eq!(scan.questions.len(), 1);
    assert_eq!(scan.questions[0].id, "P1-1");
    assert_eq!(scan.dirs, vec!["子卷"]);
    assert!(scan.files.contains(&"img.svg".to_string()));
    let other: Vec<&str> = scan.other_md.iter().map(|o| o.file.as_str()).collect();
    assert_eq!(other, vec!["noans.md", "note.md"]);
    assert!(scan.other_md[0].reason.contains("答案"));
    assert!(scan.other_md[1].reason.contains("一级标题"));
}

fn find_paths(root: &Path) {
    let empty: HashSet<String> = HashSet::new();
    assert_eq!(
        find_question_file(root, "P2-1", &empty),
        Some(root.join("子卷/P2-1.md"))
    );
    assert_eq!(find_question_file(root, "missing", &empty), None);
}

fn template_and_marks(root: &Path) {
    template_found_by_parsed_id(root);
    marks_hide_but_keep_files(root);
}

fn template_found_by_parsed_id(root: &Path) {
    let empty: HashSet<String> = HashSet::new();
    std::fs::write(
        root.join("题目模板.md"),
        "> [学科] 来源 | 章节 @ 定位 > 题型\n\n此处写题目。\n\n## 答案\n\n答案。\n",
    )
    .unwrap();
    let qs = load_questions_from(root, "s1", false, &HashSet::new());
    let tpl = qs.iter().find(|q| q.id == "定位").expect("template is a question");
    assert_eq!(tpl.file, "题目模板.md", "file name is exposed");
    assert_eq!(
        find_question_file(root, "定位", &empty),
        None,
        "no file named 定位.md exists"
    );
    assert_eq!(
        find_question_by_id(root, "定位", &empty),
        Some(root.join("题目模板.md")),
        "found by parsed id"
    );
}

fn marks_hide_but_keep_files(root: &Path) {
    let mut marked = HashSet::new();
    marked.insert("P1-1.md".to_string());
    assert_eq!(
        loaded_ids(root, true, &marked),
        vec!["P2-1", "定位"],
        "marked file not loaded"
    );
    assert_eq!(
        find_question_file(root, "P1-1", &marked),
        None,
        "marked file is invisible to lookups too"
    );
    let scan = scan_dir(root, root, &marked);
    assert_eq!(scan.questions.len(), 1, "only the unmarked template remains");
    assert!(
        scan.other_md
            .iter()
            .any(|o| o.file == "P1-1.md" && o.reason == "已标记为非题目")
    );
    assert!(root.join("P1-1.md").is_file(), "file untouched");
}
