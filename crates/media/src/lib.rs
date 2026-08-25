//! LiveKit control-plane (F061) and shared media helpers.

mod image;

pub use image::{
    looks_like_executable, make_jpeg_thumbnail, sniff_image, validate_attachment,
    AllowedAttachment, AttachmentKind, ImageKind, ATTACHMENT_MAX_BYTES, AVATAR_MAX_BYTES,
    BANNER_MAX_BYTES, THUMBNAIL_MAX_EDGE,
};

/// Workspace crate identity.
pub const CRATE_NAME: &str = "voxnexus-media";
