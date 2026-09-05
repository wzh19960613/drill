#[derive(Default, Debug)]
pub struct HeaderMeta {
    pub subject: String,
    pub origin: String,
    pub chapter: String,
    pub locate: String,
    pub qtype: String,
}

pub fn parse_header(header: &str) -> HeaderMeta {
    let (subject, rest) = split_subject(header.trim());
    let [origin, chapter, locate, qtype] = split_fields(rest);
    HeaderMeta {
        subject,
        origin,
        chapter,
        locate,
        qtype,
    }
}

fn split_subject(rest: &str) -> (String, &str) {
    if !rest.starts_with('[') {
        return (String::new(), rest);
    }
    match rest.find(']') {
        Some(close) => (
            rest[1..close].trim().to_string(),
            rest[close + 1..].trim_start(),
        ),
        None => (String::new(), rest),
    }
}

const SEPS: [char; 3] = ['|', '@', '>'];

fn split_fields(rest: &str) -> [String; 4] {
    let mut fields: [String; 4] = Default::default();
    let mut field = 0;
    let mut tail = rest;
    while let Some((pos, sep_idx)) = next_sep(tail, field) {
        fields[field] = tail[..pos].trim().to_string();
        field = sep_idx + 1;
        tail = &tail[pos + SEPS[sep_idx].len_utf8()..];
    }
    fields[field] = tail.trim().to_string();
    fields
}

fn next_sep(tail: &str, from: usize) -> Option<(usize, usize)> {
    SEPS.iter()
        .enumerate()
        .skip(from)
        .filter_map(|(si, sep)| tail.find(*sep).map(|p| (p, si)))
        .min_by_key(|(p, _)| *p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_header() {
        let m = parse_header("[数学] 示例题库 · 演示 | 第1章 极限 @ P15-11 > 填空题");
        assert_eq!(m.subject, "数学");
        assert_eq!(m.origin, "示例题库 · 演示");
        assert_eq!(m.chapter, "第1章 极限");
        assert_eq!(m.locate, "P15-11");
        assert_eq!(m.qtype, "填空题");
    }

    #[test]
    fn partial_headers() {
        let m = parse_header("2026 上海高考 @ P25-12 > 选择题");
        assert_eq!(
            (m.subject.as_str(), m.origin.as_str()),
            ("", "2026 上海高考")
        );
        assert_eq!((m.chapter.as_str(), m.qtype.as_str()), ("", "选择题"));

        let m = parse_header("[数学] 某书");
        assert_eq!((m.subject.as_str(), m.origin.as_str()), ("数学", "某书"));
        assert_eq!(m.locate, "");

        let m = parse_header("| 第6章 @ P50-6");
        assert_eq!((m.origin.as_str(), m.chapter.as_str()), ("", "第6章"));
    }

    #[test]
    fn skips_earlier_separators_after_a_later_one() {
        let m = parse_header("来源 @ P1-1 | 杂项");
        assert_eq!(m.origin, "来源");
        assert_eq!(m.locate, "P1-1 | 杂项");
    }
}
