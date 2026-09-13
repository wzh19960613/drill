pub fn image_ext(name: &str) -> Option<&'static str> {
    let ext = name.rsplit_once('.')?.1;
    Some(match ext {
        "svg" => "svg",
        "png" => "png",
        "jpg" | "jpeg" => "jpeg",
        "webp" => "webp",
        "gif" => "gif",
        _ => return None,
    })
}

pub fn mime_of(ext: &str) -> &'static str {
    match ext {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

pub fn magic_matches(ext: &str, bytes: &[u8]) -> bool {
    match ext {
        "svg" => {
            let text = String::from_utf8_lossy(bytes);
            let head = text.trim_start_matches('\u{feff}');
            head.trim_start().starts_with('<')
        }
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpeg" => bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF,
        "webp" => {
            bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
        }
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_extensions() {
        assert_eq!(image_ext("图.svg"), Some("svg"));
        assert_eq!(image_ext("a.png"), Some("png"));
        assert_eq!(image_ext("a.jpg"), Some("jpeg"));
        assert_eq!(image_ext("a.jpeg"), Some("jpeg"));
        assert_eq!(image_ext("a.webp"), Some("webp"));
        assert_eq!(image_ext("a.gif"), Some("gif"));
        assert_eq!(image_ext("a.PNG"), None, "case-sensitive");
        assert_eq!(image_ext("a.md"), None);
        assert_eq!(image_ext("noext"), None);
        assert_eq!(image_ext(".hidden"), None);
    }

    #[test]
    fn sniffs_magics() {
        assert!(magic_matches("svg", b"  <svg/>"));
        assert!(magic_matches("svg", "\u{feff}<svg/>".as_bytes()));
        assert!(!magic_matches("svg", b"not xml"));
        assert!(magic_matches("png", b"\x89PNG\r\n\x1a\n...."));
        assert!(!magic_matches("png", b"PNG"));
        assert!(magic_matches("jpeg", b"\xFF\xD8\xFF\xe0"));
        assert!(!magic_matches("jpeg", b"\xFF\xD8"));
        assert!(magic_matches("webp", b"RIFF0004WEBPVP8 "));
        assert!(!magic_matches("webp", b"RIFF0004VP8 "));
        assert!(magic_matches("gif", b"GIF89a1"));
        assert!(!magic_matches("gif", b"JIF89a1"));
    }
}
