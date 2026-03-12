//! Task-oriented data structures, builders, and request payloads.

use super::{
    attachment::Attachment,
    custom_field::{CustomField, CustomFieldValue},
    user::UserReference,
    workspace::WorkspaceReference,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use thiserror::Error;

/// Lightweight reference to a task or subtask.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TaskReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Optional display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl TaskReference {
    /// Produce a human readable label for the reference.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Task membership metadata linking it to projects and sections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TaskMembership {
    /// Parent project reference.
    #[serde(default)]
    pub project: Option<TaskProjectReference>,
    /// Section reference when available.
    #[serde(default)]
    pub section: Option<TaskSectionReference>,
}

/// Compact project reference used within task payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub struct TaskProjectReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Optional project name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl TaskProjectReference {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Compact section reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TaskSectionReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Section name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl TaskSectionReference {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Compact tag reference used within tasks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub struct TaskTagReference {
    /// Globally unique identifier.
    pub gid: String,
    /// Tag name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
}

impl TaskTagReference {
    /// Human readable label.
    #[must_use]
    pub fn label(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.gid.clone())
    }
}

/// Status of a task relative to the assignee's prioritisation buckets.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskAssigneeStatus {
    /// Task recently assigned.
    #[default]
    Inbox,
    /// Deferred for later.
    Later,
    /// Scheduled for upcoming work.
    Upcoming,
    /// Due today.
    Today,
    /// Waiting on someone/something else.
    Waiting,
    /// Fallback for unsupported values.
    #[serde(other)]
    Unknown,
}

/// Full task payload returned by the Asana API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    /// Globally unique identifier.
    pub gid: String,
    /// Display name.
    pub name: String,
    /// Resource type marker.
    #[serde(default)]
    pub resource_type: Option<String>,
    /// Resource subtype marker.
    #[serde(default)]
    pub resource_subtype: Option<String>,
    /// Optional notes in plain text.
    #[serde(default)]
    pub notes: Option<String>,
    /// Optional notes in HTML.
    #[serde(default)]
    pub html_notes: Option<String>,
    /// Completion flag.
    #[serde(default)]
    pub completed: bool,
    /// Time the task was completed.
    #[serde(default)]
    pub completed_at: Option<String>,
    /// User who completed the task.
    #[serde(default)]
    pub completed_by: Option<UserReference>,
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Last modification timestamp.
    #[serde(default)]
    pub modified_at: Option<String>,
    /// Due date (all day).
    #[serde(default)]
    pub due_on: Option<String>,
    /// Due timestamp (specific time).
    #[serde(default)]
    pub due_at: Option<String>,
    /// Start date (all day).
    #[serde(default)]
    pub start_on: Option<String>,
    /// Start timestamp (specific time).
    #[serde(default)]
    pub start_at: Option<String>,
    /// Assigned user.
    #[serde(default)]
    pub assignee: Option<UserReference>,
    /// Assignee status bucket.
    #[serde(default)]
    pub assignee_status: Option<TaskAssigneeStatus>,
    /// Workspace reference.
    #[serde(default)]
    pub workspace: Option<WorkspaceReference>,
    /// Parent task reference when this is a subtask.
    #[serde(default)]
    pub parent: Option<TaskReference>,
    /// Project/section memberships.
    #[serde(default)]
    pub memberships: Vec<TaskMembership>,
    /// Projects referenced directly.
    #[serde(default)]
    pub projects: Vec<TaskProjectReference>,
    /// Tag references.
    #[serde(default)]
    pub tags: Vec<TaskTagReference>,
    /// Followers for notifications.
    #[serde(default)]
    pub followers: Vec<UserReference>,
    /// Tasks this task depends on.
    #[serde(default)]
    pub dependencies: Vec<TaskReference>,
    /// Tasks blocked by this task.
    #[serde(default)]
    pub dependents: Vec<TaskReference>,
    /// Custom field payloads.
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
    /// Attachment metadata when requested.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Public permalink.
    #[serde(default)]
    pub permalink_url: Option<String>,
    /// Number of subtasks this task contains.
    #[serde(default)]
    pub num_subtasks: Option<i64>,
}

