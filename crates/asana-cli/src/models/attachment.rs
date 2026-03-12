//! Attachment metadata returned alongside tasks.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// File attachment associated with a task or comment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Attachment {
    /// Globally unique identifier.
    pub gid: String,
    /// Display name for the attachment.
    pub name: String,
    /// Resource type marker reported by Asana.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Additional subtype information (e.g., "asana", "dropbox").
    #[serde(default)]
    pub resource_subtype: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Host provider identifier.
    #[serde(default)]
    pub host: Option<String>,
    /// Attachment size in bytes when available.
    #[serde(default)]
    pub size: Option<u64>,
    /// Direct download URL (expires).
    #[serde(default)]
    pub download_url: Option<String>,
    /// Permanent permalink URL.
    #[serde(default)]
    pub permanent_url: Option<String>,
}

/// Parameters for listing attachments.
#[derive(Debug, Clone)]
pub struct AttachmentListParams {
    /// Task identifier.
    pub task_gid: String,
    /// Maximum number to fetch.
    pub limit: Option<usize>,
}

/// Parameters for uploading attachments.
#[derive(Debug, Clone)]
pub struct AttachmentUploadParams {
    /// Task identifier.
    pub task_gid: String,
    /// Local file path.
    pub file_path: PathBuf,
    /// Optional filename override.
    pub name: Option<String>,
}
