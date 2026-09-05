//! Mirrors the frontend `math.ts` pipeline: pull out `$$...$$` / `$...$` math

pub enum Chunk {
    Text(String),
    MathInline { latex: String },
    MathBlock { latex: String },
}

/// Split a paragraph into text / math chunks (frontend `richTextBase`).
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

pub enum Paragraph {
    Image { name: String },
    Quote(Vec<Chunk>),
    Body(Vec<Chunk>),
}

/// Classify one paragraph (frontend `paraHtml`).
pub fn paragraph(src: &str) -> Paragraph {
    let trimmed = src.trim();
    if let Some(name) = image_name(trimmed) {
        return Paragraph::Image { name };
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
        return Paragraph::Quote(split_math(&body));
    }
    Paragraph::Body(split_math(src))
}

fn image_name(src: &str) -> Option<String> {
    const OPEN: &str = "![[";

    let rest = src.strip_prefix(OPEN)?;
    let end = rest.find("]]")?;
    let name = &rest[..end];
    name.ends_with(".svg").then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(chunks: &[Chunk]) -> Vec<&str> {
        chunks
            .iter()
            .filter_map(|c| match c {
                Chunk::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn splits_inline_and_display_math() {
        let chunks = split_math("a $x^2$ b $$y_1$$ c");
        assert_eq!(chunks.len(), 5);
        assert!(matches!(&chunks[1], Chunk::MathInline { latex } if latex == "x^2"));
        assert!(matches!(&chunks[3], Chunk::MathBlock { latex } if latex == "y_1"));
    }

    #[test]
    fn escapes_and_bolds_text() {
        let chunks = split_math("看 **函数 & 极限** 与 50% 涨幅");
        let t = texts(&chunks).join("");
        assert!(t.contains("*函数 & 极限*"), "bold: {t}");
        assert!(t.contains(r"50\%") || t.contains("50%"), "{t}");
    }

    #[test]
    fn paragraph_kinds() {
        assert!(matches!(
            paragraph("![[P40-5图.svg]]"),
            Paragraph::Image { name } if name == "P40-5图.svg"
        ));
        assert!(matches!(paragraph("> 提示"), Paragraph::Quote(_)));
        assert!(matches!(paragraph("普通段落"), Paragraph::Body(_)));
    }

    #[test]
    fn unmatched_dollar_is_literal() {
        let chunks = split_math("价格 $5 与\n6");
        assert_eq!(chunks.len(), 1, "no math should be detected");
    }

    #[test]
    fn unclosed_bold_stays_literal() {
        let chunks = split_math("只有 ** 一个星对");
        assert!(texts(&chunks).join("").contains("\\*\\*"));
    }

    #[test]
    fn escapes_markup_significant_chars() {
        assert_eq!(
            escape_markup(r#"\#$*_`[]<>@~'"#),
            r"\\ \# \$ \* \_ \` \[ \] \< \> \@ \~ \'".replace(' ', "")
        );
        assert_eq!(escape_markup("中文（）；"), "中文（）；");
    }

    #[test]
    fn quote_strips_marker_on_every_line() {
        let chunks = match paragraph("> 一行\n> 二行") {
            Paragraph::Quote(c) => c,
            _ => panic!("expected quote"),
        };
        assert_eq!(texts(&chunks).join(""), "一行#linebreak()二行");
    }

    #[test]
    fn unmatched_display_opener_is_literal() {
        let chunks = split_math("a $$b");
        assert!(texts(&chunks).join("").contains("\\$\\$"));
    }

    #[test]
    fn two_dollars_around_text_still_form_math() {
        // "$5 与 $6" parses "5 与 " as inline math: the first `$` pairs with
        // the next `$` on the same line — locked in as the intended behavior
        let chunks = split_math("价格 $5 与 $6 之间");
        assert_eq!(chunks.len(), 3);
        assert!(matches!(&chunks[0], Chunk::Text(t) if t == "价格 "));
        assert!(matches!(&chunks[1], Chunk::MathInline { latex } if latex.trim() == "5 与"));
        assert!(matches!(&chunks[2], Chunk::Text(t) if t.starts_with("6")));
    }

    #[test]
    fn image_name_edges() {
        assert!(image_name("![[a.svg]]").as_deref() == Some("a.svg"));
        assert_eq!(
            image_name("![[P40-5图.svg]]").as_deref(),
            Some("P40-5图.svg")
        );
        assert_eq!(
            image_name("![[X.SVG]]"),
            None,
            "extension is case-sensitive"
        );
        assert_eq!(image_name("![[unclosed.svg"), None, "missing ]]");
        assert_eq!(image_name("![[ ]]"), None, "blank name");
        assert_eq!(image_name("plain text"), None);
        assert_eq!(image_name("![[photo.png]]"), None, "only svg is an image");
    }
}
