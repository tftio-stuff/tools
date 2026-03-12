//! Project domain models and request payload helpers.

use super::{user::UserReference, workspace::WorkspaceReference};
use clap::ValueEnum;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Permission levels exposed for project members.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum MemberPermission {
    /// Full access.
    Member,
    /// Read/write limited to comments.
    Commenter,
    /// View only.
    Viewer,
}

/// Full project payload returned from Asana.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Project {
    /// Project identifier.
    pub gid: String,
    /// Display name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Optional description/notes.
    #[serde(default)]
    pub notes: Option<String>,
    /// Color slug when assigned.
    #[serde(default)]
    pub color: Option<String>,
    /// Whether the project has been archived.
    #[serde(default)]
    pub archived: bool,
    /// Visibility flag (public/private).
    #[serde(default)]
    pub public: Option<bool>,
    /// ISO 8601 due date.
    #[serde(default)]
    pub due_on: Option<String>,
    /// ISO 8601 start date.
    #[serde(default)]
    pub start_on: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last modification timestamp.
    #[serde(default)]
    pub modified_at: Option<String>,
    /// Owning workspace.
    #[serde(default)]
    pub workspace: Option<WorkspaceReference>,
    /// Owning team (Asana represents teams as workspaces with `resource_type` `team`).
    #[serde(default)]
    pub team: Option<WorkspaceReference>,
    /// Project owner.
    #[serde(default)]
    pub owner: Option<UserReference>,
    /// Members when requested via `opt_fields`.
    #[serde(default)]
    pub members: Vec<ProjectMember>,
    /// Recent status updates when requested separately.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<ProjectStatus>,
    /// Arbitrary custom fields.
    #[serde(default)]
    pub custom_fields: BTreeMap<String, serde_json::Value>,
}

impl Project {
    /// Determine whether the project matches a set of filters.
    #[must_use]
    pub fn matches(&self, filters: &[ProjectFilter]) -> bool {
        filters.iter().all(|filter| filter.matches(self))
    }
}

/// Response payload for project members endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectMembers {
    /// Project identifier.
    pub project_gid: String,
    /// Member collection.
    pub members: Vec<ProjectMember>,
}

/// Simplified project member representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectMember {
    /// Member identifier.
    pub gid: String,
    /// Linked user reference.
    pub user: UserReference,
    /// Optional role (commenter/viewer/member).
    #[serde(default)]
    pub role: Option<MemberPermission>,
}

/// Summary of a project status update.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ProjectStatus {
    /// Status identifier.
    pub gid: String,
    /// Status headline.
    #[serde(default)]
    pub title: Option<String>,
    /// Status color (green, yellow, red).
    #[serde(default)]
    pub color: Option<String>,
    /// Rich text body when provided.
    #[serde(default)]
    pub text: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Author metadata.
    #[serde(default)]
    pub created_by: Option<UserReference>,
}

/// Parameters accepted by the `/projects` listing endpoint.
#[derive(Debug, Clone, Default)]
pub struct ProjectListParams {
    /// Optional workspace filter.
    pub workspace: Option<String>,
    /// Optional team filter.
    pub team: Option<String>,
    /// Filter archived flag.
    pub archived: Option<bool>,
    /// Additional fields to request.
    pub fields: BTreeSet<String>,
    /// Maximum number of items to fetch (client side constraint).
    pub limit: Option<usize>,
    /// Optional saved filter expressions.
    pub filters: Vec<ProjectFilter>,
    /// Sort field.
    pub sort: Option<ProjectSort>,
}

impl ProjectListParams {
    /// Transform parameters into query pairs for the API.
    #[must_use]
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(workspace) = &self.workspace {
            pairs.push(("workspace".into(), workspace.clone()));
        }
        if let Some(team) = &self.team {
            pairs.push(("team".into(), team.clone()));
        }
        if let Some(archived) = self.archived {
            pairs.push(("archived".into(), archived.to_string()));
        }
        if !self.fields.is_empty() {
            let field_list = self.fields.iter().cloned().collect::<Vec<_>>().join(",");
            pairs.push(("opt_fields".into(), field_list));
        }
        pairs
    }
}

/// Sort options supported by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectSort {
    /// Sort by project name (case insensitive).
    Name,
    /// Sort by creation timestamp.
    CreatedAt,
    /// Sort by modification timestamp.
    ModifiedAt,
}

/// Statement describing a single filter operation.
#[derive(Debug, Clone)]
pub enum ProjectFilter {
    /// Field equality.
    Equals(String, String),
    /// Field inequality.
    NotEquals(String, String),
    /// Regular expression match.
    Regex(String, Regex),
    /// Substring match.
    Contains(String, String),
}

