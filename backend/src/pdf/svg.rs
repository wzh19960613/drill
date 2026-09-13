use std::fs;
use std::sync::OnceLock;

use regex::Regex;

use crate::fsutil::{find_recursive, is_safe_component};

pub fn load_image(
    name: &str,
    source_dirs: &[std::path::PathBuf],
) -> Option<(Vec<u8>, &'static str)> {
    let fmt = crate::images::image_ext(name)?;
    if !is_safe_component(name) {
        return None;
    }
    for dir in source_dirs {
        if let Some(path) = find_recursive(dir, name) {
            if let Ok(bytes) = fs::read(&path) {

                if !crate::images::magic_matches(fmt, &bytes) {
                    crate::fsutil::log_warn(&format!(
                        "[warn] image {} fails the content sniff, skipping it",
                        path.display()
                    ));
                    continue;
                }
                if fmt == "svg" {
                    let svg = strip_dark_media(&String::from_utf8_lossy(&bytes));
                    return Some((svg.into_bytes(), "svg"));
                }
                return Some((bytes, fmt));
            }
        }
    }
    None
}

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
    fn loads_raster_and_strips_svg_for_typst() {
        let dir = std::env::temp_dir().join(format!("drill-pdfimg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("nested")).unwrap();
        std::fs::write(
            dir.join("图.svg"),
            "<svg>@media (prefers-color-scheme: dark) {}</svg>",
        )
        .unwrap();
        let mut png = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
        png.extend_from_slice(b"rest");
        std::fs::write(dir.join("nested/shot.png"), &png).unwrap();

        let dirs = vec![dir.clone()];
        let (bytes, fmt) = load_image("图.svg", &dirs).unwrap();
        assert_eq!(fmt, "svg");
        assert!(!String::from_utf8_lossy(&bytes).contains("prefers-color-scheme"));

        let (bytes, fmt) = load_image("shot.png", &dirs).unwrap();
        assert_eq!(fmt, "png");
        assert_eq!(bytes, png);

        assert!(load_image("../escape.png", &dirs).is_none());
        assert!(load_image("missing.png", &dirs).is_none());
        assert!(load_image("doc.pdf", &dirs).is_none());

        std::fs::write(dir.join("bad.png"), b"not a png at all").unwrap();
        assert!(load_image("bad.png", &dirs).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
