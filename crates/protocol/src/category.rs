//! Channel category HTTP DTOs (F026).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

/// Public category representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub space_id: Option<Uuid>,
    pub name: String,
    pub position: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Create a category in a community (optionally scoped to a Space).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub space_id: Option<Uuid>,
}

/// Update a category (name, position, or scope).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub position: Option<i32>,
    pub space_id: Option<Option<Uuid>>,
}

/// Categories in a community scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CategoryListResponse {
    pub categories: Vec<CategoryResponse>,
}

/// List categories query (`space_id` omitted = community-level).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListCategoriesQuery {
    pub space_id: Option<Uuid>,
}

/// Reorder categories by sending new positions in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct ReorderCategoriesRequest {
    #[validate(length(min = 1, max = 200))]
    pub category_ids: Vec<Uuid>,
}
