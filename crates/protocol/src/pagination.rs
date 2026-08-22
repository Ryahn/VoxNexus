use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default page size for cursor lists.
pub const DEFAULT_PAGE_LIMIT: u16 = 50;
/// Maximum page size accepted from clients.
pub const MAX_PAGE_LIMIT: u16 = 100;

/// Query parameters for cursor-paginated lists (`before` / `after` + `limit`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorQuery {
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub limit: Option<u16>,
}

impl CursorQuery {
    /// Limit clamped to `1..=MAX_PAGE_LIMIT`, default [`DEFAULT_PAGE_LIMIT`].
    #[must_use]
    pub fn resolved_limit(&self) -> u16 {
        self.limit
            .unwrap_or(DEFAULT_PAGE_LIMIT)
            .clamp(1, MAX_PAGE_LIMIT)
    }
}

/// Cursor page. `has_more` is true when another page exists in the query direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

impl<T> CursorPage<T> {
    #[must_use]
    pub fn new(items: Vec<T>, has_more: bool) -> Self {
        Self { items, has_more }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_defaults_and_clamps() {
        assert_eq!(CursorQuery::default().resolved_limit(), DEFAULT_PAGE_LIMIT);
        assert_eq!(
            CursorQuery {
                limit: Some(0),
                ..CursorQuery::default()
            }
            .resolved_limit(),
            1
        );
        assert_eq!(
            CursorQuery {
                limit: Some(1000),
                ..CursorQuery::default()
            }
            .resolved_limit(),
            MAX_PAGE_LIMIT
        );
    }
}
