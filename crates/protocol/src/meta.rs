use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Build and version of this instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MetaResponse {
    pub name: String,
    pub version: String,
}
