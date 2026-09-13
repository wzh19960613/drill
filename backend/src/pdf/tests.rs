use super::*;

use payload::PrintItem;

fn payload(doc: &str, items: Vec<PrintItem>) -> ExportPayload {
    serde_json::from_value(serde_json::json!({
        "doc": doc, "paper": "A4", "perPage": 2,
        "date": "2026-09-03", "title": "测试题本", "bookId": null,
        "fontScale": 1, "optsPerRow": 4, "items": items_value(items),
    }))
    .unwrap()
}

fn items_value(items: Vec<PrintItem>) -> serde_json::Value {
    serde_json::to_value(items).unwrap()
}

fn sample_item() -> PrintItem {
    serde_json::from_value(serde_json::json!({
        "id": "P29-10", "source": "s1", "subject": "数学", "origin": "示例题库",
        "locate": "P29-10", "chapter": "第3章 概念", "qtype": "选择题",
        "stem": ["设 $f(x)=\\begin{cases}x>0\\\\ x\\leqslant 0\\end{cases}$，则（　）。"],
        "options": [
            {"id": 1, "text": "不连续"},
            {"id": 2, "text": "连续，但不可导"},
            {"id": 3, "text": "可导，但导函数不连续"},
            {"id": 4, "text": "可导，且导函数连续"}],
        "correct_id": 4, "correct_ids": [4], "answer": ["**(D)**。"],
        "solution": ["连续性：$\\lim\\limits_{x\\to0}x^2=0$。"], "notes": [],
        "seq": 1, "optionOrder": [2, 0, 3, 1]
    }))
    .unwrap()
}

#[test]
fn safe_filename_keeps_cjk() {
    assert_eq!(safe_filename("题本/1:*.pdf"), "题本-1---pdf");
    assert_eq!(safe_filename("///"), "export");
}