impl Task {
    /// Determines whether the task is currently incomplete.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        !self.completed
    }
}

/// Parameters for listing tasks via the API.
#[derive(Debug, Clone, Default)]
pub struct TaskListParams {
    /// Workspace filter.
    pub workspace: Option<String>,
    /// Project filter.
    pub project: Option<String>,
    /// Section filter.
    pub section: Option<String>,
    /// Assignee filter.
    pub assignee: Option<String>,
    /// Completed since timestamp.
    pub completed_since: Option<String>,
    /// Modified since timestamp.
    pub modified_since: Option<String>,
    /// Exact due date filter.
    pub due_on: Option<String>,
    /// Include subtasks in listing.
    pub include_subtasks: bool,
    /// Maximum number of items to fetch (client side).
    pub limit: Option<usize>,
    /// Additional fields to request.
    pub fields: BTreeSet<String>,
    /// Sort order applied post-fetch.
    pub sort: Option<TaskSort>,
    /// Post-fetch completion filter.
    pub completed: Option<bool>,
    /// Post-fetch due date upper bound (inclusive, YYYY-MM-DD).
    pub due_before: Option<String>,
    /// Post-fetch due date lower bound (inclusive, YYYY-MM-DD).
    pub due_after: Option<String>,
}

impl TaskListParams {
    /// Convert the structure into query string pairs.
    #[must_use]
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(workspace) = &self.workspace {
            pairs.push(("workspace".into(), workspace.clone()));
        }
        if let Some(project) = &self.project {
            pairs.push(("project".into(), project.clone()));
        }
        if let Some(section) = &self.section {
            pairs.push(("section".into(), section.clone()));
        }
        if let Some(assignee) = &self.assignee {
            pairs.push(("assignee".into(), assignee.clone()));
        }
        if let Some(completed_since) = &self.completed_since {
            pairs.push(("completed_since".into(), completed_since.clone()));
        }
        if let Some(modified_since) = &self.modified_since {
            pairs.push(("modified_since".into(), modified_since.clone()));
        }
        if let Some(due_on) = &self.due_on {
            pairs.push(("due_on".into(), due_on.clone()));
        }
        // Note: include_subtasks flag is used to control separate subtask fetching
        // after the main API call. The deprecated opt_expand=subtasks parameter
        // no longer returns complete field data and has been removed.
        if !self.fields.is_empty() {
            let field_list = self.fields.iter().cloned().collect::<Vec<_>>().join(",");
            pairs.push(("opt_fields".into(), field_list));
        }
        pairs
    }

    /// Apply local filters after API pagination.
    pub fn apply_post_filters(&self, tasks: &mut Vec<Task>) {
        if let Some(expected) = self.completed {
            tasks.retain(|task| task.completed == expected);
        }
        if let Some(due_before) = &self.due_before {
            tasks.retain(|task| task.due_on.as_ref().is_some_and(|due| due <= due_before));
        }
        if let Some(due_after) = &self.due_after {
            tasks.retain(|task| task.due_on.as_ref().is_some_and(|due| due >= due_after));
        }
    }
}

/// Supported sort orders for task listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSort {
    /// Alphabetical by task name.
    Name,
    /// Due date ascending.
    DueOn,
    /// Creation timestamp ascending.
    CreatedAt,
    /// Modification timestamp ascending.
    ModifiedAt,
    /// Assignee display name.
    Assignee,
}

