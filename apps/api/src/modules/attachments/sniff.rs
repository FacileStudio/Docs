/// The largest attachment, in bytes.
pub const MAX_BYTES: usize = 20 << 20;

/// The media type the bytes declare by their magic number, among the accepted ones. SVG and
/// everything else answer `None`: the type never comes from the request.
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

#[cfg(test)]
mod tests {
    use super::sniff;

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
