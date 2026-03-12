//! Tag-oriented data structures, builders, and request payloads.

use super::{user::UserReference, workspace::WorkspaceReference};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

/// Compact tag reference used in listings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub struct TagCompact {
    /// Globally unique identifier.
    pub gid: String,
    /// Tag name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl TagCompact {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.name
    }
}

/// Supported tag colors in Asana.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TagColor {
    /// Dark blue color.
    DarkBlue,
    /// Dark brown color.
    DarkBrown,
    /// Dark green color.
    DarkGreen,
    /// Dark orange color.
    DarkOrange,
    /// Dark pink color.
    DarkPink,
    /// Dark purple color.
    DarkPurple,
    /// Dark red color.
    DarkRed,
    /// Dark teal color.
    DarkTeal,
    /// Dark warm gray color.
    DarkWarmGray,
    /// Light blue color.
    LightBlue,
    /// Light brown color.
    LightBrown,
    /// Light green color.
    LightGreen,
    /// Light orange color.
    LightOrange,
    /// Light pink color.
    LightPink,
    /// Light purple color.
    LightPurple,
    /// Light red color.
    LightRed,
    /// Light teal color.
    LightTeal,
    /// Light warm gray color.
    LightWarmGray,
    /// Fallback for unsupported values.
    #[serde(other)]
    Unknown,
}

/// Full tag payload returned by the Asana API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Tag {
    /// Globally unique identifier.
    pub gid: String,
    /// Tag name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Tag color.
    #[serde(default)]
    pub color: Option<TagColor>,
    /// Notes or description.
    #[serde(default)]
    pub notes: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Followers for notifications.
    #[serde(default)]
    pub followers: Vec<UserReference>,
    /// Workspace reference.
    #[serde(default)]
    pub workspace: Option<WorkspaceReference>,
    /// Public permalink.
    #[serde(default)]
    pub permalink_url: Option<String>,
}

/// Parameters for listing tags via the API.
#[derive(Debug, Clone, Default)]
pub struct TagListParams {
    /// Workspace filter (required for listing tags).
    pub workspace: String,
    /// Maximum number of items to fetch (client side).
    pub limit: Option<usize>,
}

impl TagListParams {
    /// Convert the structure into query string pairs.
    #[must_use]
    pub fn to_query(&self) -> Vec<(String, String)> {
        vec![("workspace".into(), self.workspace.clone())]
    }
}

/// Payload for creating tags.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TagCreateData {
    /// Tag name (required).
    pub name: String,
    /// Workspace or organization identifier (required).
    pub workspace: String,
    /// Optional color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TagColor>,
    /// Optional notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Followers to notify.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub followers: Vec<String>,
}

/// API envelope for create requests.
#[derive(Debug, Clone, Serialize)]
pub struct TagCreateRequest {
    /// Wrapped data payload.
    pub data: TagCreateData,
}

/// Builder for constructing validated tag create payloads.
#[derive(Debug, Clone)]
pub struct TagCreateBuilder {
    data: TagCreateData,
}

impl TagCreateBuilder {
    /// Start building a new tag payload with the required name and workspace.
    #[must_use]
    pub fn new(name: impl Into<String>, workspace: impl Into<String>) -> Self {
        Self {
            data: TagCreateData {
                name: name.into(),
                workspace: workspace.into(),
                color: None,
                notes: None,
                followers: Vec::new(),
            },
        }
    }

    /// Override the tag name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = name.into();
        self
    }

    /// Set the tag color.
    #[must_use]
    pub fn color(mut self, color: TagColor) -> Self {
        self.data.color = Some(color);
        self
    }

    /// Provide notes or description.
    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.data.notes = Some(notes.into());
        self
    }

    /// Add a follower identifier.
    #[must_use]
    pub fn follower(mut self, follower: impl Into<String>) -> Self {
        let gid = follower.into();
        if !self.data.followers.contains(&gid) {
            self.data.followers.push(gid);
        }
        self
    }

    /// Finalise the builder into a request payload performing validation.
    ///
    /// # Errors
    ///
    /// Returns a validation error if mandatory fields are missing or invalid.
    pub fn build(self) -> Result<TagCreateRequest, TagValidationError> {
        if self.data.name.trim().is_empty() {
            return Err(TagValidationError::MissingName);
        }
        if self.data.workspace.trim().is_empty() {
            return Err(TagValidationError::MissingWorkspace);
        }
        Ok(TagCreateRequest { data: self.data })
    }
}

