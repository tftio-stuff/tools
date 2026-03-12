//! Workspace and team references.

use serde::{Deserialize, Serialize};

/// Lightweight workspace reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WorkspaceReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Optional display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Optional resource type marker from Asana.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl WorkspaceReference {
    /// Display helper returning a human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Full workspace payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Workspace {
    /// Globally unique identifier.
    pub gid: String,
    /// Workspace name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Email domains for organization (if applicable).
    #[serde(default)]
    pub email_domains: Vec<String>,
    /// Whether workspace is an organization.
    #[serde(default)]
    pub is_organization: bool,
}

/// Parameters for listing workspaces.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceListParams {
    /// Maximum number to fetch.
    pub limit: Option<usize>,
}
