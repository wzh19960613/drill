use super::super::latex;
use super::super::markup::{self, escape_markup, paragraph, Chunk, Paragraph};
use super::super::payload::{answer_letters, display_options, ExportPayload, PrintItem};
use super::blocks::{chunks_markup, display_length, stem};
use super::ImageSlots;

pub(super) fn question_block(
    item: &PrintItem,
    payload: &ExportPayload,
    images: &ImageSlots,
    h: f64,
    capacity: f64,
) -> String {
    let body = question_content(item, payload, images);
    if h > capacity {
        format!("#qblock-flow[\n{}]\n", body)
    } else {
        format!("#qblock({h:.2}mm)[\n{}]\n", body)
    }
}

pub(crate) fn question_content(
    item: &PrintItem,
    payload: &ExportPayload,
    images: &ImageSlots,
) -> String {
    let mut body = String::new();
    let lead = question_lead(item, images);
    match lead.split_once('\u{1}') {
        Some((head, rest)) => {
            body.push_str(&format!(
                "#block(breakable: false, width: 100%)[\n{}{}\n]\n{}",
                meta_line(item, payload),
                head,
                rest
            ));
        }
        None => {
            body.push_str(&meta_line(item, payload));
            body.push_str(&lead);
        }
    }
    body.push_str(&options(item, payload));
    if !payload.is_workbook() {
        body.push_str(&answers(item, images));
    }
    body
}

fn question_lead(item: &PrintItem, images: &ImageSlots) -> String {
    let mut out = format!("#strong[{}.]", item.seq);
    let mut started = false;
    let mut first_done = false;
    const HEAD_MARK: &str = "\u{1}";
    for src in &item.question.stem {
        lead_paragraph(&mut out, &paragraph(src), images, &mut started);
        if !first_done {
            first_done = true;
            out.push_str(HEAD_MARK);
        }
    }
    if !started {
        out.push_str("\n\n");
    }
    out
}

fn lead_paragraph(out: &mut String, p: &Paragraph, images: &ImageSlots, started: &mut bool) {
    match p {
        Paragraph::Image { name } => {
            *started |= emit_lead_block(
                out,
                images.get(name).map(|(k, fmt)| format!("#fig(\"{k}\", \"{fmt}\")\n\n")),
            );
        }
        Paragraph::Quote(chunks) => {
            let inner = chunks_markup(chunks);
            *started |= emit_lead_block(out, Some(format!("#quoteblock[{inner}]\n\n")));
        }
        Paragraph::Body(chunks) => {
            let inner = chunks_markup(chunks);
            if *started {
                out.push_str(&inner);
                out.push_str("\n\n");
            } else {
                out.push(' ');
                out.push_str(&inner);
                out.push_str("\n\n");
                *started = true;
            }
        }
        other => {
            let inner = super::blocks::block_markup(other);
            if !inner.is_empty() {
                *started |= emit_lead_block(out, Some(inner));
            }
        }
    }
}

fn emit_lead_block(out: &mut String, markup: Option<String>) -> bool {
    match markup {
        Some(markup) => {
            out.push_str("\n\n");
            out.push_str(&markup);
            true
        }
        None => false,
    }
}

fn meta_line(item: &PrintItem, payload: &ExportPayload) -> String {
    if !payload.show_meta {
        return String::new();
    }
    let q = &item.question;
    let subject = if q.subject.is_empty() {
        "未知"
    } else {
        q.subject.as_str()
    };
    let meta =
        super::super::jsslots::question_meta(payload, item.seq as usize, &q.chapter, &q.locate, subject);
    let text = escape_markup(&format!("{subject} · {meta}"));
    format!(
        "#block(width: 100%, align(right, text(size: 0.71em, fill: rgb(\"#999999\"))[{text}]))\n#v(-1.5mm)\n"
    )
}

fn options(item: &PrintItem, payload: &ExportPayload) -> String {
    let opts = display_options(item);
    if opts.is_empty() {
        return String::new();
    }
    let (w, _) = super::paper_size(&payload.paper);
    let avail = w - super::measure_width(payload);
    let cells: Vec<String> = opts
        .iter()
        .map(|(letter, text)| {
            let body = one_paragraph(text);
            format!("[#strong[（{letter}）]{body}]")
        })
        .collect();
    format!(
        "#v(2.5mm)\n#fitopts({target}, {avail:.1}mm, ({}))\n\n",
        cells.join(", "),
        target = payload.opts_per_row
    )
}

fn answers(item: &PrintItem, images: &ImageSlots) -> String {
    let q = &item.question;
    let mut out = String::new();
    let single_simple = q.answer.len() == 1 && {
        let first = q.answer[0].as_str();
        !first.contains('\n')
            && !first.starts_with("$$")
            && !first.contains("![[" )
            && matches!(paragraph(first), Paragraph::Body(_))
            && (display_length(first) < 20 || math_dominant(first))
    };
    let answer_boxed = answer_letters(item).is_some() || single_simple;
    if !q.answer.is_empty() {
        if answer_boxed {
            out.push_str(&format!("#ansbox[{}]\n", answer_content(item)));
            for src in q.answer.iter().skip(1) {
                out.push_str(&stem(std::slice::from_ref(src), images));
            }
        } else {
            out.push_str(&labeled("答案", &stem(&q.answer, images)));
        }
    }
    if !q.solution.is_empty() {
        let body = stem(&q.solution, images);
        if answer_boxed {
            out.push_str(&body);
        } else {
            out.push_str(&labeled("解析", &body));
        }
    }
    if !q.notes.is_empty() {
        out.push_str(&labeled("备注", &stem(&q.notes, images)));
    }
    out
}

