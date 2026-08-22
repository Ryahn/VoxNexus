//! HTTP DTOs shared by the server and generated clients.

mod error;
mod meta;
mod pagination;

pub use error::{error_codes, ErrorBody};
pub use meta::MetaResponse;
pub use pagination::{CursorPage, CursorQuery, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};
