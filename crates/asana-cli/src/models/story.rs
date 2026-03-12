//! Story (comment) data structures for task activity.

use super::user::UserReference;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

/// Story type classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    /// User-created comment.
    Comment,
    /// System-generated activity (not supported for creation).
    System,
}

/// Compact story reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::struct_field_names)]
pub struct StoryCompact {
    /// Globally unique identifier.
    pub gid: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Story type.
    #[serde(rename = "type")]
    pub story_type: StoryType,
}

/// Full story payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::struct_field_names)]
pub struct Story {
    /// Globally unique identifier.
    pub gid: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Story type.
    #[serde(rename = "type")]
    pub story_type: StoryType,
    /// Plain text content.
    #[serde(default)]
    pub text: Option<String>,
    /// HTML formatted content.
    #[serde(default)]
    pub html_text: Option<String>,
    /// Whether story is pinned.
    #[serde(default)]
    pub is_pinned: bool,
    /// Whether story can be edited by current user.
    #[serde(default)]
    pub is_editable: bool,
    /// Whether story has been edited.
    #[serde(default)]
    pub is_edited: bool,
    /// Story author.
    #[serde(default)]
    pub created_by: Option<UserReference>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Parameters for listing stories.
#[derive(Debug, Clone, Default)]
pub struct StoryListParams {
    /// Task identifier.
    pub task_gid: String,
    /// Maximum number to fetch.
    pub limit: Option<usize>,
}

/// Payload for creating stories.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct StoryCreateData {
    /// Plain text content (required, mutually exclusive with `html_text`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML formatted content (mutually exclusive with text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
    /// Whether to pin the comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
}

/// API envelope for create requests.
#[derive(Debug, Clone, Serialize)]
pub struct StoryCreateRequest {
    /// Story data payload.
    pub data: StoryCreateData,
}

/// Builder for story create payloads.
#[derive(Debug, Clone)]
pub struct StoryCreateBuilder {
    data: StoryCreateData,
}

impl StoryCreateBuilder {
    /// Create builder with plain text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            data: StoryCreateData {
                text: Some(text.into()),
                html_text: None,
                is_pinned: None,
            },
        }
    }

    /// Create builder with HTML text.
    #[must_use]
    pub fn with_html(html_text: impl Into<String>) -> Self {
        Self {
            data: StoryCreateData {
                text: None,
                html_text: Some(html_text.into()),
                is_pinned: None,
            },
        }
    }

    /// Set whether comment should be pinned.
    #[must_use]
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.data.is_pinned = Some(pinned);
        self
    }

    /// Build the request.
    ///
    /// # Errors
    /// Returns [`StoryValidationError`] if the request has missing or conflicting text fields.
    pub fn build(self) -> Result<StoryCreateRequest, StoryValidationError> {
        if self.data.text.is_none() && self.data.html_text.is_none() {
            return Err(StoryValidationError::MissingText);
        }
        if self.data.text.is_some() && self.data.html_text.is_some() {
            return Err(StoryValidationError::BothTextFormats);
        }
        Ok(StoryCreateRequest { data: self.data })
    }
}

/// Payload for updating stories.
#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct StoryUpdateData {
    /// Updated plain text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Updated HTML text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_text: Option<String>,
    /// Updated pin status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
}

/// API envelope for update requests.
#[derive(Debug, Clone, Serialize)]
pub struct StoryUpdateRequest {
    /// Story update data payload.
    pub data: StoryUpdateData,
}

/// Builder for story update payloads.
#[derive(Debug, Clone, Default)]
pub struct StoryUpdateBuilder {
    data: StoryUpdateData,
}

impl StoryUpdateBuilder {
    /// Create a new empty update builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the plain text content.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.data.text = Some(text.into());
        self
    }

    /// Set the HTML formatted content.
    #[must_use]
    pub fn html_text(mut self, html: impl Into<String>) -> Self {
        self.data.html_text = Some(html.into());
        self
    }

    /// Set whether the comment is pinned.
    #[must_use]
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.data.is_pinned = Some(pinned);
        self
    }

    /// Build the update request.
    ///
    /// # Errors
    /// Returns [`StoryValidationError`] if no fields are set to update.
    pub fn build(self) -> Result<StoryUpdateRequest, StoryValidationError> {
        if self.data.text.is_none()
            && self.data.html_text.is_none()
            && self.data.is_pinned.is_none()
        {
            return Err(StoryValidationError::EmptyUpdate);
        }
        Ok(StoryUpdateRequest { data: self.data })
    }
}

/// Validation errors for story payloads.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoryValidationError {
    /// Story must have either text or `html_text`.
    #[error("story must have either text or html_text")]
    MissingText,
    /// Story cannot have both text and `html_text`.
    #[error("story cannot have both text and html_text")]
    BothTextFormats,
    /// Story update must change at least one field.
    #[error("story update must change at least one field")]
    EmptyUpdate,
}

impl Deref for StoryUpdateBuilder {
    type Target = StoryUpdateData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_builder_requires_text() {
        let builder = StoryCreateBuilder {
            data: StoryCreateData {
                text: None,
                html_text: None,
                is_pinned: None,
            },
        };
        assert_eq!(
            builder.build().unwrap_err(),
            StoryValidationError::MissingText
        );
    }

    #[test]
    fn create_builder_rejects_both_formats() {
        let builder = StoryCreateBuilder {
            data: StoryCreateData {
                text: Some("plain".into()),
                html_text: Some("<b>html</b>".into()),
                is_pinned: None,
            },
        };
        assert_eq!(
            builder.build().unwrap_err(),
            StoryValidationError::BothTextFormats
        );
    }

    #[test]
    fn create_builder_success() {
        let request = StoryCreateBuilder::new("This is a comment")
            .pinned(true)
            .build()
            .unwrap();
        assert_eq!(request.data.text.as_deref(), Some("This is a comment"));
        assert_eq!(request.data.is_pinned, Some(true));
    }

    #[test]
    fn update_builder_requires_changes() {
        let builder = StoryUpdateBuilder::new();
        assert_eq!(
            builder.build().unwrap_err(),
            StoryValidationError::EmptyUpdate
        );
    }
}
