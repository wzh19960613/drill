use super::super::latex;
use super::super::markup::{self, Chunk, Paragraph};
use super::ImageSlots;
use crate::fsutil::log_warn;

pub(crate) fn stem(paragraphs: &[String], images: &ImageSlots) -> String {
    let mut out = String::new();
    let mut pending_heading: Option<String> = None;
    for src in paragraphs {
        let classified = markup::paragraph(src);
        let rendered = paragraph_markup(&classified, images);
        if rendered.is_empty() {
            continue;
        }
        if is_lead(&classified, src) {
            if let Some(h) = pending_heading.take() {
                emit(&mut out, h);
            }
            pending_heading = Some(rendered);
            continue;
        }
        match pending_heading.take() {
            Some(h) => emit(&mut out, format!("{h}\n{rendered}")),
            None => emit(&mut out, rendered),
        }
    }
    if let Some(h) = pending_heading {
        emit(&mut out, h);
    }
    out
}

fn emit(out: &mut String, body: String) {
    out.push_str(&format!(
        "#block(breakable: false, width: 100%)[{}\n]\n\n",
        body
    ));
}

fn is_lead(classified: &Paragraph, src: &str) -> bool {
    matches!(classified, Paragraph::Heading { .. })
        || matches!(&classified, Paragraph::Body(_)
            if src.trim().starts_with("**") && src.trim().ends_with("**"))
}

fn paragraph_markup(p: &Paragraph, images: &ImageSlots) -> String {
    match p {
        Paragraph::Image { name } => match images.get(name) {
            Some((key, fmt)) => format!("#fig(\"{key}\", \"{fmt}\")"),
            None => {
                log_warn(&format!("[warn] export svg not found in sources: {name}"));
                String::new()
            }
        },
        Paragraph::Quote(chunks) => format!("#quoteblock[{}]", chunks_markup(chunks)),
        Paragraph::Body(chunks) => chunks_markup(chunks),
        other => block_markup(other).trim_end().to_string(),
    }
}

pub(crate) fn block_markup(p: &Paragraph) -> String {
    match p {
        Paragraph::List { ordered, items } => list_markup(*ordered, items),
        Paragraph::Table { rows } => table_markup(rows),
        Paragraph::Code { text } => code_markup(text),
        Paragraph::Heading { chunks } => format!(
            "#v(1mm)\n#text(size: 1.05em, weight: \"bold\")[{}]\n#v(0.5mm)\n\n",
            chunks_markup(chunks)
        ),
        _ => String::new(),
    }
}

fn list_markup(ordered: bool, items: &[String]) -> String {
    let mut out = String::new();
    for item in items {
        let body = chunks_markup(&markup::split_math(item));
        out.push_str(if ordered { "+ " } else { "- " });
        out.push_str(&body);
        out.push('\n');
    }
    out.push('\n');
    out
}

fn table_markup(rows: &[Vec<String>]) -> String {
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let cells: Vec<String> = rows
        .iter()
        .flat_map(|r| {
            (0..cols).map(|i| {
                let cell = r.get(i).map(String::as_str).unwrap_or("");
                format!("[{}]", chunks_markup(&markup::split_math(cell)))
            })
        })
        .collect();
    format!(
        "#table(columns: {cols}, inset: 4pt, align: center + horizon, stroke: 0.4pt + rgb(\"#9a9a9a\"), {})\n\n",
        cells.join(", ")
    )
}

fn code_markup(text: &str) -> String {
    let mut longest = 0usize;
    for l in text.lines() {
        let run = l.trim_start().chars().take_while(|c| *c == '`').count();
        longest = longest.max(run);
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}\n{text}\n{fence}\n\n")
}

pub(crate) fn chunks_markup(chunks: &[Chunk]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < chunks.len() {
        match &chunks[i] {
            Chunk::MathBlock { .. } => {
                let (run, next) = math_run(chunks, i);
                i = next;
                out.push('\n');
                if run.len() == 1 {
                    out.push_str(&latex::block(run[0].as_str()));
                } else {
                    out.push_str(&latex::blocks(&run));
                }
                out.push('\n');
            }
            Chunk::Text(text) => {
                out.push_str(text);
                i += 1;
            }
            Chunk::MathInline { latex } => {
                out.push_str(&latex::inline(latex.as_str()));
                i += 1;
            }
        }
    }
    out
}

fn math_run(chunks: &[Chunk], start: usize) -> (Vec<String>, usize) {
    let mut run: Vec<String> = Vec::new();
    let mut j = start;
    while j < chunks.len() {
        match &chunks[j] {
            Chunk::MathBlock { latex } => {
                run.push(latex.clone());
                j += 1;
            }
            Chunk::Text(t) if t.trim().replace("#linebreak()", "").trim().is_empty() => j += 1,
            _ => break,
        }
    }
    (run, j)
}

pub(crate) fn display_length(src: &str) -> usize {
    use regex::Regex;
    static RE: std::sync::OnceLock<(Regex, Regex, Regex, Regex, Regex)> = std::sync::OnceLock::new();
    let (img, dmath, imath, bold, code) = RE.get_or_init(|| {
        (
            Regex::new(r"!\[\[[^\]]*\]\]").unwrap(),
            Regex::new(r"\$\$([\s\S]+?)\$\$").unwrap(),
            Regex::new(r"\$([^$\n]+?)\$").unwrap(),
            Regex::new(r"\*\*").unwrap(),
            Regex::new(r"`([^`]*)`").unwrap(),
        )
    });
    let s = img.replace_all(src, "");
    let s = dmath.replace_all(&s, |c: &regex::Captures| visible_math(&c[1]));
    let s = imath.replace_all(&s, |c: &regex::Captures| visible_math(&c[1]));
    let s = bold.replace_all(&s, "");
    let s = code.replace_all(&s, "$1");
    s.chars().count()
}

fn visible_math(tex: &str) -> String {
    use regex::Regex;
    static RE: std::sync::OnceLock<(Regex, Regex)> = std::sync::OnceLock::new();
    let (cmd, struct_) = RE.get_or_init(|| {
        (
            Regex::new(r"\\[a-zA-Z]+").unwrap(),
            Regex::new(r"[{}^_]").unwrap(),
        )
    });
    let s = cmd.replace_all(tex, "");
    struct_.replace_all(&s, "").into_owned()
}