/// Parameters for searching tasks.
#[derive(Debug, Clone, Default)]
pub struct TaskSearchParams {
    /// Workspace to search in (required).
    pub workspace: String,
    /// Full-text search query.
    pub text: Option<String>,
    /// Resource subtype filter.
    pub resource_subtype: Option<String>,
    /// Completion status filter.
    pub completed: Option<bool>,
    /// Include subtasks.
    pub is_subtask: Option<bool>,
    /// Filter blocked tasks.
    pub is_blocked: Option<bool>,
    /// Filter tasks with attachments.
    pub has_attachment: Option<bool>,
    /// Assignee filter (gid or "me").
    pub assignee: Option<String>,
    /// Project filter (gid).
    pub projects: Vec<String>,
    /// Section filter (gid).
    pub sections: Vec<String>,
    /// Tag filter (gid).
    pub tags: Vec<String>,
    /// Created after date (YYYY-MM-DD).
    pub created_after: Option<String>,
    /// Created before date (YYYY-MM-DD).
    pub created_before: Option<String>,
    /// Modified after date (YYYY-MM-DD).
    pub modified_after: Option<String>,
    /// Modified before date (YYYY-MM-DD).
    pub modified_before: Option<String>,
    /// Due after date (YYYY-MM-DD).
    pub due_after: Option<String>,
    /// Due before date (YYYY-MM-DD).
    pub due_before: Option<String>,
    /// Sort field.
    pub sort_by: Option<String>,
    /// Sort ascending.
    pub sort_ascending: bool,
    /// Maximum number of items to fetch.
    pub limit: Option<usize>,
    /// Additional fields to request.
    pub fields: BTreeSet<String>,
}

impl TaskSearchParams {
    /// Convert to query parameters.
    #[must_use]
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(text) = &self.text {
            pairs.push(("text".into(), text.clone()));
        }
        if let Some(subtype) = &self.resource_subtype {
            pairs.push(("resource_subtype".into(), subtype.clone()));
        }
        if let Some(completed) = self.completed {
            pairs.push(("completed".into(), completed.to_string()));
        }
        if let Some(is_subtask) = self.is_subtask {
            pairs.push(("is_subtask".into(), is_subtask.to_string()));
        }
        if let Some(is_blocked) = self.is_blocked {
            pairs.push(("is_blocked".into(), is_blocked.to_string()));
        }
        if let Some(has_attachment) = self.has_attachment {
            pairs.push(("has_attachment".into(), has_attachment.to_string()));
        }
        if let Some(assignee) = &self.assignee {
            pairs.push(("assignee.any".into(), assignee.clone()));
        }

        for project in &self.projects {
            pairs.push(("projects.any".into(), project.clone()));
        }
        for section in &self.sections {
            pairs.push(("sections.any".into(), section.clone()));
        }
        for tag in &self.tags {
            pairs.push(("tags.any".into(), tag.clone()));
        }

        if let Some(date) = &self.created_after {
            pairs.push(("created_at.after".into(), date.clone()));
        }
        if let Some(date) = &self.created_before {
            pairs.push(("created_at.before".into(), date.clone()));
        }
        if let Some(date) = &self.modified_after {
            pairs.push(("modified_at.after".into(), date.clone()));
        }
        if let Some(date) = &self.modified_before {
            pairs.push(("modified_at.before".into(), date.clone()));
        }
        if let Some(date) = &self.due_after {
            pairs.push(("due_on.after".into(), date.clone()));
        }
        if let Some(date) = &self.due_before {
            pairs.push(("due_on.before".into(), date.clone()));
        }

        if let Some(sort_by) = &self.sort_by {
            pairs.push(("sort_by".into(), sort_by.clone()));
            pairs.push(("sort_ascending".into(), self.sort_ascending.to_string()));
        }

        if !self.fields.is_empty() {
            let field_list = self.fields.iter().cloned().collect::<Vec<_>>().join(",");
            pairs.push(("opt_fields".into(), field_list));
        }

        pairs
    }
}

/// Payload for creating tasks.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TaskCreateData {
    /// Task name (required).
    pub name: String,
    /// Optional notes (plain text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Optional notes in HTML format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<String>,
    /// Workspace or organization identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// Associated project identifiers.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<String>,
    /// Section identifier when provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Parent task identifier for creating subtasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Assigned user (gid or email).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Due date (all day).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// Due timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    /// Start date (all day).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<String>,
    /// Start timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    /// Tags to apply.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Followers to notify.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub followers: Vec<String>,
    /// Custom field assignments.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub custom_fields: BTreeMap<String, serde_json::Value>,
}

/// API envelope for create requests.
#[derive(Debug, Clone, Serialize)]
pub struct TaskCreateRequest {
    /// Wrapped data payload.
    pub data: TaskCreateData,
}

