//! LiveKit control-plane (F061) and shared media helpers.

mod image;

pub use image::{sniff_image, ImageKind, AVATAR_MAX_BYTES, BANNER_MAX_BYTES};

/// Workspace crate identity.
pub const CRATE_NAME: &str = "voxnexus-media";
