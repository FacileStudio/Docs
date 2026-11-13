/// The largest image or PDF, in bytes: the types served inline.
pub const MAX_BYTES: usize = 20 << 20;

/// The largest file of any other type, in bytes, unless `ATTACHMENT_MAX_BYTES` says otherwise.
pub const FILE_MAX_BYTES: usize = 25 << 20;

const FALLBACK: &str = "application/octet-stream";

/// The media type the bytes declare by their magic number, among the types shown inline. SVG
/// and everything else answer `None`: they are stored as downloads typed by [`declared`].
pub fn sniff(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some("image/png"),
        [0xff, 0xd8, 0xff, ..] => Some("image/jpeg"),
        [b'G', b'I', b'F', b'8', b'7' | b'9', b'a', ..] => Some("image/gif"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("image/webp"),
        [b'%', b'P', b'D', b'F', b'-', ..] => Some("application/pdf"),
        _ => None,
    }
}

/// Whether a stored type is one [`sniff`] answers, so its bytes were checked and it may be
/// shown inline.
pub fn inline(mime: &str) -> bool {
    ["image/png", "image/jpeg", "image/gif", "image/webp", "application/pdf"].contains(&mime)
}

fn active(subtype: &str) -> bool {
    subtype.ends_with("+xml")
        || subtype.contains("svg")
        || ["html", "xhtml", "xml", "javascript", "ecmascript", "x-javascript"].contains(&subtype)
}

/// The type a file whose bytes [`sniff`] does not know is stored under: the type the upload
/// declared, lowercased and without parameters, or `application/octet-stream` when it is
/// malformed, could run as markup or script (HTML, XML, SVG, JavaScript), or claims an inline
/// type its bytes do not have.
pub fn declared(raw: Option<&str>) -> String {
    let essence = raw.unwrap_or_default().split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
    let token = |part: &str| {
        !part.is_empty()
            && part.len() <= 100
            && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    };
    match essence.split_once('/') {
        Some((kind, subtype)) if token(kind) && token(subtype) && !active(subtype) && !inline(&essence) => essence,
        _ => FALLBACK.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{declared, sniff};

    #[test]
    fn declared_types_are_kept_unless_they_could_run_or_lie() {
        assert_eq!(declared(Some("Application/ZIP; charset=x")), "application/zip");
        assert_eq!(declared(Some("text/plain")), "text/plain");
        let docx = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
        assert_eq!(declared(Some(docx)), docx);
        for risky in ["text/html", "image/svg+xml", "application/xhtml+xml", "text/xml", "text/javascript"] {
            assert_eq!(declared(Some(risky)), "application/octet-stream", "{risky}");
        }
        for bad in [None, Some(""), Some("zip"), Some("a/b c"), Some("image/png"), Some("a/\"\r\n")] {
            assert_eq!(declared(bad), "application/octet-stream", "{bad:?}");
        }
    }

    #[test]
    fn only_the_five_accepted_types_are_recognised() {
        assert_eq!(sniff(b"\x89PNG\r\n\x1a\nrest"), Some("image/png"));
        assert_eq!(sniff(b"\xff\xd8\xff\xe0"), Some("image/jpeg"));
        assert_eq!(sniff(b"GIF89a.."), Some("image/gif"));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff(b"%PDF-1.7"), Some("application/pdf"));
        assert_eq!(sniff(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"), None);
        assert_eq!(sniff(b"<?xml version=\"1.0\"?><svg/>"), None);
        assert_eq!(sniff(b"\x89PN"), None);
    }
}