/// Builder for constructing validated task create payloads.
#[derive(Debug, Clone)]
pub struct TaskCreateBuilder {
    data: TaskCreateData,
}

impl TaskCreateBuilder {
    /// Start building a new task payload with the required name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            data: TaskCreateData {
                name,
                notes: None,
                html_notes: None,
                workspace: None,
                projects: Vec::new(),
                section: None,
                parent: None,
                assignee: None,
                due_on: None,
                due_at: None,
                start_on: None,
                start_at: None,
                tags: Vec::new(),
                followers: Vec::new(),
                custom_fields: BTreeMap::new(),
            },
        }
    }

    /// Override the task name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = name.into();
        self
    }

    /// Provide plain text notes.
    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.data.notes = Some(notes.into());
        self
    }

    /// Provide HTML formatted notes.
    #[must_use]
    pub fn html_notes(mut self, notes: impl Into<String>) -> Self {
        self.data.html_notes = Some(notes.into());
        self
    }

    /// Set the workspace gid.
    #[must_use]
    pub fn workspace(mut self, workspace: impl Into<String>) -> Self {
        self.data.workspace = Some(workspace.into());
        self
    }

    /// Add a project gid association.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        let gid = project.into();
        if !self.data.projects.contains(&gid) {
            self.data.projects.push(gid);
        }
        self
    }

    /// Target a specific section gid when creating within a project.
    #[must_use]
    pub fn section(mut self, section: impl Into<String>) -> Self {
        self.data.section = Some(section.into());
        self
    }

    /// Set the parent task gid to create a subtask.
    #[must_use]
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.data.parent = Some(parent.into());
        self
    }

    /// Assign the task to a user (gid or email).
    #[must_use]
    pub fn assignee(mut self, assignee: impl Into<String>) -> Self {
        self.data.assignee = Some(assignee.into());
        self
    }

    /// Set the due date (all day).
    #[must_use]
    pub fn due_on(mut self, due_on: impl Into<String>) -> Self {
        self.data.due_on = Some(due_on.into());
        self
    }

    /// Set the due timestamp.
    #[must_use]
    pub fn due_at(mut self, due_at: impl Into<String>) -> Self {
        self.data.due_at = Some(due_at.into());
        self
    }

    /// Set the start date (all day).
    #[must_use]
    pub fn start_on(mut self, start_on: impl Into<String>) -> Self {
        self.data.start_on = Some(start_on.into());
        self
    }

    /// Set the start timestamp.
    #[must_use]
    pub fn start_at(mut self, start_at: impl Into<String>) -> Self {
        self.data.start_at = Some(start_at.into());
        self
    }

    /// Add a tag identifier.
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        let gid = tag.into();
        if !self.data.tags.contains(&gid) {
            self.data.tags.push(gid);
        }
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

    /// Assign a custom field value.
    #[must_use]
    pub fn custom_field(mut self, field_gid: impl Into<String>, value: CustomFieldValue) -> Self {
        self.data
            .custom_fields
            .insert(field_gid.into(), value.into_value());
        self
    }

    /// Finalise the builder into a request payload performing validation.
    ///
    /// # Errors
    ///
    /// Returns a validation error if mandatory fields are missing or invalid.
    pub fn build(self) -> Result<TaskCreateRequest, TaskValidationError> {
        if self.data.name.trim().is_empty() {
            return Err(TaskValidationError::MissingName);
        }
        if self.data.workspace.is_none()
            && self.data.projects.is_empty()
            && self.data.parent.is_none()
        {
            return Err(TaskValidationError::MissingScope);
        }
        Ok(TaskCreateRequest { data: self.data })
    }
}

/// Payload for updating existing tasks.
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
pub struct TaskUpdateData {
    /// Task name update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Plain text notes update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Option<String>>,
    /// HTML notes update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_notes: Option<Option<String>>,
    /// Completion flag change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    /// Assignee change (gid/email) or explicit null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Option<String>>,
    /// Due date change (all day) or explicit null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<Option<String>>,
    /// Due timestamp change or explicit null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<Option<String>>,
    /// Start date change or explicit null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_on: Option<Option<String>>,
    /// Start timestamp change or explicit null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<Option<String>>,
    /// Parent assignment or removal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Option<String>>,
    /// Replace tags with the provided identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Replace followers with the provided identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<Vec<String>>,
    /// Replace project associations with the provided identifiers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects: Option<Vec<String>>,
    /// Custom field updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<BTreeMap<String, serde_json::Value>>,
}

