use super::*;
use super::paragraph::{image_name, paragraph, Paragraph};

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
fn paragraph_heading() {
match paragraph("### 两点说明（线性化条件）") {
    Paragraph::Heading { chunks } => {
        assert_eq!(chunks.len(), 1);
    }
    other => panic!("{other:?}"),
}

assert!(matches!(paragraph("###没有空格"), Paragraph::Body(_)));
}

#[test]
fn paragraph_list_table_code() {
match paragraph("- 甲\n- 乙\n- 丙") {
    Paragraph::List { ordered, items } => {
        assert!(!ordered);
        assert_eq!(items, vec!["甲", "乙", "丙"]);
    }
    other => panic!("{other:?}"),
}
match paragraph("1. 第一步\n2. 第二步") {
    Paragraph::List { ordered, items } => {
        assert!(ordered);
        assert_eq!(items, vec!["第一步", "第二步"]);
    }
    other => panic!("{other:?}"),
}
match paragraph("| a | b |\n|---|---|\n| 1 | 2 |") {
    Paragraph::Table { rows } => {
        assert_eq!(rows, vec![vec!["a", "b"], vec!["1", "2"]]);
    }
    other => panic!("{other:?}"),
}
match paragraph("```py\ncode\n```") {
    Paragraph::Code { text } => assert_eq!(text, "code"),
    other => panic!("{other:?}"),
}

assert!(matches!(paragraph("| a | b |\n更多"), Paragraph::Body(_)));
}

#[test]
fn list_marker_requires_following_whitespace() {

assert!(matches!(paragraph("3、给出三种动态性能指标的基本概念；"), Paragraph::Body(_)));
assert!(matches!(paragraph("2）什么是系统？"), Paragraph::Body(_)));
assert!(matches!(paragraph("1.没有空格"), Paragraph::Body(_)));

assert!(matches!(paragraph("3、 给出……"), Paragraph::List { ordered: true, .. }));
assert!(matches!(paragraph("12) 甲\n13) 乙"), Paragraph::List { ordered: true, .. }));
assert!(matches!(paragraph("-甲"), Paragraph::Body(_)));
assert!(matches!(paragraph("* 甲"), Paragraph::List { ordered: false, .. }));
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
assert_eq!(
    image_name("![[photo.png]]").as_deref(),
    Some("photo.png"),
    "raster images are embedded too"
);
assert_eq!(image_name("![[doc.pdf]]"), None, "not an image");
}
