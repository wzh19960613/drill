use std::fs;
use std::sync::OnceLock;

use regex::Regex;

use crate::fsutil::is_safe_component;

pub fn load_svg(name: &str, source_dirs: &[std::path::PathBuf]) -> Option<String> {
    if !is_safe_asset_name(name) {
        return None;
    }
    for dir in source_dirs {
        if let Ok(svg) = fs::read_to_string(dir.join(name)) {
            return Some(strip_dark_media(&svg));
        }
    }
    None
}

/// Strip dark-mode media queries: the print output is always light, so dark
/// styles would otherwise invert the printed strokes.
pub fn strip_dark_media(svg: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(
            r"(?s)@media\s*\(\s*prefers-color-scheme\s*:\s*dark\s*\)\s*\{(?:[^{}]|\{[^{}]*\})*\}",
        )
        .unwrap()
    });
    re.replace_all(svg, "").into_owned()
}

pub fn is_safe_asset_name(name: &str) -> bool {
    is_safe_component(name) && name.ends_with(".svg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_dark_media_query() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 430 300">
  <style>
    @media (prefers-color-scheme: dark) {
      .fg { fill: #ffffff; stroke: #ffffff; }
    }
  </style>
  <line class="fg" x1="20" y1="160" x2="408" y2="160" stroke="black"/>
</svg>"#;
        let out = strip_dark_media(svg);
        assert!(
            !out.contains("prefers-color-scheme"),
            "dark query must be stripped: {out}"
        );
        assert!(out.contains("<line"));
    }

    #[test]
    fn rejects_unsafe_names() {
        assert!(is_safe_asset_name("P40-5图.svg"));
        assert!(!is_safe_asset_name("../secret.svg"));
        assert!(!is_safe_asset_name("a/b.svg"));
        assert!(!is_safe_asset_name(".hidden.svg"), "dotfiles rejected");
        assert!(!is_safe_asset_name("photo.png"));
        assert!(!is_safe_asset_name(""));
    }
}
