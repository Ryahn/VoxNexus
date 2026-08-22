//! Image sniffing and upload limits for profile media (F014).

/// Maximum avatar upload size (2 MiB).
pub const AVATAR_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Maximum banner upload size (5 MiB).
pub const BANNER_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Detected image kind from magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Png,
    Gif,
    Webp,
}

impl ImageKind {
    #[must_use]
    pub fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
        }
    }

    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Webp => "webp",
        }
    }
}

/// Sniff JPEG/PNG/GIF/WebP from leading bytes.
#[must_use]
pub fn sniff_image(bytes: &[u8]) -> Option<ImageKind> {
    if bytes.len() >= 3 && bytes[0] == 0xff && bytes[1] == 0xd8 && bytes[2] == 0xff {
        return Some(ImageKind::Jpeg);
    }
    if bytes.len() >= 8 && bytes[0..8] == [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a] {
        return Some(ImageKind::Png);
    }
    if bytes.len() >= 6 && (bytes[0..6] == *b"GIF87a" || bytes[0..6] == *b"GIF89a") {
        return Some(ImageKind::Gif);
    }
    if bytes.len() >= 12 && bytes[0..4] == *b"RIFF" && bytes[8..12] == *b"WEBP" {
        return Some(ImageKind::Webp);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_png_signature() {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        bytes.extend_from_slice(&[0; 16]);
        assert_eq!(sniff_image(&bytes), Some(ImageKind::Png));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(sniff_image(b""), None);
    }
}
