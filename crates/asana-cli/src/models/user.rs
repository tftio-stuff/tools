//! User-centric data structures consumed by the CLI.

use super::{project::MemberPermission, workspace::WorkspaceReference};
use serde::{Deserialize, Serialize};

/// Lightweight user reference returned by Asana APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Human friendly display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type reported by Asana.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Optional primary email address when included in the response.
    #[serde(default)]
    pub email: Option<String>,
}

impl UserReference {
    /// Convenience helper displaying either the name or email.
    #[must_use]
    pub fn label(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| self.gid.clone())
    }
}

/// Identity payload used when specifying members in API requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UserIdentity {
    /// Accept both email addresses and gids per Asana API contract.
    pub resource: String,
    /// Optional role/permission hint.
    #[serde(default)]
    pub role: Option<MemberPermission>,
}

/// User photo URLs at different sizes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::struct_field_names)]
pub struct UserPhoto {
    /// 21x21 pixel image URL.
    #[serde(rename = "image_21x21")]
    #[serde(default)]
    pub image_21x21: Option<String>,
    /// 27x27 pixel image URL.
    #[serde(rename = "image_27x27")]
    #[serde(default)]
    pub image_27x27: Option<String>,
    /// 36x36 pixel image URL.
    #[serde(rename = "image_36x36")]
    #[serde(default)]
    pub image_36x36: Option<String>,
    /// 60x60 pixel image URL.
    #[serde(rename = "image_60x60")]
    #[serde(default)]
    pub image_60x60: Option<String>,
    /// 128x128 pixel image URL.
    #[serde(rename = "image_128x128")]
    #[serde(default)]
    pub image_128x128: Option<String>,
}

/// Full user payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct User {
    /// Globally unique identifier.
    pub gid: String,
    /// User's name.
    pub name: String,
    /// Email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Photo URLs.
    #[serde(default)]
    pub photo: Option<UserPhoto>,
    /// Workspaces user is member of.
    #[serde(default)]
    pub workspaces: Vec<WorkspaceReference>,
}

/// Parameters for listing users in a workspace.
#[derive(Debug, Clone)]
pub struct UserListParams {
    /// Workspace identifier.
    pub workspace_gid: String,
    /// Maximum number to fetch.
    pub limit: Option<usize>,
}
