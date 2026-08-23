use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use voxnexus_domain::{CommunityCreationMode, RegistrationMode};

/// Build, version, and public instance policy flags for the SPA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MetaResponse {
    pub name: String,
    pub version: String,
    pub registration_mode: RegistrationMode,
    pub community_creation_mode: CommunityCreationMode,
}
