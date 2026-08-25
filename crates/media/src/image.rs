//! Image sniffing, attachment allow-lists, and thumbnails (F014 / F038).

use image::imageops::FilterType;
use image::ImageFormat;

/// Maximum avatar upload size (2 MiB).
pub const AVATAR_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Maximum banner upload size (5 MiB).
pub const BANNER_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Maximum message attachment size (5 MiB).
pub const ATTACHMENT_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Longest edge for generated thumbnails.
pub const THUMBNAIL_MAX_EDGE: u32 = 320;

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

/// Allowed attachment after magic-byte / extension checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedAttachment {
    pub content_type: String,
    pub kind: AttachmentKind,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Coarse attachment kind for UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentKind {
    Image,
    File,
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

/// True when bytes look like a Windows/PE or ELF executable.
#[must_use]
pub fn looks_like_executable(bytes: &[u8]) -> bool {
    if bytes.len() >= 2 && bytes[0] == 0x4d && bytes[1] == 0x5a {
        return true;
    }
    if bytes.len() >= 4 && bytes[0..4] == [0x7f, b'E', b'L', b'F'] {
        return true;
    }
    false
}

fn extension_of(filename: &str) -> Option<String> {
    let name = filename.rsplit('/').next().unwrap_or(filename);
    let name = name.rsplit('\\').next().unwrap_or(name);
    let (_, ext) = name.rsplit_once('.')?;
    if ext.is_empty() {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

fn is_blocked_extension(ext: &str) -> bool {
    matches!(
        ext,
        "exe"
            | "dll"
            | "bat"
            | "cmd"
            | "com"
            | "msi"
            | "scr"
            | "ps1"
            | "vbs"
            | "js"
            | "jse"
            | "wsf"
            | "sh"
            | "bash"
            | "zsh"
            | "app"
            | "dmg"
            | "pkg"
    )
}

/// Validate attachment bytes + filename against the allow-list.
#[must_use]
pub fn validate_attachment(bytes: &[u8], filename: &str) -> Option<AllowedAttachment> {
    if bytes.is_empty() || bytes.len() > ATTACHMENT_MAX_BYTES {
        return None;
    }
    if looks_like_executable(bytes) {
        return None;
    }
    let filename = filename.trim();
    if filename.is_empty() || filename.len() > 255 {
        return None;
    }
    if let Some(ext) = extension_of(filename) {
        if is_blocked_extension(&ext) {
            return None;
        }
    }

    if let Some(kind) = sniff_image(bytes) {
        let (width, height) = image::load_from_memory(bytes)
            .ok()
            .map(|img| (img.width(), img.height()))
            .map_or((None, None), |(w, h)| (Some(w), Some(h)));
        return Some(AllowedAttachment {
            content_type: kind.mime().to_owned(),
            kind: AttachmentKind::Image,
            width,
            height,
        });
    }

    if bytes.len() >= 5 && bytes.starts_with(b"%PDF-") {
        return Some(AllowedAttachment {
            content_type: "application/pdf".to_owned(),
            kind: AttachmentKind::File,
            width: None,
            height: None,
        });
    }

    let ext = extension_of(filename)?;
    if ext == "txt" || ext == "md" || ext == "csv" || ext == "log" {
        if bytes.iter().take(512).any(|b| *b == 0) {
            return None;
        }
        return Some(AllowedAttachment {
            content_type: "text/plain".to_owned(),
            kind: AttachmentKind::File,
            width: None,
            height: None,
        });
    }

    None
}

/// Build a JPEG thumbnail (longest edge ≤ [`THUMBNAIL_MAX_EDGE`]).
///
/// # Errors
///
/// Returns when the image cannot be decoded or encoded.
pub fn make_jpeg_thumbnail(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let thumb = img.resize(THUMBNAIL_MAX_EDGE, THUMBNAIL_MAX_EDGE, FilterType::Triangle);
    let mut out = Vec::new();
    thumb
        .write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Jpeg)
        .map_err(|error| error.to_string())?;
    Ok(out)
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

    #[test]
    fn rejects_mz_executable() {
        let bytes = b"MZ\0\0fake-pe";
        assert!(looks_like_executable(bytes));
        assert!(validate_attachment(bytes, "payload.exe").is_none());
    }

    #[test]
    fn allows_plain_text() {
        let allowed = validate_attachment(b"hello world", "notes.txt").expect("txt");
        assert_eq!(allowed.content_type, "text/plain");
        assert_eq!(allowed.kind, AttachmentKind::File);
    }
}