impl ProjectFilter {
    /// Evaluate filter against a project instance.
    #[must_use]
    pub fn matches(&self, project: &Project) -> bool {
        match self {
            Self::Equals(field, expected) => {
                field_value(project, field).is_some_and(|value| value == expected.as_str())
            }
            Self::NotEquals(field, forbidden) => {
                field_value(project, field).is_none_or(|value| value != forbidden.as_str())
            }
            Self::Regex(field, pattern) => {
                field_value(project, field).is_some_and(|value| pattern.is_match(&value))
            }
            Self::Contains(field, needle) => field_value(project, field).is_some_and(|value| {
                value
                    .to_ascii_lowercase()
                    .contains(&needle.to_ascii_lowercase())
            }),
        }
    }
}

fn field_value(project: &Project, field: &str) -> Option<String> {
    match field {
        "name" => Some(project.name.clone()),
        "gid" => Some(project.gid.clone()),
        "notes" => project.notes.clone(),
        "color" => project.color.clone(),
        "archived" => Some(project.archived.to_string()),
        "public" => project.public.map(|value| value.to_string()),
        "due_on" => project.due_on.clone(),
        "start_on" => project.start_on.clone(),
        "created_at" => project.created_at.clone(),
        "modified_at" => project.modified_at.clone(),
        "workspace" => project
            .workspace
            .as_ref()
            .map(super::workspace::WorkspaceReference::label),
        "team" => project
            .team
            .as_ref()
            .map(super::workspace::WorkspaceReference::label),
        "owner" => project
            .owner
            .as_ref()
            .map(super::user::UserReference::label),
        "owner.name" | "owner_name" => project.owner.as_ref().and_then(|owner| owner.name.clone()),
        "owner.email" | "owner_email" => {
            project.owner.as_ref().and_then(|owner| owner.email.clone())
        }
        other => project
            .custom_fields
            .get(other)
            .and_then(|value| value.as_str().map(ToString::to_string)),
    }
}

/// Request payload for creating a project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProjectCreateData {
    /// Name of the new project.
    pub name: String,
    /// Workspace or organization identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// Team identifier when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    /// Notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Color slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Start date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// Whether the project is public.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Owner identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// List of member identifiers.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
    /// Custom field assignments.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub custom_fields: BTreeMap<String, serde_json::Value>,
}

/// Envelope for create requests.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectCreateRequest {
    /// Wrapped data payload.
    pub data: ProjectCreateData,
}

/// Request payload for updating existing projects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ProjectUpdateData {
    /// New project name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Notes/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Color slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Start date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// Archive toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// Privacy flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Owner change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl ProjectUpdateData {
    /// Determine whether any fields have been populated.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.notes.is_none()
            && self.color.is_none()
            && self.start_on.is_none()
            && self.due_on.is_none()
            && self.archived.is_none()
            && self.public.is_none()
            && self.owner.is_none()
    }
}

/// Envelope for update requests.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectUpdateRequest {
    /// Wrapped data payload.
    pub data: ProjectUpdateData,
}

/// Template definition stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectTemplate {
    /// Template name for display.
    pub name: String,
    /// Underlying project configuration.
    pub project: ProjectCreateData,
    /// Optional description for humans.
    #[serde(default)]
    pub description: Option<String>,
    /// Tag metadata applied during listing.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Source file path, populated at load time.
    #[serde(skip)]
    pub source: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn sample_project() -> Project {
        Project {
            gid: "P1".into(),
            name: "Demo Project".into(),
            resource_type: None,
            notes: Some("Demo".into()),
            color: None,
            archived: false,
            public: Some(true),
            due_on: None,
            start_on: None,
            created_at: None,
            modified_at: None,
            workspace: Some(WorkspaceReference {
                gid: "W1".into(),
                name: Some("Engineering".into()),
                resource_type: None,
            }),
            team: None,
            owner: Some(UserReference {
                gid: "U1".into(),
                name: Some("Owner".into()),
                resource_type: None,
                email: Some("owner@example.com".into()),
            }),
            members: Vec::new(),
            statuses: Vec::new(),
            custom_fields: BTreeMap::new(),
        }
    }

    #[test]
    fn equals_filter_matches_project_name() {
        let project = sample_project();
        let filter = ProjectFilter::Equals("name".into(), "Demo Project".into());
        assert!(filter.matches(&project));
    }

    #[test]
    fn regex_filter_matches_owner_email() {
        let project = sample_project();
        let filter = ProjectFilter::Regex(
            "owner.email".into(),
            Regex::new(r"(?i)owner@example\.com").unwrap(),
        );
        assert!(filter.matches(&project));
    }

    #[test]
    fn update_data_is_empty_when_no_fields_set() {
        let mut data = ProjectUpdateData::default();
        assert!(data.is_empty());
        data.archived = Some(true);
        assert!(!data.is_empty());
    }
}
