use super::Chunk;

#[derive(Debug)]
pub enum Paragraph {
    Image { name: String },
    Heading { chunks: Vec<Chunk> },
    Quote(Vec<Chunk>),
    Body(Vec<Chunk>),
    List {
        ordered: bool,
        items: Vec<String>,
    },
    Table { rows: Vec<Vec<String>> },
    Code { text: String },
}

pub fn paragraph(src: &str) -> Paragraph {
    let trimmed = src.trim();
    if let Some(name) = image_name(trimmed) {
        return Paragraph::Image { name };
    }
    if let Some(rest) = heading_body(trimmed) {
        return Paragraph::Heading {
            chunks: super::split_math(rest),
        };
    }
    if trimmed.starts_with("```") {
        return code_paragraph(trimmed);
    }
    if let Some(rows) = table_rows(trimmed) {
        return Paragraph::Table { rows };
    }
    if let Some((ordered, items)) = list_items(trimmed) {
        return Paragraph::List { ordered, items };
    }
    if trimmed.starts_with('>') {
        let body = trimmed
            .split('\n')
            .map(|l| {
                l.trim_start()
                    .trim_start_matches('>')
                    .trim_start()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Paragraph::Quote(super::split_math(&body));
    }
    Paragraph::Body(super::split_math(src))
}

fn code_paragraph(trimmed: &str) -> Paragraph {
    let inner = trimmed
        .split('\n')
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let inner = inner.trim_end_matches("```").trim_end();
    Paragraph::Code {
        text: inner.to_string(),
    }
}

fn heading_body(src: &str) -> Option<&str> {
    let t = src.trim();
    let hashes = t.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&hashes) && t[hashes..].starts_with(' ') {
        return Some(t[hashes..].trim());
    }
    None
}

fn split_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

fn is_sep_row(line: &str) -> bool {
    let t = line.trim();
    t.contains('-')
        && t.contains('|')
        && t.chars().all(|c| matches!(c, '-' | ':' | '|' | ' ' | '\t'))
}

fn table_rows(src: &str) -> Option<Vec<Vec<String>>> {
    let lines: Vec<&str> = src.split('\n').map(str::trim).collect();
    if lines.len() < 2 || !lines[0].contains('|') || !is_sep_row(lines[1]) {
        return None;
    }
    let mut rows = vec![split_row(lines[0])];
    for line in lines.iter().skip(2) {
        if line.is_empty() {
            return None;
        }
        rows.push(split_row(line));
    }
    Some(rows)
}

fn strip_list_marker(line: &str) -> Option<(bool, &str)> {
    let t = line.trim_start();
    let bullets = ["-", "*", "+"].iter().find_map(|b| t.strip_prefix(b));
    if let Some(rest) = bullets {
        return rest
            .starts_with(char::is_whitespace)
            .then(|| (false, rest.trim_start()));
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        let rest = &t[digits..];
        for sep in [".", ")", "、"] {
            if let Some(r) = rest.strip_prefix(sep) {
                if r.starts_with(char::is_whitespace) {
                    return Some((true, r.trim_start()));
                }
            }
        }
    }
    None
}

fn list_items(src: &str) -> Option<(bool, Vec<String>)> {
    let lines: Vec<&str> = src.split('\n').collect();
    let (ordered, _) = strip_list_marker(lines.first()?)?;
    let mut items: Vec<String> = Vec::new();
    for line in lines {
        match strip_list_marker(line) {
            Some((_, rest)) => items.push(rest.to_string()),
            None => {
                let last = items.last_mut()?;
                last.push('\n');
                last.push_str(line.trim());
            }
        }
    }
    (!items.is_empty()).then_some((ordered, items))
}

pub(super) fn image_name(src: &str) -> Option<String> {
    const OPEN: &str = "![[";
    let rest = src.strip_prefix(OPEN)?;
    let end = rest.find("]]")?;
    let name = &rest[..end];
    (crate::images::image_ext(name).is_some()).then(|| name.to_string())
}
