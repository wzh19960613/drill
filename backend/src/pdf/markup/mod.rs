mod paragraph;
#[cfg(test)]
mod tests;

pub use paragraph::{paragraph, Paragraph};

#[derive(Debug)]
pub enum Chunk {
    Text(String),
    MathInline { latex: String },
    MathBlock { latex: String },
}

pub fn split_math(src: &str) -> Vec<Chunk> {
    let mut maths: Vec<(bool, String)> = Vec::new();
    let mut text = String::new();
    let mut rest = src;
    while let Some(start) = rest.find("$$") {
        text.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        match rest.find("$$").filter(|&end| end > 0) {
            Some(end) => {
                maths.push((true, rest[..end].to_string()));
                text.push_str(&placeholder(maths.len() - 1));
                rest = &rest[end + 2..];
            }
            None => {
                text.push_str("$$");
                break;
            }
        }
    }
    text.push_str(rest);
    let text = split_inline_math(&text, &mut maths);
    restore(text, &maths)
}

fn placeholder(index: usize) -> String {
    format!("\u{0}{index}\u{0}")
}

fn split_inline_math(src: &str, maths: &mut Vec<(bool, String)>) -> String {
    let mut out = String::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            if let Some(next) = src[i + 1..].find(['$', '\n']).filter(|n| *n > 0) {
                let content_end = i + 1 + next;
                if bytes[content_end] == b'$' {
                    maths.push((false, src[i + 1..content_end].to_string()));
                    out.push_str(&placeholder(maths.len() - 1));
                    i = content_end + 1;
                    continue;
                }
            }
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn restore(text: String, maths: &[(bool, String)]) -> Vec<Chunk> {
    let styled = styled_text(&text);
    let mut chunks = Vec::new();
    let mut plain = String::new();
    let mut chars = styled.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{0}' {
            let index: String = chars.by_ref().take_while(|c| *c != '\u{0}').collect();
            let index: usize = index.parse().unwrap_or(usize::MAX);
            if !plain.is_empty() {
                chunks.push(Chunk::Text(std::mem::take(&mut plain)));
            }
            match maths.get(index) {
                Some((true, latex)) => chunks.push(Chunk::MathBlock {
                    latex: latex.clone(),
                }),
                Some((false, latex)) => chunks.push(Chunk::MathInline {
                    latex: latex.clone(),
                }),
                None => {}
            }
        } else {
            plain.push(ch);
        }
    }
    if !plain.is_empty() {
        chunks.push(Chunk::Text(plain));
    }
    chunks
}

fn styled_text(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(pair) = find_bold(rest) {
        let (before, inner, after) = pair;
        out.push_str(&escape_markup(before));
        out.push_str(&format!("*{}*", escape_markup(inner)));
        rest = after;
    }
    out.push_str(&escape_markup(rest));
    out.replace('\n', "#linebreak()")
}

fn find_bold(src: &str) -> Option<(&str, &str, &str)> {
    let start = src.find("**")?;
    let rest = &src[start + 2..];
    let end = rest.find("**")?;
    Some((&src[..start], &rest[..end], &rest[end + 2..]))
}

pub fn escape_markup(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for ch in src.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '#' | '$' | '*' | '_' | '`' | '[' | ']' | '<' | '>' | '@' | '~' | '\'' | '"' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}
