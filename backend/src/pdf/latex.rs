use crate::fsutil::log_warn;

pub fn normalize(src: &str) -> String {
    normalize_inner(src, false)
}

pub fn normalize_display(src: &str) -> String {
    normalize_inner(src, true)
}

fn normalize_inner(src: &str, display_fractions: bool) -> String {
    let collapsed = strip_row_spacing(src);
    let without_display = collapsed.replace(r"\displaystyle", "");
    let dfrac = if display_fractions {
        r"\displaystyle\frac"
    } else {
        r"\frac"
    };
    without_display
        .replace(r"\dfrac", dfrac)
        .replace(r"\tfrac", r"\frac")
        .replace(r"\dbinom", r"\binom")
}

fn strip_row_spacing(src: &str) -> String {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\\\\\[[^\]]*\]").unwrap());
    re.replace_all(src, r"\\").into_owned()
}

fn escape_typst_string(src: &str) -> String {
    src.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn inline(latex: &str) -> String {
    inline_inner(latex, false)
}

pub fn inline_display(latex: &str) -> String {
    inline_inner(latex, true)
}

fn inline_inner(latex: &str, display_fractions: bool) -> String {
    let normalized = if display_fractions {
        normalize_display(latex)
    } else {
        normalize(latex)
    };
    match mitex::convert_math(&normalized, None) {
        Ok(converted) => format!("#mi(\"{}\")", escape_typst_string(&converted)),
        Err(_) => raw_fallback(latex),
    }
}

pub fn block(latex: &str) -> String {
    match mitex::convert_math(&normalize(latex), None) {
        Ok(converted) => format!("#mm(\"{}\")", escape_typst_string(&converted)),
        Err(_) => raw_fallback(latex),
    }
}

fn raw_fallback(latex: &str) -> String {
    log_warn(&format!(
        "[warn] math conversion failed, falling back to raw text: {latex}"
    ));
    format!("`{}`", latex.replace('`', "'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_katex_macros() {
        assert_eq!(normalize(r"\dfrac{1}{2}"), r"\frac{1}{2}");
        assert_eq!(
            normalize_display(r"\dfrac{1}{2}"),
            r"\displaystyle\frac{1}{2}"
        );
        assert_eq!(normalize(r"\tfrac{1}{2}"), r"\frac{1}{2}");
        assert_eq!(normalize(r"\dbinom{n}{k}"), r"\binom{n}{k}");
        assert_eq!(normalize(r"\displaystyle x"), " x");
    }

    #[test]
    fn strips_row_spacing() {
        assert_eq!(normalize(r"a\\[4pt] b"), r"a\\ b");
        assert_eq!(normalize(r"a\\ b"), r"a\\ b");
    }

    #[test]
    fn converts_cases_env() {
        let out = inline(r"f(x)=\begin{cases}x>0\\ x\leqslant 0\end{cases}");
        assert!(out.starts_with("#mi(\""));
        assert!(out.contains("cases"));
        assert!(out.ends_with("\")"));
    }
}
