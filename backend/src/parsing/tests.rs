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
    assert_eq!(q.answer_line, "**(D)**。");
    assert_eq!(q.solution.len(), 2);
}

#[test]
fn parse_fill_in_infers_type_and_keeps_solution_under_its_heading() {
    // solution paragraphs are ONLY taken from the `## 解析` section; extra
    // answer paragraphs are no longer reinterpreted as solution
    let md = "> [数学] 示例题库 · 演示 | 第1章 函数极限与连续 @ P15-11\n\n求极限 $\\lim x=$______。\n\n## 答案\n\n$\\dfrac{1}{2}$。\n\n泰勒展开。\n";
    let q = parse_md(md).unwrap();
    assert_eq!(q.qtype, "填空题");
    assert_eq!(q.chapter, "第1章 函数极限与连续");
    assert!(q.options.is_empty());
    assert_eq!(q.correct_id, None);
    assert_eq!(q.answer_line, "$\\dfrac{1}{2}$。");
    assert!(q.solution.is_empty(), "legacy inference is gone");
}

#[test]
fn parse_proof_only_solution() {
    let md = "> [数学] 示例题库 · 演示 | 第6章 @ P50-6 > 证明题\n\n设 $x>0$，证明不等式。\n\n## 解析\n\n令 $f(t)=\\ln(1+t)$：\n\n> 关于式子的来历：构造辅助函数。\n";
    let q = parse_md(md).unwrap();
    assert_eq!(q.qtype, "证明题");
    assert_eq!(q.answer_line, "");
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
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("P9-9.md");
    fs::write(&path, "题干______。\n\n## 答案\n\n42。\n").unwrap();
    let q = parse_file(&path).unwrap();
    assert_eq!(q.id, "P9-9");
    assert_eq!(q.stem, vec!["题干______。"]);
    let _ = fs::remove_file(&path);
}

#[test]
fn parse_without_id_fails() {
    assert!(parse_md("题干______。\n\n## 答案\n\n42。\n").is_err());
}

/// Real-bank smoke test: `DRILL_BANK_DIR=<bank> cargo test`
#[test]
fn real_bank_smoke() {
    let Ok(dir) = std::env::var("DRILL_BANK_DIR") else {
        return;
    };
    let qs = load_questions_from(std::path::Path::new(&dir), &dir);
    assert!(!qs.is_empty(), "bank is empty");
    for q in &qs {
        // files without a header take their id from the file name, so only
        // compare against locate when a header provided one
        if !q.locate.is_empty() {
            assert_eq!(q.id, q.locate, "{}: locate != id", q.id);
        }
        assert!(!q.stem.is_empty(), "{}: no stem", q.id);
        assert!(
            !q.answer_line.is_empty() || !q.solution.is_empty(),
            "{}: no answer and no solution",
            q.id
        );
        if !q.options.is_empty() {
            assert!(q.correct_id.is_some(), "{}: choice without answer id", q.id);
        }
    }
    println!("{} questions passed validation", qs.len());
}

#[test]
fn parse_crlf_line_endings() {
    let md = "> @ P1-1\r\n\r\n题干（　）。\r\n\r\n(A) 甲\r\n\r\n## 答案\r\n\r\n**(A)**。\r\n";
    let q = parse_md(md).unwrap();
    assert_eq!(q.stem, vec!["题干（　）。"]);
    assert_eq!(q.options[0].text, "甲");
    assert_eq!(q.answer_line, "**(A)**。");
}

#[test]
fn parse_header_only_file_has_no_content() {
    let q = parse_md("> @ P1-1\n").unwrap();
    assert!(q.stem.is_empty());
    assert!(q.options.is_empty());
    assert_eq!(q.answer_line, "");
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

#[test]
fn question_file_filter_skips_template_and_hidden() {
    assert!(is_question_file("P1-1.md"));
    assert!(!is_question_file(".hidden.md"));
    assert!(!is_question_file("template.md"));
    assert!(!is_question_file("模板.md"));
    assert!(!is_question_file("TEMPLATE.MD"));
    assert!(!is_question_file("notes.txt"));
}

#[test]
fn missing_source_directory_yields_empty_list() {
    let qs = load_questions_from(std::path::Path::new("/nonexistent-drill-dir"), "s1");
    assert!(qs.is_empty());
}

#[test]
fn parse_file_cached_reparses_when_the_file_changes() {
    let dir = std::env::temp_dir().join(format!("drill-cache-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("P1-1.md");
    fs::write(&path, "> @ P1-1\n\n题干一______。\n\n## 答案\n\n1。\n").unwrap();

    let q1 = parse_file_cached(&path).unwrap();
    assert_eq!(q1.stem, vec!["题干一______。"]);
    // second read hits the cache and returns the same value
    assert_eq!(parse_file_cached(&path).unwrap().stem, q1.stem);

    // rewrite with a different size: mtime/size change invalidates the entry
    fs::write(
        &path,
        "> @ P1-1\n\n题干二（更长）______。\n\n## 答案\n\n2。\n",
    )
    .unwrap();
    let q2 = parse_file_cached(&path).unwrap();
    assert_eq!(
        q2.stem,
        vec!["题干二（更长）______。"],
        "changed file must be reparsed"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn parse_cache_is_bounded() {
    let dir = std::env::temp_dir().join(format!("drill-cache-cap-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    // PARSE_CACHE_MAX + 1 distinct files: the cache must stay bounded
    for i in 0..=PARSE_CACHE_MAX {
        fs::write(
            dir.join(format!("P{i}.md")),
            format!("> @ P{i}\n\n题干______。\n\n## 答案\n\n{i}。\n"),
        )
        .unwrap();
    }
    for i in 0..=PARSE_CACHE_MAX {
        let q = parse_file_cached(&dir.join(format!("P{i}.md"))).unwrap();
        assert_eq!(q.id, format!("P{i}"));
    }
    assert!(
        lock(cache()).len() <= PARSE_CACHE_MAX,
        "cache must stay bounded"
    );
    let _ = fs::remove_dir_all(&dir);
}