#[test]
fn renders_automatic_control_block_math() {
    let mut item = sample_item();
    item.question.solution = vec![
        "$$\\Phi_E(s)=\\Phi_E(0)+\\dot{\\Phi}_E(0)s+\\frac{1}{2!}\\ddot{\\Phi}_E(0)s^2+\\cdots$$".to_string(),
        "$$c_i=\\frac{1}{i!}\\Phi_E^{(i)}(0)\\qquad i=0,1,2,\\cdots$$".to_string(),
    ];
    let bytes = render(&payload("answers", vec![item]), &[])
        .expect("automatic-control formulas should render");
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn short_answer_pill_only_for_under_20_chars() {
    let mut short = sample_item();
    short.question.answer = vec!["42。".to_string()];
    short.question.options = vec![];
    let bytes = render(&payload("answers", vec![short]), &[]).unwrap();
    assert!(bytes.starts_with(b"%PDF"));

    let mut long = sample_item();
    long.question.answer = vec!["一二三四五六七八九十一二三四五六七八九十一。".to_string()];
    long.question.options = vec![];
    let bytes = render(&payload("answers", vec![long]), &[]).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn renders_partial_derivative_math() {
    let mut item = sample_item();
    item.question.answer = vec![
        "$$y=f(x)=f(x_0)+\\left.\\frac{\\mathrm{d}f}{\\mathrm{d}x}\\right|_{x_0}\\Delta x+\\frac{1}{2!}\\left.\\frac{\\mathrm{d}^2 f}{\\mathrm{d}x^2}\\right|_{x_0}(\\Delta x)^2+\\cdots$$".to_string(),
        "$$Y(s)=\\left(\\frac{\\partial f}{\\partial x_1}\\right)_0 X_1(s)+\\cdots$$".to_string(),
    ];
    let bytes = render(&payload("answers", vec![item]), &[])
        .expect("partial-derivative formulas should render");
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn renders_list_table_and_code_blocks() {
    let mut item = sample_item();
    item.question.stem.push("- 甲\n- 乙".to_string());
    item.question.stem.push("| 量 | 值 |\n|---|---|\n| a | 1 |".to_string());
    item.question.stem.push("```\ncode\n```".to_string());
    item.question.stem.push("1. 第一步\n2. 第二步".to_string());
    let bytes = render(&payload("workbook", vec![item]), &[])
        .expect("render with markdown blocks should succeed");
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn embeds_raster_images_from_source_dirs() {
    let dir = std::env::temp_dir().join(format!("drill-pdf-raster-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    let png: &[u8] = &[
        0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n', 0, 0, 0, 0x0d, b'I', b'H',
        b'D', b'R', 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1f, 0x15, 0xc4, 0x89, 0, 0,
        0, 0x0d, b'I', b'D', b'A', b'T', 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0, 0x1f,
        0, 0x05, 0, 1, 0xff, 0x89, 0x99, 0x3d, 0x1d, 0, 0, 0, 0, b'I', b'E', b'N',
        b'D', 0xae, 0x42, 0x60, 0x82,
    ];
    std::fs::write(dir.join("nested/t.png"), png).unwrap();
    let mut item = sample_item();
    item.question.stem.insert(0, "![[t.png]]".to_string());
    let dirs = [dir.clone()];
    let bytes = render(&payload("workbook", vec![item]), &dirs)
        .expect("render with png should succeed");
    assert!(bytes.starts_with(b"%PDF"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn automatic_control_answers_book_smoke() {
    let Ok(dir) = std::env::var("DRILL_AUTOC_BOOK") else {
        return;
    };
    let root = std::path::PathBuf::from(dir);
    let qs = crate::parsing::load_questions_from(&root, "s2", true, &Default::default());
    let auto: Vec<_> = qs.into_iter().filter(|q| q.subject == "自控").collect();
    assert!(!auto.is_empty(), "no 自控 questions found");
    let items = sequenced_items(&auto);
    let payload: ExportPayload = serde_json::from_value(serde_json::json!({
        "doc": "answers", "paper": "A4", "perPage": 0,
        "date": "2026-09-12", "title": "自控答案本", "bookId": null,
        "fontScale": 1, "optsPerRow": 4, "items": items,
    }))
    .unwrap();
    if std::env::var("DRILL_DEBUG_TYPST").is_ok() {
        let names = document::collect_images(&payload);
        let (slots, inputs) = load_images(&names, std::slice::from_ref(&root));
        let heights = measure_question_heights(&payload, &slots, &inputs);
        let pages = layout::repack(&heights, &payload);
        std::fs::write(
            "/tmp/drill-final.typ",
            document::build_document(&payload, &slots, &pages, &heights),
        )
        .unwrap();
        eprintln!("dumped /tmp/drill-final.typ; pages {}", pages.len());
    }
    let bytes = render(&payload, &[root]).expect("自控 answers book must render");
    assert!(bytes.starts_with(b"%PDF"));
    if std::env::var("DRILL_DUMP_PDF").is_ok() {
        std::fs::write("/tmp/autoc-answers.pdf", &bytes).unwrap();
        eprintln!("dumped /tmp/autoc-answers.pdf ({} pages approx)", bytes.len() / 4000);
    }
}

#[test]
fn renders_workbook_pdf() {
    let payload = payload("workbook", vec![sample_item()]);
    let bytes = render(&payload, &[]).expect("render should succeed");
    assert!(bytes.starts_with(b"%PDF"), "must be a pdf");
}

#[test]
fn renders_answers_pdf() {
    let payload = payload("answers", vec![sample_item()]);
    let bytes = render(&payload, &[]).expect("render should succeed");
    assert!(bytes.starts_with(b"%PDF"), "must be a pdf");
}

fn sequenced_items(questions: &[crate::model::Question]) -> Vec<serde_json::Value> {
    questions
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let mut v = serde_json::to_value(q).unwrap();
            v["seq"] = (i + 1).into();
            v["optionOrder"] = serde_json::Value::Null;
            v
        })
        .collect()
}