/// Payload for updating existing tags.
///
/// Uses `Option<Option<T>>` for certain fields to distinguish three API states:
/// - `None`: Don't update field (omit from JSON payload)
/// - `Some(None)`: Clear field (send `null` in JSON to remove value)
/// - `Some(Some(value))`: Set field to new value
///
/// This is required by the Asana API which treats missing fields differently from
/// explicit `null` values. Omitting a field preserves its current value, while
/// sending `null` clears it.
#[allow(clippy::option_option)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TagUpdateData {
    /// Tag name update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Color update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<TagColor>,
    /// Notes update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Option<String>>,
    /// Replace followers with the provided identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<Vec<String>>,
}

impl TagUpdateData {
    /// Determine whether any fields have been set.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.color.is_none()
            && self.notes.is_none()
            && self.followers.is_none()
    }
}

/// API envelope for update requests.
#[derive(Debug, Clone, Serialize)]
pub struct TagUpdateRequest {
    /// Wrapped data payload.
    pub data: TagUpdateData,
}

/// Builder for constructing validated tag update payloads.
#[derive(Debug, Default, Clone)]
pub struct TagUpdateBuilder {
    data: TagUpdateData,
}

impl TagUpdateBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tag name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    /// Set the tag color.
    #[must_use]
    pub fn color(mut self, color: TagColor) -> Self {
        self.data.color = Some(color);
        self
    }

    /// Set or update notes.
    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.data.notes = Some(Some(notes.into()));
        self
    }

    /// Clear notes.
    #[must_use]
    pub fn clear_notes(mut self) -> Self {
        self.data.notes = Some(None);
        self
    }

    /// Replace followers with the provided identifiers.
    #[must_use]
    pub fn followers<I, S>(mut self, followers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut values: Vec<String> = followers.into_iter().map(Into::into).collect();
        values.sort();
        values.dedup();
        self.data.followers = Some(values);
        self
    }

    /// Finalise the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if no fields were modified.
    pub fn build(self) -> Result<TagUpdateRequest, TagValidationError> {
        if self.data.is_empty() {
            return Err(TagValidationError::EmptyUpdate);
        }
        Ok(TagUpdateRequest { data: self.data })
    }
}

/// Errors emitted during tag payload validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TagValidationError {
    /// Tag name was missing or blank.
    #[error("tag name cannot be empty")]
    MissingName,
    /// Workspace identifier missing when creating a tag.
    #[error("tags require a workspace identifier")]
    MissingWorkspace,
    /// Update payload did not contain any fields.
    #[error("tag update payload does not include any changes")]
    EmptyUpdate,
}

impl Deref for TagUpdateBuilder {
    type Target = TagUpdateData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_builder_requires_name() {
        let builder = TagCreateBuilder::new("  ", "ws-123");
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TagValidationError::MissingName);
    }

    #[test]
    fn create_builder_requires_workspace() {
        let builder = TagCreateBuilder::new("Important", "  ");
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TagValidationError::MissingWorkspace);
    }

    #[test]
    fn create_builder_success() {
        let builder = TagCreateBuilder::new("Important", "ws-123")
            .color(TagColor::DarkRed)
            .notes("High priority items");
        let request = builder.build().expect("builder should succeed");
        assert_eq!(request.data.name, "Important");
        assert_eq!(request.data.workspace, "ws-123");
        assert_eq!(request.data.color, Some(TagColor::DarkRed));
    }

    #[test]
    fn update_builder_requires_changes() {
        let builder = TagUpdateBuilder::new();
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TagValidationError::EmptyUpdate);
    }

    #[test]
    fn update_builder_accepts_changes() {
        let request = TagUpdateBuilder::new()
            .name("Updated")
            .color(TagColor::LightGreen)
            .build()
            .expect("builder should succeed");
        assert_eq!(request.data.name.as_deref(), Some("Updated"));
        assert_eq!(request.data.color, Some(TagColor::LightGreen));
    }

    #[test]
    fn update_builder_clears_notes() {
        let request = TagUpdateBuilder::new()
            .clear_notes()
            .build()
            .expect("builder should succeed");
        assert_eq!(request.data.notes, Some(None));
    }
}