impl TaskUpdateData {
    /// Determine whether any fields have been set.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.notes.is_none()
            && self.html_notes.is_none()
            && self.completed.is_none()
            && self.assignee.is_none()
            && self.due_on.is_none()
            && self.due_at.is_none()
            && self.start_on.is_none()
            && self.start_at.is_none()
            && self.parent.is_none()
            && self.tags.is_none()
            && self.followers.is_none()
            && self.projects.is_none()
            && self.custom_fields.is_none()
    }
}

/// API envelope for update requests.
#[derive(Debug, Clone, Serialize)]
pub struct TaskUpdateRequest {
    /// Wrapped data payload.
    pub data: TaskUpdateData,
}

/// Builder for constructing validated task update payloads.
#[derive(Debug, Default, Clone)]
pub struct TaskUpdateBuilder {
    data: TaskUpdateData,
}

impl TaskUpdateBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the task name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.data.name = Some(name.into());
        self
    }

    /// Replace notes with the provided plain text.
    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.data.notes = Some(Some(notes.into()));
        self
    }

    /// Clear the plain text notes.
    #[must_use]
    pub fn clear_notes(mut self) -> Self {
        self.data.notes = Some(None);
        self
    }

    /// Set HTML formatted notes.
    #[must_use]
    pub fn html_notes(mut self, notes: impl Into<String>) -> Self {
        self.data.html_notes = Some(Some(notes.into()));
        self
    }

    /// Clear HTML notes.
    #[must_use]
    pub fn clear_html_notes(mut self) -> Self {
        self.data.html_notes = Some(None);
        self
    }

    /// Mark the task completed/incomplete.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn completed(mut self, completed: bool) -> Self {
        self.data.completed = Some(completed);
        self
    }

    /// Assign the task.
    #[must_use]
    pub fn assignee(mut self, assignee: impl Into<String>) -> Self {
        self.data.assignee = Some(Some(assignee.into()));
        self
    }

    /// Remove the current assignee.
    #[must_use]
    pub fn clear_assignee(mut self) -> Self {
        self.data.assignee = Some(None);
        self
    }

    /// Set the due date (all day).
    #[must_use]
    pub fn due_on(mut self, due_on: impl Into<String>) -> Self {
        self.data.due_on = Some(Some(due_on.into()));
        self
    }

    /// Clear the due date.
    #[must_use]
    pub fn clear_due_on(mut self) -> Self {
        self.data.due_on = Some(None);
        self
    }

    /// Set the due timestamp.
    #[must_use]
    pub fn due_at(mut self, due_at: impl Into<String>) -> Self {
        self.data.due_at = Some(Some(due_at.into()));
        self
    }

    /// Clear the due timestamp.
    #[must_use]
    pub fn clear_due_at(mut self) -> Self {
        self.data.due_at = Some(None);
        self
    }

    /// Set the start date.
    #[must_use]
    pub fn start_on(mut self, start_on: impl Into<String>) -> Self {
        self.data.start_on = Some(Some(start_on.into()));
        self
    }

    /// Clear the start date.
    #[must_use]
    pub fn clear_start_on(mut self) -> Self {
        self.data.start_on = Some(None);
        self
    }

    /// Set the start timestamp.
    #[must_use]
    pub fn start_at(mut self, start_at: impl Into<String>) -> Self {
        self.data.start_at = Some(Some(start_at.into()));
        self
    }

    /// Clear the start timestamp.
    #[must_use]
    pub fn clear_start_at(mut self) -> Self {
        self.data.start_at = Some(None);
        self
    }

    /// Set the parent task.
    #[must_use]
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.data.parent = Some(Some(parent.into()));
        self
    }

    /// Remove the parent task.
    #[must_use]
    pub fn clear_parent(mut self) -> Self {
        self.data.parent = Some(None);
        self
    }

    /// Replace tags with the provided identifiers.
    #[must_use]
    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut values: Vec<String> = tags.into_iter().map(Into::into).collect();
        values.sort();
        values.dedup();
        self.data.tags = Some(values);
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

    /// Replace project associations.
    #[must_use]
    pub fn projects<I, S>(mut self, projects: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut values: Vec<String> = projects.into_iter().map(Into::into).collect();
        values.sort();
        values.dedup();
        self.data.projects = Some(values);
        self
    }

    /// Set a custom field value.
    #[must_use]
    pub fn custom_field(mut self, field_gid: impl Into<String>, value: CustomFieldValue) -> Self {
        let map = self.data.custom_fields.get_or_insert_with(BTreeMap::new);
        map.insert(field_gid.into(), value.into_value());
        self
    }

    /// Finalise the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if no fields were modified.
    pub fn build(self) -> Result<TaskUpdateRequest, TaskValidationError> {
        if self.data.is_empty() {
            return Err(TaskValidationError::EmptyUpdate);
        }
        Ok(TaskUpdateRequest { data: self.data })
    }
}

