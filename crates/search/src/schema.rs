//! Versioned Typesense collection schemas (populated by F057 indexers).

use serde::Serialize;

/// Bump when collection field sets change incompatibly (operators reindex).
pub const SCHEMA_VERSION: u32 = 1;

pub const COLLECTION_MESSAGES: &str = "messages";
pub const COLLECTION_USERS: &str = "users";
pub const COLLECTION_CHANNELS: &str = "channels";

/// Typesense field definition.
#[derive(Debug, Clone, Serialize)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
}

/// Collection create body.
#[derive(Debug, Clone, Serialize)]
pub struct CollectionSchema {
    pub name: String,
    pub fields: Vec<FieldSchema>,
}

fn field(name: &str, field_type: &str, facet: bool, optional: bool) -> FieldSchema {
    FieldSchema {
        name: name.to_owned(),
        field_type: field_type.to_owned(),
        facet: facet.then_some(true),
        optional: optional.then_some(true),
    }
}

/// Messages collection — text body + filterable ids (empty until F057).
#[must_use]
pub fn messages_schema() -> CollectionSchema {
    CollectionSchema {
        name: COLLECTION_MESSAGES.to_owned(),
        fields: vec![
            field("community_id", "string", true, false),
            field("channel_id", "string", true, false),
            field("author_id", "string", true, false),
            field("body", "string", false, false),
            field("created_at", "int64", false, false),
            field("schema_version", "int32", true, false),
        ],
    }
}

/// Users collection — identity fields for directory search.
#[must_use]
pub fn users_schema() -> CollectionSchema {
    CollectionSchema {
        name: COLLECTION_USERS.to_owned(),
        fields: vec![
            field("username", "string", false, false),
            field("display_name", "string", false, true),
            field("schema_version", "int32", true, false),
        ],
    }
}

/// Channels collection — name within a community.
#[must_use]
pub fn channels_schema() -> CollectionSchema {
    CollectionSchema {
        name: COLLECTION_CHANNELS.to_owned(),
        fields: vec![
            field("community_id", "string", true, false),
            field("name", "string", false, false),
            field("schema_version", "int32", true, false),
        ],
    }
}

/// All collections ensured at startup.
#[must_use]
pub fn all_collection_schemas() -> Vec<CollectionSchema> {
    vec![messages_schema(), users_schema(), channels_schema()]
}
