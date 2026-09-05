pub fn paragraphs(lines: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for l in lines {
        let t = l.trim();
        if t.is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(t.to_string());
        }
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

pub fn nat_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let na: String = ai.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                let nb: String = bi.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                let ord = cmp_unpadded(&na, &nb);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                ai.next();
                bi.next();
                let ord = x.cmp(&y);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}

fn cmp_unpadded(a: &str, b: &str) -> std::cmp::Ordering {
    let (a, b) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn nat_cmp_orders_numbers_numerically() {
        let mut v = vec!["P10-1", "P9-9", "P15-11", "P15-2", "a1", "a10", "a2"];
        v.sort_by(|a, b| nat_cmp(a, b));
        assert_eq!(
            v,
            vec!["P9-9", "P10-1", "P15-2", "P15-11", "a1", "a2", "a10"]
        );
    }

    #[test]
    fn nat_cmp_ignores_leading_zeros_and_handles_empty() {
        assert_eq!(nat_cmp("P01-1", "P1-1"), Ordering::Equal);
        assert_eq!(nat_cmp("P001-1", "P01-2"), Ordering::Less);
        assert_eq!(nat_cmp("", ""), Ordering::Equal);
        assert_eq!(nat_cmp("", "a"), Ordering::Less);
    }

    #[test]
    fn paragraphs_split_on_blank_lines() {
        let lines = vec!["a", "b", "", "", "c"];
        assert_eq!(paragraphs(&lines), vec!["a\nb", "c"]);
    }

    #[test]
    fn paragraphs_trim_each_line() {
        let lines = vec!["  a  ", "  b", "", " c"];
        assert_eq!(paragraphs(&lines), vec!["a\nb", "c"]);
    }
}