/// Errors emitted during task payload validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TaskValidationError {
    /// Task name was missing or blank.
    #[error("task name cannot be empty")]
    MissingName,
    /// Workspace or project context missing when creating a task.
    #[error("tasks require either a workspace or at least one project")]
    MissingScope,
    /// Update payload did not contain any fields.
    #[error("task update payload does not include any changes")]
    EmptyUpdate,
}

impl Deref for TaskUpdateBuilder {
    type Target = TaskUpdateData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn create_builder_requires_name() {
        let builder = TaskCreateBuilder::new("  ");
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TaskValidationError::MissingName);
    }

    #[test]
    fn create_builder_requires_scope() {
        let builder = TaskCreateBuilder::new("Sample task").notes("demo");
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TaskValidationError::MissingScope);
    }

    #[test]
    fn create_builder_success() {
        let builder = TaskCreateBuilder::new("Sample task")
            .workspace("123")
            .assignee("me");
        let request = builder.build().expect("builder should succeed");
        assert_eq!(request.data.name, "Sample task");
        assert_eq!(request.data.workspace.as_deref(), Some("123"));
    }

    #[test]
    fn create_builder_accepts_parent_scope() {
        let request = TaskCreateBuilder::new("Child")
            .parent("T1")
            .build()
            .expect("builder should succeed");
        assert_eq!(request.data.parent.as_deref(), Some("T1"));
    }

    #[test]
    fn update_builder_requires_changes() {
        let builder = TaskUpdateBuilder::new();
        let result = builder.build();
        assert_eq!(result.unwrap_err(), TaskValidationError::EmptyUpdate);
    }

    #[test]
    fn update_builder_accepts_changes() {
        let request = TaskUpdateBuilder::new()
            .name("Updated")
            .completed(true)
            .build()
            .expect("builder should succeed");
        assert_eq!(request.data.name.as_deref(), Some("Updated"));
        assert_eq!(request.data.completed, Some(true));
    }

    #[test]
    fn create_builder_serializes_custom_field() {
        let request = TaskCreateBuilder::new("With field")
            .workspace("ws-1")
            .custom_field("cf1", CustomFieldValue::Bool(true))
            .build()
            .expect("builder should succeed");
        assert_eq!(
            request.data.custom_fields.get("cf1"),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn update_builder_clears_assignee() {
        let request = TaskUpdateBuilder::new()
            .clear_assignee()
            .build()
            .expect("builder should succeed");
        assert_eq!(request.data.assignee, Some(None));
    }

    #[test]
    fn update_builder_sets_custom_field_value() {
        let request = TaskUpdateBuilder::new()
            .custom_field("cf1", CustomFieldValue::Number(5.0))
            .build()
            .expect("builder should succeed");
        let map = request
            .data
            .custom_fields
            .as_ref()
            .expect("custom fields set");
        assert!(
            map.get("cf1")
                .and_then(Value::as_f64)
                .is_some_and(|value| (value - 5.0).abs() < f64::EPSILON)
        );
    }
}
