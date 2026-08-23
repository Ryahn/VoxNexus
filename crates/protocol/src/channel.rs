//! Channel HTTP DTOs (F027).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;
use voxnexus_domain::ChannelType;

/// Public channel representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub community_id: Uuid,
    pub space_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    pub name: String,
    pub topic: String,
    pub position: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub archived_at: Option<DateTime<Utc>>,
    pub config: Value,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Create a channel in a community scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    #[validate(length(max = 500))]
    pub topic: Option<String>,
    pub space_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub config: Option<Value>,
}

/// Update a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub topic: Option<String>,
    pub position: Option<i32>,
    pub space_id: Option<Option<Uuid>>,
    pub category_id: Option<Option<Uuid>>,
    pub config: Option<Value>,
}

/// Channels in a list scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ChannelListResponse {
    pub channels: Vec<ChannelResponse>,
}

/// List channels query.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListChannelsQuery {
    pub space_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub include_archived: Option<bool>,
}

/// Reorder channels in a scope by id list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Validate)]
pub struct ReorderChannelsRequest {
    #[validate(length(min = 1, max = 500))]
    pub channel_ids: Vec<Uuid>,
}