fn labeled(label: &str, body: &str) -> String {
    format!(
        "#block(breakable: false, width: 100%)[#anslabel[{label}]\n{}\n]\n",
        body.trim_end()
    )
}

fn math_dominant(src: &str) -> bool {
    markup::split_math(src)
        .iter()
        .filter_map(|c| match c {
            markup::Chunk::Text(t) => Some(t.trim().len()),
            _ => None,
        })
        .sum::<usize>()
        < 10
}

fn answer_content(item: &PrintItem) -> String {
    match answer_letters(item) {
        Some(letters) if letters.len() == 1 => {
            let letter = letters[0];
            let text = display_options(item)
                .into_iter()
                .find(|(l, _)| *l == letter)
                .map(|(_, text)| text)
                .unwrap_or_default();
            format!(
                "（{letter}）{}",
                one_paragraph_inner(&strip_trailing_period(&text), true)
            )
        }
        Some(letters) => letters.iter().collect::<String>(),
        None => one_paragraph_inner(
            &strip_trailing_period(item.question.answer.first().map(String::as_str).unwrap_or("")),
            true,
        ),
    }
}

fn one_paragraph_inner(src: &str, display_fractions: bool) -> String {
    let mut out = String::new();
    for chunk in markup::split_math(src) {
        match chunk {
            Chunk::Text(text) => out.push_str(&text),
            Chunk::MathInline { latex } => out.push_str(&if display_fractions {
                latex::inline_display(latex.as_str())
            } else {
                latex::inline(latex.as_str())
            }),
            Chunk::MathBlock { latex } => out.push_str(&latex::block(latex.as_str())),
        }
    }
    out
}

fn one_paragraph(src: &str) -> String {
    one_paragraph_inner(src, false)
}

fn strip_trailing_period(src: &str) -> String {
    src.trim_end()
        .trim_end_matches(['。', '.'])
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod answer_tests {
    use super::*;

    fn answer_item() -> PrintItem {
        serde_json::from_value(serde_json::json!({
            "id": "P1", "seq": 1, "source": "s", "subject": "数学", "origin": "o",
            "locate": "P1", "chapter": "c", "qtype": "选择题",
            "stem": ["题______。"], "options": [], "correct_id": null,
            "correct_ids": [], "answer": [], "solution": [], "notes": []
        }))
        .unwrap()
    }

    #[test]
    fn strip_trailing_period_edges() {
        assert_eq!(strip_trailing_period("42"), "42");
        assert_eq!(strip_trailing_period("42。。"), "42");
        assert_eq!(strip_trailing_period("42.。"), "42");
        assert_eq!(strip_trailing_period("42 "), "42");
        assert_eq!(strip_trailing_period("…"), "…");
        assert_eq!(strip_trailing_period(""), "");
        assert_eq!(strip_trailing_period("。"), "");
    }

    #[test]
    fn ansbox_covers_tall_math_structures() {
        let mut item = answer_item();
        item.question.options = vec![];
        for ans in [
            r"$f(x)=\begin{cases}1, & x>0\\ 0, & x\le 0\end{cases}$",
            r"$\begin{pmatrix}1 & 2\\ 3 & 4\end{pmatrix}$",
            r"$\sum\limits_{i=1}^{n} a_i b_i$",
        ] {
            item.question.answer = vec![ans.to_string()];
            let out = answers(&item, &ImageSlots::default());
            assert!(out.contains("#ansbox["), "math answer stays boxed: {ans}");
        }

        item.question.answer = vec!["这是一段很长的纯文本答案，超过二十个字符。".to_string()];
        let out = answers(&item, &ImageSlots::default());
        assert!(!out.contains("#ansbox["), "long prose stays outside the box");

        item.question.answer = vec![r"答案为 $\dfrac{1}{2}$，其中 $a>0$。".to_string()];
        let out = answers(&item, &ImageSlots::default());
        assert!(out.contains("#ansbox["));
    }

    #[test]
    fn multi_line_answers_get_section_labels() {
        let mut item = answer_item();
        item.question.options = vec![];
        item.question.answer = vec!["第一行。".to_string(), "第二行。".to_string()];
        item.question.solution = vec!["解析正文。".to_string()];
        let out = answers(&item, &ImageSlots::default());
        assert!(out.contains("#anslabel[答案]"), "multi-line answer labeled");
        assert!(out.contains("#anslabel[解析]"), "solution labeled alongside");
        assert!(!out.contains("#ansbox["));

        item.question.answer = vec!["42。".to_string()];
        let out = answers(&item, &ImageSlots::default());
        assert!(out.contains("#ansbox["));
        assert!(!out.contains("#anslabel[答案]"));
        assert!(!out.contains("#anslabel[解析]"));
    }
}
