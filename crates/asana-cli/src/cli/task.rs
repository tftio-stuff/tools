//! Task CLI command implementations including subtasks and relationship management.

use super::build_api_client;
use crate::{
    api::{self, ApiClient},
    config::Config,
    error::Result,
    models::{
        AttachmentListParams, AttachmentUploadParams, CustomFieldValue, StoryCreateBuilder,
        StoryListParams, StoryUpdateBuilder, Task, TaskCreateBuilder, TaskCreateRequest,
        TaskListParams, TaskReference, TaskSearchParams, TaskSort, TaskUpdateBuilder,
        TaskUpdateRequest, TaskValidationError,
    },
    output::{
        TaskOutputFormat,
        task::{render_task_detail, render_task_list},
    },
};
use anyhow::{Context, anyhow, bail};
use chrono::Utc;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeSet, VecDeque},
    fmt::Write as FmtWrite,
    fs,
    io::{IsTerminal, stdout},
    path::{Path, PathBuf},
};
use tokio::runtime::Builder as RuntimeBuilder;
use tracing::{debug, warn};

/// Primary `task` subcommands.
#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    /// List tasks with optional filtering.
    List(TaskListArgs),
    /// Display detailed information about a task.
    Show(TaskShowArgs),
    /// Create a new task.
    Create(TaskCreateArgs),
    /// Update an existing task.
    Update(TaskUpdateArgs),
    /// Delete a task.
    Delete(TaskDeleteArgs),
    /// Create multiple tasks from structured input.
    CreateBatch(TaskBatchCreateArgs),
    /// Update multiple tasks from structured input.
    UpdateBatch(TaskBatchUpdateArgs),
    /// Complete multiple tasks from structured input.
    CompleteBatch(TaskBatchCompleteArgs),
    /// Search for tasks with fuzzy matching.
    Search(TaskSearchArgs),
    /// Manage subtasks.
    Subtasks {
        #[command(subcommand)]
        command: TaskSubtasksCommand,
    },
    /// Manage dependencies (tasks this task depends on).
    DependsOn {
        #[command(subcommand)]
        command: TaskDependencyCommand,
    },
    /// Manage dependents (tasks blocked by this task).
    Blocks {
        #[command(subcommand)]
        command: TaskDependentCommand,
    },
    /// Manage project memberships.
    Projects {
        #[command(subcommand)]
        command: TaskProjectCommand,
    },
    /// Manage task followers.
    Followers {
        #[command(subcommand)]
        command: TaskFollowerCommand,
    },
    /// Manage task tags.
    Tags {
        #[command(subcommand)]
        command: TaskTagCommand,
    },
    /// Manage task comments (stories).
    Comments {
        #[command(subcommand)]
        command: TaskCommentsCommand,
    },
    /// Manage task attachments.
    Attachments {
        #[command(subcommand)]
        command: TaskAttachmentsCommand,
    },
    /// Move a task to a section within a project.
    MoveToSection(TaskMoveToSectionArgs),
}

/// Arguments for `task list`.
#[derive(Args, Debug)]
pub struct TaskListArgs {
    /// Workspace identifier filter.
    #[arg(long)]
    pub workspace: Option<String>,
    /// Project identifier filter.
    #[arg(long)]
    pub project: Option<String>,
    /// Section identifier filter.
    #[arg(long)]
    pub section: Option<String>,
    /// Assignee identifier or email filter.
    #[arg(long)]
    pub assignee: Option<String>,
    /// Filter by completion state.
    #[arg(long)]
    pub completed: Option<bool>,
    /// Only include tasks due on or before the provided date.
    #[arg(long = "due-before")]
    pub due_before: Option<String>,
    /// Only include tasks due on or after the provided date.
    #[arg(long = "due-after")]
    pub due_after: Option<String>,
    /// Include subtasks in the listing response.
    #[arg(long)]
    pub include_subtasks: bool,
    /// Maximum number of tasks to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Sort order (`name`, `due_on`, `created_at`, `modified_at`, `assignee`).
    #[arg(long)]
    pub sort: Option<String>,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task show`.
#[derive(Args, Debug)]
pub struct TaskShowArgs {
    /// Task identifier (gid).
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task create`.
#[derive(Args, Debug)]
pub struct TaskCreateArgs {
    /// Task name; required unless `--interactive`.
    #[arg(long)]
    pub name: Option<String>,
    /// Workspace identifier.
    #[arg(long)]
    pub workspace: Option<String>,
    /// Project identifiers to associate with the task.
    #[arg(long = "project", value_name = "PROJECT")]
    pub projects: Vec<String>,
    /// Section identifier within the first project.
    #[arg(long)]
    pub section: Option<String>,
    /// Parent task identifier to create a subtask.
    #[arg(long)]
    pub parent: Option<String>,
    /// Assignee identifier (gid or email).
    #[arg(long)]
    pub assignee: Option<String>,
    /// Task notes in plain text.
    #[arg(long)]
    pub notes: Option<String>,
    /// Task notes in HTML format.
    #[arg(long = "html-notes")]
    pub html_notes: Option<String>,
    /// Due date (natural language accepted).
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    /// Due date/time (natural language accepted).
    #[arg(long = "due-at")]
    pub due_at: Option<String>,
    /// Start date (natural language accepted).
    #[arg(long = "start-on")]
    pub start_on: Option<String>,
    /// Start date/time (natural language accepted).
    #[arg(long = "start-at")]
    pub start_at: Option<String>,
    /// Tags to apply to the task.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Followers to subscribe to notifications.
    #[arg(long = "follower", value_name = "USER")]
    pub followers: Vec<String>,
    /// Custom field assignments in KEY=VALUE form.
    #[arg(long = "custom-field", value_name = "KEY=VALUE")]
    pub custom_fields: Vec<String>,
    /// Prompt for missing values interactively.
    #[arg(long)]
    pub interactive: bool,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task update`.
///
/// Contains multiple boolean fields (13 total) that map directly to CLI flags.
/// Each flag represents a distinct user action:
/// - 10 "clear" flags: `--clear-notes`, `--clear-assignee`, etc.
/// - 2 state toggles: `--complete`, `--incomplete`
/// - Other operation flags
///
/// This structure mirrors the CLI interface design where each boolean corresponds
/// to an explicit flag users can provide. Alternative designs (e.g., enums) would
/// break the existing CLI interface.
#[allow(clippy::struct_excessive_bools)]
#[derive(Args, Debug)]
pub struct TaskUpdateArgs {
    /// Task identifier (gid).
    #[arg(value_name = "TASK")]
    pub task: String,
    /// New task name.
    #[arg(long)]
    pub name: Option<String>,
    /// Replace notes with plain text content.
    #[arg(long)]
    pub notes: Option<String>,
    /// Clear existing plain text notes.
    #[arg(long)]
    pub clear_notes: bool,
    /// Replace notes with HTML content.
    #[arg(long = "html-notes")]
    pub html_notes: Option<String>,
    /// Clear existing HTML notes.
    #[arg(long)]
    pub clear_html_notes: bool,
    /// Assign the task to the specified user (gid or email).
    #[arg(long)]
    pub assignee: Option<String>,
    /// Remove the current assignee.
    #[arg(long)]
    pub clear_assignee: bool,
    /// Mark the task complete.
    #[arg(long)]
    pub complete: bool,
    /// Mark the task incomplete.
    #[arg(long)]
    pub incomplete: bool,
    /// Set all-day due date (natural language accepted).
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    /// Clear the all-day due date.
    #[arg(long)]
    pub clear_due_on: bool,
    /// Set due date/time (natural language accepted).
    #[arg(long = "due-at")]
    pub due_at: Option<String>,
    /// Clear the due date/time.
    #[arg(long)]
    pub clear_due_at: bool,
    /// Set start date (natural language accepted).
    #[arg(long = "start-on")]
    pub start_on: Option<String>,
    /// Clear the start date.
    #[arg(long)]
    pub clear_start_on: bool,
    /// Set start date/time (natural language accepted).
    #[arg(long = "start-at")]
    pub start_at: Option<String>,
    /// Clear the start date/time.
    #[arg(long)]
    pub clear_start_at: bool,
    /// Set parent task identifier.
    #[arg(long)]
    pub parent: Option<String>,
    /// Remove the parent task.
    #[arg(long)]
    pub clear_parent: bool,
    /// Replace tags with provided identifiers.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Remove all tags.
    #[arg(long)]
    pub clear_tags: bool,
    /// Replace followers with provided identifiers.
    #[arg(long = "follower", value_name = "USER")]
    pub followers: Vec<String>,
    /// Remove all followers.
    #[arg(long)]
    pub clear_followers: bool,
    /// Replace project associations.
    #[arg(long = "project", value_name = "PROJECT")]
    pub projects: Vec<String>,
    /// Remove all project associations.
    #[arg(long)]
    pub clear_projects: bool,
    /// Custom field updates in KEY=VALUE form.
    #[arg(long = "custom-field", value_name = "KEY=VALUE")]
    pub custom_fields: Vec<String>,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task delete`.
#[derive(Args, Debug)]
pub struct TaskDeleteArgs {
    /// Task identifier (gid).
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Skip confirmation prompt.
    #[arg(long)]
    pub force: bool,
}

/// Arguments for `task create-batch`.
#[derive(Args, Debug)]
pub struct TaskBatchCreateArgs {
    /// Path to JSON or CSV batch file.
    #[arg(long = "file", value_name = "PATH")]
    pub file: PathBuf,
    /// Override detected input format (`json` or `csv`).
    #[arg(long = "format", value_enum)]
    pub format: Option<BatchFormat>,
    /// Continue processing after an error.
    #[arg(long)]
    pub continue_on_error: bool,
    /// Output format for created tasks.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task update-batch`.
#[derive(Args, Debug)]
pub struct TaskBatchUpdateArgs {
    /// Path to JSON or CSV batch file.
    #[arg(long = "file", value_name = "PATH")]
    pub file: PathBuf,
    /// Override detected input format (`json` or `csv`).
    #[arg(long = "format", value_enum)]
    pub format: Option<BatchFormat>,
    /// Continue processing after an error.
    #[arg(long)]
    pub continue_on_error: bool,
    /// Output format for updated tasks.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task complete-batch`.
#[derive(Args, Debug)]
pub struct TaskBatchCompleteArgs {
    /// Path to JSON or CSV batch file.
    #[arg(long = "file", value_name = "PATH")]
    pub file: PathBuf,
    /// Override detected input format (`json` or `csv`).
    #[arg(long = "format", value_enum)]
    pub format: Option<BatchFormat>,
    /// Continue processing after an error.
    #[arg(long)]
    pub continue_on_error: bool,
    /// Output format for resulting tasks.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task search`.
#[derive(Args, Debug)]
pub struct TaskSearchArgs {
    /// Full-text search query.
    #[arg(long)]
    pub query: Option<String>,
    /// Workspace identifier (required).
    #[arg(long)]
    pub workspace: Option<String>,
    /// Filter by assignee (gid or email).
    #[arg(long)]
    pub assignee: Option<String>,
    /// Filter by project.
    #[arg(long = "project", value_name = "PROJECT")]
    pub projects: Vec<String>,
    /// Filter by section.
    #[arg(long = "section", value_name = "SECTION")]
    pub sections: Vec<String>,
    /// Filter by tag.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Filter by completion status.
    #[arg(long)]
    pub completed: Option<bool>,
    /// Include subtasks.
    #[arg(long)]
    pub include_subtasks: bool,
    /// Filter blocked tasks.
    #[arg(long)]
    pub blocked: Option<bool>,
    /// Filter tasks with attachments.
    #[arg(long)]
    pub has_attachments: Option<bool>,
    /// Created after date (YYYY-MM-DD).
    #[arg(long)]
    pub created_after: Option<String>,
    /// Created before date (YYYY-MM-DD).
    #[arg(long)]
    pub created_before: Option<String>,
    /// Modified after date (YYYY-MM-DD).
    #[arg(long)]
    pub modified_after: Option<String>,
    /// Modified before date (YYYY-MM-DD).
    #[arg(long)]
    pub modified_before: Option<String>,
    /// Due after date (YYYY-MM-DD).
    #[arg(long)]
    pub due_after: Option<String>,
    /// Due before date (YYYY-MM-DD).
    #[arg(long)]
    pub due_before: Option<String>,
    /// Sort by field (`modified_at`, likes, `created_at`).
    #[arg(long)]
    pub sort_by: Option<String>,
    /// Sort ascending (default: descending).
    #[arg(long)]
    pub sort_ascending: bool,
    /// Maximum number to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Only show recently accessed tasks (legacy mode).
    #[arg(long = "recent-only")]
    pub recent_only: bool,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Batch file format.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BatchFormat {
    /// JSON array of objects.
    Json,
    /// CSV file with headers.
    Csv,
}

const RECENT_TASKS_FILE: &str = "recent_tasks.json";
const RECENT_TASKS_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentTaskEntry {
    gid: String,
    name: String,
    last_accessed: String,
}

#[derive(Debug, Deserialize)]
struct BatchCreateRecord {
    name: String,
    workspace: Option<String>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    projects: Vec<String>,
    section: Option<String>,
    parent: Option<String>,
    assignee: Option<String>,
    due_on: Option<String>,
    due_at: Option<String>,
    start_on: Option<String>,
    start_at: Option<String>,
    notes: Option<String>,
    html_notes: Option<String>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    followers: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_map_field")]
    custom_fields: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct BatchUpdateRecord {
    task: String,
    name: Option<String>,
    notes: Option<String>,
    #[serde(default)]
    clear_notes: Option<bool>,
    html_notes: Option<String>,
    #[serde(default)]
    clear_html_notes: Option<bool>,
    completed: Option<bool>,
    assignee: Option<String>,
    #[serde(default)]
    clear_assignee: Option<bool>,
    due_on: Option<String>,
    #[serde(default)]
    clear_due_on: Option<bool>,
    due_at: Option<String>,
    #[serde(default)]
    clear_due_at: Option<bool>,
    start_on: Option<String>,
    #[serde(default)]
    clear_start_on: Option<bool>,
    start_at: Option<String>,
    #[serde(default)]
    clear_start_at: Option<bool>,
    parent: Option<String>,
    #[serde(default)]
    clear_parent: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    tags: Vec<String>,
    #[serde(default)]
    clear_tags: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    followers: Vec<String>,
    #[serde(default)]
    clear_followers: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_list_field")]
    projects: Vec<String>,
    #[serde(default)]
    clear_projects: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_map_field")]
    custom_fields: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
struct BatchCompleteRecord {
    task: String,
    #[serde(default = "true_by_default")]
    completed: bool,
}

/// Subcommands for `task subtasks`.
#[derive(Subcommand, Debug)]
pub enum TaskSubtasksCommand {
    /// List subtasks for a parent task.
    List(TaskSubtasksListArgs),
    /// Create a new subtask beneath a parent.
    Create(TaskSubtasksCreateArgs),
    /// Convert a task to a subtask or detach it.
    Convert(TaskSubtasksConvertArgs),
}

/// Arguments for `task subtasks list`.
#[derive(Args, Debug)]
pub struct TaskSubtasksListArgs {
    /// Parent task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Traverse subtasks recursively.
    #[arg(long)]
    pub recursive: bool,
    /// Additional fields to request.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task subtasks create`.
#[derive(Args, Debug)]
pub struct TaskSubtasksCreateArgs {
    /// Parent task identifier.
    #[arg(value_name = "TASK")]
    pub parent: String,
    /// Task name; required unless `--interactive`.
    #[arg(long)]
    pub name: Option<String>,
    /// Assignee identifier (gid or email).
    #[arg(long)]
    pub assignee: Option<String>,
    /// Due date (natural language accepted).
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    /// Due date/time (natural language accepted).
    #[arg(long = "due-at")]
    pub due_at: Option<String>,
    /// Start date (natural language accepted).
    #[arg(long = "start-on")]
    pub start_on: Option<String>,
    /// Start date/time (natural language accepted).
    #[arg(long = "start-at")]
    pub start_at: Option<String>,
    /// Tags to apply.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Followers to notify.
    #[arg(long = "follower", value_name = "USER")]
    pub followers: Vec<String>,
    /// Custom field assignments in KEY=VALUE form.
    #[arg(long = "custom-field", value_name = "KEY=VALUE")]
    pub custom_fields: Vec<String>,
    /// Prompt for missing values interactively.
    #[arg(long)]
    pub interactive: bool,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for `task subtasks convert`.
#[derive(Args, Debug)]
pub struct TaskSubtasksConvertArgs {
    /// Task identifier to convert.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// New parent task identifier.
    #[arg(long, conflicts_with = "root")]
    pub parent: Option<String>,
    /// Convert the task back to a top-level task.
    #[arg(long)]
    pub root: bool,
}

/// Subcommands for dependency management.
#[derive(Subcommand, Debug)]
pub enum TaskDependencyCommand {
    /// List dependencies.
    List(TaskDependencyListArgs),
    /// Add dependencies.
    Add(TaskDependencyModifyArgs),
    /// Remove dependencies.
    Remove(TaskDependencyModifyArgs),
}

/// Subcommands for dependent management.
#[derive(Subcommand, Debug)]
pub enum TaskDependentCommand {
    /// List dependents.
    List(TaskDependentListArgs),
    /// Add dependents.
    Add(TaskDependentModifyArgs),
    /// Remove dependents.
    Remove(TaskDependentModifyArgs),
}

/// Subcommands for project membership management.
#[derive(Subcommand, Debug)]
pub enum TaskProjectCommand {
    /// Add the task to a project.
    Add(TaskProjectAddArgs),
    /// Remove the task from a project.
    Remove(TaskProjectRemoveArgs),
}

/// Subcommands for follower management.
#[derive(Subcommand, Debug)]
pub enum TaskFollowerCommand {
    /// Add followers to the task.
    Add(TaskFollowerModifyArgs),
    /// Remove followers from the task.
    Remove(TaskFollowerModifyArgs),
}

/// Subcommands for tag management.
#[derive(Subcommand, Debug)]
pub enum TaskTagCommand {
    /// Add a tag to the task.
    Add(TaskTagModifyArgs),
    /// Remove a tag from the task.
    Remove(TaskTagModifyArgs),
    /// List tags on the task.
    List(TaskTagListArgs),
}

/// Arguments for dependency listing.
#[derive(Args, Debug)]
pub struct TaskDependencyListArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for dependency modifications.
#[derive(Args, Debug)]
pub struct TaskDependencyModifyArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Dependency identifiers to add/remove.
    #[arg(long = "dependency", value_name = "TASK", num_args = 1.., required = true)]
    pub dependencies: Vec<String>,
}

/// Arguments for dependent listing.
#[derive(Args, Debug)]
pub struct TaskDependentListArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<TaskOutputFormat>,
}

/// Arguments for dependent modifications.
#[derive(Args, Debug)]
pub struct TaskDependentModifyArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Dependent identifiers to add/remove.
    #[arg(long = "dependent", value_name = "TASK", num_args = 1.., required = true)]
    pub dependents: Vec<String>,
}

/// Arguments for project association (add).
#[derive(Args, Debug)]
pub struct TaskProjectAddArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Project identifier to add.
    #[arg(long)]
    pub project: String,
    /// Optional section identifier.
    #[arg(long)]
    pub section: Option<String>,
}

/// Arguments for project association removal.
#[derive(Args, Debug)]
pub struct TaskProjectRemoveArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Project identifier to remove.
    #[arg(long)]
    pub project: String,
}

/// Arguments for follower modifications.
#[derive(Args, Debug)]
pub struct TaskFollowerModifyArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Followers to add or remove.
    #[arg(long = "follower", value_name = "USER", num_args = 1.., required = true)]
    pub followers: Vec<String>,
}

/// Arguments for tag modification.
#[derive(Args, Debug)]
pub struct TaskTagModifyArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Tag identifier to add or remove.
    #[arg(long = "tag", value_name = "TAG")]
    pub tag: String,
}

/// Arguments for listing tags on a task.
#[derive(Args, Debug)]
pub struct TaskTagListArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: TaskOutputFormat,
}

/// Subcommands for comments (stories) management.
#[derive(Subcommand, Debug)]
pub enum TaskCommentsCommand {
    /// List comments on a task.
    List(TaskCommentsListArgs),
    /// Add a comment to a task.
    Add(TaskCommentsAddArgs),
    /// Show a specific comment.
    Show(TaskCommentsShowArgs),
    /// Update a comment.
    Update(TaskCommentsUpdateArgs),
    /// Delete a comment.
    Delete(TaskCommentsDeleteArgs),
}

/// Arguments for listing comments on a task.
#[derive(Args, Debug)]
pub struct TaskCommentsListArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Maximum number to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: TaskOutputFormat,
}

/// Arguments for adding a comment to a task.
#[derive(Args, Debug)]
pub struct TaskCommentsAddArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Comment text.
    #[arg(long)]
    pub text: String,
    /// Pin the comment.
    #[arg(long)]
    pub pin: bool,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TaskOutputFormat,
}

/// Arguments for showing a specific comment.
#[derive(Args, Debug)]
pub struct TaskCommentsShowArgs {
    /// Comment identifier.
    #[arg(value_name = "COMMENT")]
    pub comment: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TaskOutputFormat,
}

/// Arguments for updating a comment.
#[derive(Args, Debug)]
pub struct TaskCommentsUpdateArgs {
    /// Comment identifier.
    #[arg(value_name = "COMMENT")]
    pub comment: String,
    /// Updated text.
    #[arg(long)]
    pub text: Option<String>,
    /// Pin the comment.
    #[arg(long)]
    pub pin: bool,
    /// Unpin the comment.
    #[arg(long)]
    pub unpin: bool,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TaskOutputFormat,
}

/// Arguments for deleting a comment.
#[derive(Args, Debug)]
pub struct TaskCommentsDeleteArgs {
    /// Comment identifier.
    #[arg(value_name = "COMMENT")]
    pub comment: String,
    /// Skip confirmation.
    #[arg(long)]
    pub yes: bool,
}

/// Subcommands for attachments management.
#[derive(Subcommand, Debug)]
pub enum TaskAttachmentsCommand {
    /// List attachments on a task.
    List(TaskAttachmentsListArgs),
    /// Upload an attachment to a task.
    Upload(TaskAttachmentsUploadArgs),
    /// Download an attachment.
    Download(TaskAttachmentsDownloadArgs),
    /// Show attachment details.
    Show(TaskAttachmentsShowArgs),
    /// Delete an attachment.
    Delete(TaskAttachmentsDeleteArgs),
}

/// Arguments for listing attachments on a task.
#[derive(Args, Debug)]
pub struct TaskAttachmentsListArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Maximum number to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: TaskOutputFormat,
}

/// Arguments for uploading an attachment.
#[derive(Args, Debug)]
pub struct TaskAttachmentsUploadArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// File path to upload.
    #[arg(long)]
    pub file: PathBuf,
    /// Optional filename override.
    #[arg(long)]
    pub name: Option<String>,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TaskOutputFormat,
}

/// Arguments for downloading an attachment.
#[derive(Args, Debug)]
pub struct TaskAttachmentsDownloadArgs {
    /// Attachment identifier.
    #[arg(value_name = "ATTACHMENT")]
    pub attachment: String,
    /// Output file path.
    #[arg(long)]
    pub output: PathBuf,
}

/// Arguments for showing attachment details.
#[derive(Args, Debug)]
pub struct TaskAttachmentsShowArgs {
    /// Attachment identifier.
    #[arg(value_name = "ATTACHMENT")]
    pub attachment: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TaskOutputFormat,
}

/// Arguments for deleting an attachment.
#[derive(Args, Debug)]
pub struct TaskAttachmentsDeleteArgs {
    /// Attachment identifier.
    #[arg(value_name = "ATTACHMENT")]
    pub attachment: String,
    /// Skip confirmation.
    #[arg(long)]
    pub yes: bool,
}

/// Arguments for moving a task to a section.
#[derive(Args, Debug)]
pub struct TaskMoveToSectionArgs {
    /// Task identifier.
    #[arg(value_name = "TASK")]
    pub task: String,
    /// Section identifier to move the task to.
    #[arg(long)]
    pub section: String,
    /// Optional: insert task before this task gid.
    #[arg(long = "insert-before")]
    pub insert_before: Option<String>,
    /// Optional: insert task after this task gid.
    #[arg(long = "insert-after")]
    pub insert_after: Option<String>,
}

/// Parse and execute task commands.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn handle_task_command(command: TaskCommand, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialise async runtime")?;

    runtime.block_on(async move {
        match command {
            TaskCommand::List(args) => list_tasks_command(&client, config, args).await,
            TaskCommand::Show(args) => show_task_command(&client, config, args).await,
            TaskCommand::Create(args) => create_task_command(&client, config, args).await,
            TaskCommand::Update(args) => update_task_command(&client, config, args).await,
            TaskCommand::Delete(args) => delete_task_command(&client, args).await,
            TaskCommand::CreateBatch(args) => create_batch_command(&client, config, args).await,
            TaskCommand::UpdateBatch(args) => update_batch_command(&client, config, args).await,
            TaskCommand::CompleteBatch(args) => complete_batch_command(&client, config, args).await,
            TaskCommand::Search(args) => search_task_command(&client, config, args).await,
            TaskCommand::Subtasks { command } => {
                handle_subtasks_command(&client, config, command).await
            }
            TaskCommand::DependsOn { command } => {
                handle_dependencies_command(&client, command).await
            }
            TaskCommand::Blocks { command } => handle_dependents_command(&client, command).await,
            TaskCommand::Projects { command } => handle_projects_command(&client, command).await,
            TaskCommand::Followers { command } => handle_followers_command(&client, command).await,
            TaskCommand::Tags { command } => handle_tags_command(&client, command).await,
            TaskCommand::Comments { command } => handle_comments_command(&client, command).await,
            TaskCommand::Attachments { command } => {
                handle_attachments_command(&client, command).await
            }
            TaskCommand::MoveToSection(args) => move_to_section_command(&client, args).await,
        }
    })
}

async fn list_tasks_command(client: &ApiClient, config: &Config, args: TaskListArgs) -> Result<()> {
    let mut params = TaskListParams {
        workspace: args.workspace.clone().or_else(|| {
            config
                .default_workspace()
                .map(std::string::ToString::to_string)
        }),
        project: args.project.clone(),
        section: args.section.clone(),
        assignee: resolve_assignee(args.assignee.clone(), config, true),
        completed: args.completed,
        include_subtasks: args.include_subtasks,
        limit: args.limit,
        sort: parse_sort(args.sort.as_deref())?,
        ..Default::default()
    };

    if let Some(due_before) = args.due_before.as_ref() {
        params.due_before = Some(parse_date_input(due_before)?);
    }
    if let Some(due_after) = args.due_after.as_ref() {
        params.due_after = Some(parse_date_input(due_after)?);
    }
    params.fields.extend(args.fields.iter().cloned());

    debug!(?params, "listing tasks with params");

    let tasks = api::list_tasks(client, params).await?;
    let format = determine_output(args.output);
    let rendered = render_task_list(&tasks, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn show_task_command(client: &ApiClient, config: &Config, args: TaskShowArgs) -> Result<()> {
    let format = determine_output(args.output);
    let fields = if args.fields.is_empty() {
        Vec::new()
    } else {
        args.fields.clone()
    };

    let task = api::get_task(client, &args.task, fields).await?;
    let rendered = render_task_detail(&task, format, stdout().is_terminal())?;
    println!("{rendered}");

    if matches!(format, TaskOutputFormat::Table | TaskOutputFormat::Markdown) {
        if !task.dependencies.is_empty() {
            println!("\nDepends on:");
            println!("{}", format_task_refs(&task.dependencies));
        }
        if !task.dependents.is_empty() {
            println!("\nBlocks:");
            println!("{}", format_task_refs(&task.dependents));
        }

        let subtasks = api::list_subtasks(client, &task.gid, vec![]).await?;
        if !subtasks.is_empty() {
            println!("\nSubtasks:");
            let entries: Vec<(usize, Task)> =
                subtasks.into_iter().map(|child| (0usize, child)).collect();
            println!("{}", render_subtask_tree(&entries));
        }
    }

    if let Err(err) = record_recent_task(config, &task) {
        warn!(task = %task.gid, "failed to record recent task: {err:?}");
    }

    Ok(())
}

fn prompt_create_task_interactive(
    args: &mut TaskCreateArgs,
    config: &Config,
) -> Result<(String, Option<String>)> {
    let mut name = args.name.clone().unwrap_or_default();
    let mut workspace = args.workspace.clone().or_else(|| {
        config
            .default_workspace()
            .map(std::string::ToString::to_string)
    });

    if args.interactive {
        if name.trim().is_empty() {
            name = Input::new()
                .with_prompt("Task name")
                .interact_text()
                .context("failed to read task name")?;
        }
        if workspace
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            let value: String = Input::new()
                .with_prompt("Workspace gid")
                .interact_text()
                .context("failed to read workspace gid")?;
            workspace = Some(value);
        }
        if args.assignee.is_none() {
            let value: String = Input::new()
                .with_prompt("Assignee (gid or email, optional)")
                .allow_empty(true)
                .interact_text()
                .context("failed to read assignee")?;
            if !value.trim().is_empty() {
                args.assignee = Some(value);
            }
        }
        if args.due_on.is_none() {
            let value: String = Input::new()
                .with_prompt("Due date (optional, natural language accepted)")
                .allow_empty(true)
                .interact_text()
                .context("failed to read due date")?;
            if !value.trim().is_empty() {
                args.due_on = Some(value);
            }
        }
    }

    Ok((name, workspace))
}

async fn create_task_command(
    client: &ApiClient,
    config: &Config,
    mut args: TaskCreateArgs,
) -> Result<()> {
    if args.interactive {
        ensure_tty()?;
    }

    let (name, workspace) = prompt_create_task_interactive(&mut args, config)?;

    let mut builder = TaskCreateBuilder::new(name);
    if let Some(notes) = args.notes {
        builder = builder.notes(notes);
    }
    if let Some(html_notes) = args.html_notes {
        builder = builder.html_notes(html_notes);
    }
    if let Some(ws) = workspace {
        builder = builder.workspace(ws);
    }
    let resolved_assignee = resolve_assignee(args.assignee.clone(), config, false);
    for project in args.projects {
        builder = builder.project(project);
    }
    if let Some(section) = args.section {
        builder = builder.section(section);
    }
    if let Some(parent) = args.parent {
        builder = builder.parent(parent);
    }
    if let Some(assignee) = resolved_assignee {
        builder = builder.assignee(assignee);
    }
    if let Some(value) = args.due_on {
        builder = builder.due_on(parse_date_input(&value)?);
    }
    if let Some(value) = args.due_at {
        builder = builder.due_at(parse_datetime_input(&value)?);
    }
    if let Some(value) = args.start_on {
        builder = builder.start_on(parse_date_input(&value)?);
    }
    if let Some(value) = args.start_at {
        builder = builder.start_at(parse_datetime_input(&value)?);
    }
    for tag in args.tags {
        builder = builder.tag(tag);
    }
    for follower in args.followers {
        builder = builder.follower(follower);
    }
    for (field, value) in parse_custom_field_assignments(&args.custom_fields)? {
        builder = builder.custom_field(field, value);
    }

    let request = builder
        .build()
        .map_err(|err| map_validation_error(&err, "create"))?;
    let task = api::create_task(client, request).await?;
    let format = determine_output(args.output);
    let rendered = render_task_detail(&task, format, stdout().is_terminal())?;
    println!("{rendered}");
    if let Err(err) = record_recent_task(config, &task) {
        warn!(task = %task.gid, "failed to record recent task: {err:?}");
    }
    Ok(())
}

async fn update_task_command(
    client: &ApiClient,
    config: &Config,
    args: TaskUpdateArgs,
) -> Result<()> {
    let mut builder = TaskUpdateBuilder::new();

    if let Some(name) = args.name.as_ref() {
        builder = builder.name(name.clone());
    }
    if args.clear_notes {
        builder = builder.clear_notes();
    } else if let Some(notes) = args.notes.as_ref() {
        builder = builder.notes(notes.clone());
    }
    if args.clear_html_notes {
        builder = builder.clear_html_notes();
    } else if let Some(html) = args.html_notes.as_ref() {
        builder = builder.html_notes(html.clone());
    }
    if args.clear_assignee {
        builder = builder.clear_assignee();
    } else if let Some(assignee) = resolve_assignee(args.assignee.clone(), config, false) {
        builder = builder.assignee(assignee);
    }
    if args.complete && args.incomplete {
        bail!("--complete and --incomplete cannot be used together");
    } else if args.complete {
        builder = builder.completed(true);
    } else if args.incomplete {
        builder = builder.completed(false);
    }
    if args.clear_due_on {
        builder = builder.clear_due_on();
    } else if let Some(value) = args.due_on.as_ref() {
        builder = builder.due_on(parse_date_input(value)?);
    }
    if args.clear_due_at {
        builder = builder.clear_due_at();
    } else if let Some(value) = args.due_at.as_ref() {
        builder = builder.due_at(parse_datetime_input(value)?);
    }
    if args.clear_start_on {
        builder = builder.clear_start_on();
    } else if let Some(value) = args.start_on.as_ref() {
        builder = builder.start_on(parse_date_input(value)?);
    }
    if args.clear_start_at {
        builder = builder.clear_start_at();
    } else if let Some(value) = args.start_at.as_ref() {
        builder = builder.start_at(parse_datetime_input(value)?);
    }
    if args.clear_parent {
        builder = builder.clear_parent();
    } else if let Some(parent) = args.parent.as_ref() {
        builder = builder.parent(parent.clone());
    }
    if args.clear_tags {
        builder = builder.tags(Vec::<String>::new());
    } else if !args.tags.is_empty() {
        builder = builder.tags(args.tags.clone());
    }
    if args.clear_followers {
        builder = builder.followers(Vec::<String>::new());
    } else if !args.followers.is_empty() {
        builder = builder.followers(args.followers.clone());
    }
    if args.clear_projects {
        builder = builder.projects(Vec::<String>::new());
    } else if !args.projects.is_empty() {
        builder = builder.projects(args.projects.clone());
    }
    for (field, value) in parse_custom_field_assignments(&args.custom_fields)? {
        builder = builder.custom_field(field, value);
    }

    let request = builder
        .build()
        .map_err(|err| map_validation_error(&err, "update"))?;
    let task = api::update_task(client, &args.task, request).await?;
    let format = determine_output(args.output);
    let rendered = render_task_detail(&task, format, stdout().is_terminal())?;
    println!("{rendered}");
    if let Err(err) = record_recent_task(config, &task) {
        warn!(task = %task.gid, "failed to record recent task: {err:?}");
    }
    Ok(())
}

async fn delete_task_command(client: &ApiClient, args: TaskDeleteArgs) -> Result<()> {
    if !args.force {
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete task {}?", args.task))
            .default(false)
            .interact()
            .context("failed to read confirmation")?;
        if !confirmed {
            println!("Deletion aborted.");
            return Ok(());
        }
    }

    api::delete_task(client, &args.task).await?;
    println!("Deleted task {}.", args.task);
    Ok(())
}

async fn create_batch_command(
    client: &ApiClient,
    config: &Config,
    args: TaskBatchCreateArgs,
) -> Result<()> {
    let format = args.format.unwrap_or(detect_batch_format(&args.file)?);
    let records: Vec<BatchCreateRecord> = load_batch_records(&args.file, format)?;
    if records.is_empty() {
        println!("No records found in batch file.");
        return Ok(());
    }

    let total = records.len();
    let mut created = Vec::new();
    for (index, record) in records.into_iter().enumerate() {
        if stdout().is_terminal() {
            println!("[{}/{}] creating {}", index + 1, total, record.name);
        }

        let request = match build_create_request(&record, config) {
            Ok(request) => request,
            Err(err) => {
                if args.continue_on_error {
                    warn!(index, "failed to build create payload: {err:?}");
                    continue;
                }
                return Err(err);
            }
        };

        match api::create_task(client, request).await {
            Ok(task) => {
                if let Err(err) = record_recent_task(config, &task) {
                    warn!(task = %task.gid, "failed to record recent task: {err:?}");
                }
                created.push(task);
            }
            Err(err) => {
                let err = anyhow::Error::new(err);
                if args.continue_on_error {
                    warn!(index, "batch create failed: {err:?}");
                    continue;
                }
                return Err(err);
            }
        }
    }

    if created.is_empty() {
        println!("No tasks created.");
        return Ok(());
    }

    let format = determine_output(args.output);
    let rendered = render_task_list(&created, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn update_batch_command(
    client: &ApiClient,
    config: &Config,
    args: TaskBatchUpdateArgs,
) -> Result<()> {
    let format = args.format.unwrap_or(detect_batch_format(&args.file)?);
    let records: Vec<BatchUpdateRecord> = load_batch_records(&args.file, format)?;
    if records.is_empty() {
        println!("No records found in batch file.");
        return Ok(());
    }

    let total = records.len();
    let mut updated = Vec::new();
    for (index, record) in records.into_iter().enumerate() {
        if stdout().is_terminal() {
            println!("[{}/{}] updating {}", index + 1, total, record.task);
        }

        let request = match build_update_request(&record, config) {
            Ok(request) => request,
            Err(err) => {
                if args.continue_on_error {
                    warn!(index, "failed to build update payload: {err:?}");
                    continue;
                }
                return Err(err);
            }
        };

        match api::update_task(client, &record.task, request).await {
            Ok(task) => {
                if let Err(err) = record_recent_task(config, &task) {
                    warn!(task = %task.gid, "failed to record recent task: {err:?}");
                }
                updated.push(task);
            }
            Err(err) => {
                let err = anyhow::Error::new(err);
                if args.continue_on_error {
                    warn!(index, "batch update failed: {err:?}");
                    continue;
                }
                return Err(err);
            }
        }
    }

    if updated.is_empty() {
        println!("No tasks updated.");
        return Ok(());
    }

    let format = determine_output(args.output);
    let rendered = render_task_list(&updated, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn complete_batch_command(
    client: &ApiClient,
    config: &Config,
    args: TaskBatchCompleteArgs,
) -> Result<()> {
    let format = args.format.unwrap_or(detect_batch_format(&args.file)?);
    let records: Vec<BatchCompleteRecord> = load_batch_records(&args.file, format)?;
    if records.is_empty() {
        println!("No records found in batch file.");
        return Ok(());
    }

    let total = records.len();
    let mut completed = Vec::new();
    for (index, record) in records.into_iter().enumerate() {
        if stdout().is_terminal() {
            println!("[{}/{}] completing {}", index + 1, total, record.task);
        }

        let request = TaskUpdateBuilder::new()
            .completed(record.completed)
            .build()
            .map_err(|err| map_validation_error(&err, "complete task"))?;

        match api::update_task(client, &record.task, request).await {
            Ok(task) => {
                if let Err(err) = record_recent_task(config, &task) {
                    warn!(task = %task.gid, "failed to record recent task: {err:?}");
                }
                completed.push(task);
            }
            Err(err) => {
                let err = anyhow::Error::new(err);
                if args.continue_on_error {
                    warn!(index, "batch completion failed: {err:?}");
                    continue;
                }
                return Err(err);
            }
        }
    }

    if completed.is_empty() {
        println!("No tasks updated.");
        return Ok(());
    }

    let format = determine_output(args.output);
    let rendered = render_task_list(&completed, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn search_task_command(
    client: &ApiClient,
    config: &Config,
    args: TaskSearchArgs,
) -> Result<()> {
    // Handle legacy recent-only mode
    if args.recent_only {
        let recent_entries = load_recent_task_entries(config)?;
        if recent_entries.is_empty() {
            println!("No recent tasks recorded.");
            return Ok(());
        }
        let tasks: Vec<Task> = recent_entries.iter().map(recent_entry_to_task).collect();
        let format = determine_output(args.output);
        let rendered = render_task_list(&tasks, format, stdout().is_terminal())?;
        println!("{rendered}");
        return Ok(());
    }

    // Get workspace
    let workspace = args
        .workspace
        .clone()
        .or_else(|| config.default_workspace().map(ToString::to_string))
        .context("workspace is required; provide --workspace or set default_workspace in config")?;

    // Build search params
    let params = TaskSearchParams {
        workspace,
        text: args.query.clone(),
        resource_subtype: None,
        completed: args.completed,
        is_subtask: if args.include_subtasks {
            None
        } else {
            Some(false)
        },
        is_blocked: args.blocked,
        has_attachment: args.has_attachments,
        assignee: resolve_assignee(args.assignee.clone(), config, true),
        projects: args.projects.clone(),
        sections: args.sections.clone(),
        tags: args.tags.clone(),
        created_after: args.created_after.clone(),
        created_before: args.created_before.clone(),
        modified_after: args.modified_after.clone(),
        modified_before: args.modified_before.clone(),
        due_after: args.due_after.clone(),
        due_before: args.due_before.clone(),
        sort_by: args.sort_by.clone(),
        sort_ascending: args.sort_ascending,
        limit: args.limit,
        fields: BTreeSet::new(),
    };

    // Execute search
    let tasks = api::search_tasks(client, params).await?;

    if tasks.is_empty() {
        if let Some(ref query) = args.query {
            println!("No tasks matched '{query}'.");
        } else {
            println!("No tasks found.");
        }
        return Ok(());
    }

    // Record recent tasks for fuzzy selection
    for task in &tasks {
        if let Err(err) = record_recent_task(config, task) {
            warn!(task = %task.gid, "failed to record recent task: {err:?}");
        }
    }

    // Display results
    let format = determine_output(args.output);
    let rendered = render_task_list(&tasks, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn handle_subtasks_command(
    client: &ApiClient,
    config: &Config,
    command: TaskSubtasksCommand,
) -> Result<()> {
    match command {
        TaskSubtasksCommand::List(args) => subtasks_list_command(client, args).await,
        TaskSubtasksCommand::Create(args) => subtasks_create_command(client, config, args).await,
        TaskSubtasksCommand::Convert(args) => subtasks_convert_command(client, config, args).await,
    }
}

async fn handle_dependencies_command(
    client: &ApiClient,
    command: TaskDependencyCommand,
) -> Result<()> {
    match command {
        TaskDependencyCommand::List(args) => {
            let refs = api::list_dependencies(client, &args.task).await?;
            output_task_refs(refs, determine_output(args.output));
            Ok(())
        }
        TaskDependencyCommand::Add(args) => {
            api::add_dependencies(client, &args.task, args.dependencies.clone()).await?;
            println!(
                "Added {} dependenc{} to {}.",
                args.dependencies.len(),
                if args.dependencies.len() == 1 {
                    "y"
                } else {
                    "ies"
                },
                args.task
            );
            Ok(())
        }
        TaskDependencyCommand::Remove(args) => {
            api::remove_dependencies(client, &args.task, args.dependencies.clone()).await?;
            println!(
                "Removed {} dependenc{} from {}.",
                args.dependencies.len(),
                if args.dependencies.len() == 1 {
                    "y"
                } else {
                    "ies"
                },
                args.task
            );
            Ok(())
        }
    }
}

async fn handle_dependents_command(
    client: &ApiClient,
    command: TaskDependentCommand,
) -> Result<()> {
    match command {
        TaskDependentCommand::List(args) => {
            let refs = api::list_dependents(client, &args.task).await?;
            output_task_refs(refs, determine_output(args.output));
            Ok(())
        }
        TaskDependentCommand::Add(args) => {
            api::add_dependents(client, &args.task, args.dependents.clone()).await?;
            println!(
                "Marked {} task{} as blocked by {}.",
                args.dependents.len(),
                if args.dependents.len() == 1 { "" } else { "s" },
                args.task
            );
            Ok(())
        }
        TaskDependentCommand::Remove(args) => {
            api::remove_dependents(client, &args.task, args.dependents.clone()).await?;
            println!(
                "Removed {} dependent{} from {}.",
                args.dependents.len(),
                if args.dependents.len() == 1 { "" } else { "s" },
                args.task
            );
            Ok(())
        }
    }
}

async fn handle_projects_command(client: &ApiClient, command: TaskProjectCommand) -> Result<()> {
    match command {
        TaskProjectCommand::Add(args) => {
            api::add_project(
                client,
                &args.task,
                args.project.clone(),
                args.section.clone(),
            )
            .await?;
            if let Some(section) = args.section {
                println!(
                    "Added task {} to project {} (section {}).",
                    args.task, args.project, section
                );
            } else {
                println!("Added task {} to project {}.", args.task, args.project);
            }
            Ok(())
        }
        TaskProjectCommand::Remove(args) => {
            api::remove_project(client, &args.task, args.project.clone()).await?;
            println!("Removed task {} from project {}.", args.task, args.project);
            Ok(())
        }
    }
}

async fn handle_followers_command(client: &ApiClient, command: TaskFollowerCommand) -> Result<()> {
    match command {
        TaskFollowerCommand::Add(args) => {
            api::add_followers(client, &args.task, args.followers.clone()).await?;
            println!(
                "Added {} follower{} to {}.",
                args.followers.len(),
                if args.followers.len() == 1 { "" } else { "s" },
                args.task
            );
            Ok(())
        }
        TaskFollowerCommand::Remove(args) => {
            api::remove_followers(client, &args.task, args.followers.clone()).await?;
            println!(
                "Removed {} follower{} from {}.",
                args.followers.len(),
                if args.followers.len() == 1 { "" } else { "s" },
                args.task
            );
            Ok(())
        }
    }
}

async fn handle_tags_command(client: &ApiClient, command: TaskTagCommand) -> Result<()> {
    match command {
        TaskTagCommand::Add(args) => {
            api::add_tag(client, &args.task, args.tag.clone()).await?;
            println!("Added tag {} to task {}.", args.tag, args.task);
            Ok(())
        }
        TaskTagCommand::Remove(args) => {
            api::remove_tag(client, &args.task, args.tag.clone()).await?;
            println!("Removed tag {} from task {}.", args.tag, args.task);
            Ok(())
        }
        TaskTagCommand::List(args) => {
            let task = api::get_task(client, &args.task, vec![]).await?;

            if task.tags.is_empty() {
                println!("No tags on task {}.", args.task);
                return Ok(());
            }

            match args.format {
                TaskOutputFormat::Table => {
                    if stdout().is_terminal() {
                        println!("{:<20} {}", "GID".bold(), "Name".bold());
                        println!("{}", "─".repeat(50));
                    }
                    for tag in &task.tags {
                        if stdout().is_terminal() {
                            println!("{:<20} {}", tag.gid, tag.label());
                        } else {
                            println!("{}\t{}", tag.gid, tag.label());
                        }
                    }
                    if stdout().is_terminal() {
                        println!("\n{} tags listed.", task.tags.len());
                    }
                }
                TaskOutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&task.tags)
                        .context("failed to serialize tags to JSON")?;
                    println!("{json}");
                }
                _ => {
                    for tag in &task.tags {
                        println!("{}: {}", tag.gid, tag.label());
                    }
                }
            }
            Ok(())
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn handle_comments_command(client: &ApiClient, command: TaskCommentsCommand) -> Result<()> {
    match command {
        TaskCommentsCommand::List(args) => {
            let params = StoryListParams {
                task_gid: args.task.clone(),
                limit: args.limit,
            };
            let stories = api::list_stories(client, params).await?;

            if stories.is_empty() {
                println!("No comments on task {}.", args.task);
                return Ok(());
            }

            match args.format {
                TaskOutputFormat::Table => {
                    if stdout().is_terminal() {
                        println!(
                            "{:<20} {:<10} {:<15} {}",
                            "GID".bold(),
                            "Type".bold(),
                            "Author".bold(),
                            "Text".bold()
                        );
                        println!("{}", "─".repeat(80));
                    }
                    for story in &stories {
                        let author = story
                            .created_by
                            .as_ref()
                            .and_then(|u| u.name.as_deref())
                            .unwrap_or("Unknown");
                        let text = story.text.as_deref().unwrap_or("");
                        let text_preview = if text.len() > 50 {
                            format!("{}...", &text[..50])
                        } else {
                            text.to_string()
                        };

                        if stdout().is_terminal() {
                            let pin_marker = if story.is_pinned { " 📌" } else { "" };
                            println!(
                                "{:<20} {:<10} {:<15} {}{}",
                                story.gid,
                                format!("{:?}", story.story_type),
                                author,
                                text_preview,
                                pin_marker
                            );
                        } else {
                            println!(
                                "{}\t{:?}\t{}\t{}",
                                story.gid, story.story_type, author, text_preview
                            );
                        }
                    }
                    if stdout().is_terminal() {
                        println!("\n{} comments listed.", stories.len());
                    }
                }
                TaskOutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&stories)
                        .context("failed to serialize comments to JSON")?;
                    println!("{json}");
                }
                _ => {
                    for story in &stories {
                        let author = story
                            .created_by
                            .as_ref()
                            .and_then(|u| u.name.as_deref())
                            .unwrap_or("Unknown");
                        let text = story.text.as_deref().unwrap_or("");
                        println!("Comment {}: [{}] {}", story.gid, author, text);
                    }
                }
            }
            Ok(())
        }
        TaskCommentsCommand::Add(args) => {
            let request = StoryCreateBuilder::new(&args.text)
                .pinned(args.pin)
                .build()
                .context("failed to build story create request")?;

            let story = api::create_story(client, &args.task, request).await?;

            if args.format == TaskOutputFormat::Json {
                let json = serde_json::to_string_pretty(&story)
                    .context("failed to serialize comment to JSON")?;
                println!("{json}");
            } else {
                println!("Added comment {} to task {}.", story.gid, args.task);
                if story.is_pinned {
                    println!("Comment is pinned.");
                }
            }
            Ok(())
        }
        TaskCommentsCommand::Show(args) => {
            let story = api::get_story(client, &args.comment).await?;

            if args.format == TaskOutputFormat::Json {
                let json = serde_json::to_string_pretty(&story)
                    .context("failed to serialize comment to JSON")?;
                println!("{json}");
            } else {
                let gid = &story.gid;
                println!("Comment: {gid}");
                println!("Type: {:?}", story.story_type);
                if let Some(ref author) = story.created_by
                    && let Some(ref name) = author.name
                {
                    println!("Author: {name}");
                }
                if let Some(ref created_at) = story.created_at {
                    println!("Created: {created_at}");
                }
                if story.is_pinned {
                    println!("Pinned: Yes");
                }
                if story.is_edited {
                    println!("Edited: Yes");
                }
                if let Some(ref text) = story.text {
                    println!("\nText:\n{text}");
                }
            }
            Ok(())
        }
        TaskCommentsCommand::Update(args) => {
            if args.pin && args.unpin {
                bail!("Cannot specify both --pin and --unpin");
            }

            let mut builder = StoryUpdateBuilder::new();

            if let Some(text) = args.text {
                builder = builder.text(text);
            }

            if args.pin {
                builder = builder.pinned(true);
            } else if args.unpin {
                builder = builder.pinned(false);
            }

            let request = builder
                .build()
                .context("failed to build story update request")?;

            let story = api::update_story(client, &args.comment, request).await?;

            if args.format == TaskOutputFormat::Json {
                let json = serde_json::to_string_pretty(&story)
                    .context("failed to serialize comment to JSON")?;
                println!("{json}");
            } else {
                let gid = &story.gid;
                println!("Updated comment {gid}.");
            }
            Ok(())
        }
        TaskCommentsCommand::Delete(args) => {
            if !args.yes {
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Delete comment {}?", args.comment))
                    .default(false)
                    .interact()?;

                if !confirm {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            api::delete_story(client, &args.comment).await?;
            println!("Deleted comment {}.", args.comment);
            Ok(())
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn handle_attachments_command(
    client: &ApiClient,
    command: TaskAttachmentsCommand,
) -> Result<()> {
    match command {
        TaskAttachmentsCommand::List(args) => {
            let params = AttachmentListParams {
                task_gid: args.task.clone(),
                limit: args.limit,
            };
            let attachments = api::list_attachments(client, params).await?;

            if attachments.is_empty() {
                println!("No attachments on task {}.", args.task);
                return Ok(());
            }

            match args.format {
                TaskOutputFormat::Table => {
                    if stdout().is_terminal() {
                        println!(
                            "{:<20} {:<40} {:<10} {}",
                            "GID".bold(),
                            "Name".bold(),
                            "Size".bold(),
                            "Created".bold()
                        );
                        println!("{}", "─".repeat(90));
                    }
                    for attachment in &attachments {
                        let size = attachment
                            .size
                            .map_or_else(|| "N/A".to_string(), format_bytes);
                        let created = attachment.created_at.as_deref().unwrap_or("N/A");

                        if stdout().is_terminal() {
                            println!(
                                "{:<20} {:<40} {:<10} {}",
                                attachment.gid, attachment.name, size, created
                            );
                        } else {
                            println!(
                                "{}\t{}\t{}\t{}",
                                attachment.gid, attachment.name, size, created
                            );
                        }
                    }
                    if stdout().is_terminal() {
                        println!("\n{} attachments listed.", attachments.len());
                    }
                }
                TaskOutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&attachments)
                        .context("failed to serialize attachments to JSON")?;
                    println!("{json}");
                }
                _ => {
                    for attachment in &attachments {
                        println!("{}: {}", attachment.gid, attachment.name);
                    }
                }
            }
            Ok(())
        }
        TaskAttachmentsCommand::Upload(args) => {
            let params = AttachmentUploadParams {
                task_gid: args.task.clone(),
                file_path: args.file.clone(),
                name: args.name.clone(),
            };

            println!("Uploading {}...", args.file.display());
            let attachment = api::upload_attachment(client, params).await?;

            if args.format == TaskOutputFormat::Json {
                let json = serde_json::to_string_pretty(&attachment)
                    .context("failed to serialize attachment to JSON")?;
                println!("{json}");
            } else {
                println!(
                    "Uploaded attachment {} to task {}.",
                    attachment.gid, args.task
                );
                println!("Name: {}", attachment.name);
                if let Some(size) = attachment.size {
                    println!("Size: {}", format_bytes(size));
                }
            }
            Ok(())
        }
        TaskAttachmentsCommand::Download(args) => {
            println!("Downloading attachment {}...", args.attachment);
            api::download_attachment(client, &args.attachment, &args.output).await?;
            println!("Downloaded to {}", args.output.display());
            Ok(())
        }
        TaskAttachmentsCommand::Show(args) => {
            let attachment = api::get_attachment(client, &args.attachment).await?;

            if args.format == TaskOutputFormat::Json {
                let json = serde_json::to_string_pretty(&attachment)
                    .context("failed to serialize attachment to JSON")?;
                println!("{json}");
            } else {
                let gid = &attachment.gid;
                let name = &attachment.name;
                println!("GID: {gid}");
                println!("Name: {name}");
                if let Some(size) = attachment.size {
                    println!("Size: {}", format_bytes(size));
                }
                if let Some(ref created) = attachment.created_at {
                    println!("Created: {created}");
                }
                if let Some(ref host) = attachment.host {
                    println!("Host: {host}");
                }
                if let Some(ref url) = attachment.download_url {
                    println!("Download URL: {url}");
                }
                if let Some(ref url) = attachment.permanent_url {
                    println!("Permanent URL: {url}");
                }
            }
            Ok(())
        }
        TaskAttachmentsCommand::Delete(args) => {
            if !args.yes {
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Delete attachment {}?", args.attachment))
                    .default(false)
                    .interact()?;

                if !confirm {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            api::delete_attachment(client, &args.attachment).await?;
            println!("Deleted attachment {}.", args.attachment);
            Ok(())
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

async fn move_to_section_command(client: &ApiClient, args: TaskMoveToSectionArgs) -> Result<()> {
    api::add_task_to_section(
        client,
        &args.section,
        args.task.clone(),
        args.insert_before,
        args.insert_after,
    )
    .await?;
    println!("Moved task {} to section {}.", args.task, args.section);
    Ok(())
}

async fn subtasks_list_command(client: &ApiClient, args: TaskSubtasksListArgs) -> Result<()> {
    let fields = args.fields.clone();
    let entries = collect_subtasks(client, &args.task, args.recursive, 0, &fields).await?;
    if entries.is_empty() {
        println!("No subtasks found.");
        return Ok(());
    }

    let format = determine_output(args.output);
    match format {
        TaskOutputFormat::Json => {
            let payload: Vec<_> = entries
                .iter()
                .map(|(depth, task)| {
                    json!({
                        "depth": depth,
                        "task": task,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        TaskOutputFormat::Csv => {
            let tasks = tasks_with_indent(&entries);
            let rendered = render_task_list(&tasks, TaskOutputFormat::Csv, stdout().is_terminal())?;
            println!("{rendered}");
        }
        TaskOutputFormat::Markdown => {
            let tasks = tasks_with_indent(&entries);
            let rendered =
                render_task_list(&tasks, TaskOutputFormat::Markdown, stdout().is_terminal())?;
            println!("{rendered}");
        }
        TaskOutputFormat::Table => {
            let tasks = tasks_with_indent(&entries);
            let rendered =
                render_task_list(&tasks, TaskOutputFormat::Table, stdout().is_terminal())?;
            println!("{rendered}");
        }
    }

    Ok(())
}

async fn subtasks_create_command(
    client: &ApiClient,
    config: &Config,
    args: TaskSubtasksCreateArgs,
) -> Result<()> {
    if args.interactive {
        ensure_tty()?;
    }

    let mut name = args.name.clone().unwrap_or_default();
    if args.interactive && name.trim().is_empty() {
        name = Input::new()
            .with_prompt("Subtask name")
            .interact_text()
            .context("failed to read task name")?;
    }

    if name.trim().is_empty() {
        bail!("task name is required to create a subtask");
    }

    let mut builder = TaskCreateBuilder::new(name).parent(args.parent.clone());
    if let Some(assignee) = resolve_assignee(args.assignee.clone(), config, false) {
        builder = builder.assignee(assignee);
    }
    if let Some(value) = args.due_on {
        builder = builder.due_on(parse_date_input(&value)?);
    }
    if let Some(value) = args.due_at {
        builder = builder.due_at(parse_datetime_input(&value)?);
    }
    if let Some(value) = args.start_on {
        builder = builder.start_on(parse_date_input(&value)?);
    }
    if let Some(value) = args.start_at {
        builder = builder.start_at(parse_datetime_input(&value)?);
    }
    for tag in args.tags {
        builder = builder.tag(tag);
    }
    for follower in args.followers {
        builder = builder.follower(follower);
    }
    for (field, value) in parse_custom_field_assignments(&args.custom_fields)? {
        builder = builder.custom_field(field, value);
    }

    let request = builder
        .build()
        .map_err(|err| map_validation_error(&err, "create subtask"))?;
    let task = api::create_task(client, request).await?;
    let format = determine_output(args.output);
    let rendered = render_task_detail(&task, format, stdout().is_terminal())?;
    println!("{rendered}");
    if let Err(err) = record_recent_task(config, &task) {
        warn!(task = %task.gid, "failed to record recent task: {err:?}");
    }
    Ok(())
}

async fn subtasks_convert_command(
    client: &ApiClient,
    config: &Config,
    args: TaskSubtasksConvertArgs,
) -> Result<()> {
    if !args.root && args.parent.is_none() {
        bail!("provide --parent <gid> to convert to a subtask or --root to detach");
    }

    let mut builder = TaskUpdateBuilder::new();
    if args.root {
        builder = builder.clear_parent();
    } else if let Some(parent) = args.parent.clone() {
        builder = builder.parent(parent);
    }

    let request = builder
        .build()
        .map_err(|err| map_validation_error(&err, "convert subtask"))?;
    let task = api::update_task(client, &args.task, request).await?;
    println!(
        "Task {} converted {}.",
        task.gid,
        if args.root {
            "to top-level"
        } else {
            "to subtask"
        }
    );
    if let Err(err) = record_recent_task(config, &task) {
        warn!(task = %task.gid, "failed to record recent task: {err:?}");
    }
    Ok(())
}

async fn collect_subtasks(
    client: &ApiClient,
    task_gid: &str,
    recursive: bool,
    depth: usize,
    fields: &[String],
) -> Result<Vec<(usize, Task)>> {
    let mut results = Vec::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((task_gid.to_string(), depth));

    while let Some((parent_gid, level)) = queue.pop_front() {
        let subtasks = api::list_subtasks(client, &parent_gid, fields.to_vec())
            .await
            .map_err(|err| anyhow!(err))?;
        for task in subtasks {
            let gid = task.gid.clone();
            results.push((level, task));
            if recursive {
                queue.push_back((gid, level + 1));
            }
        }
    }

    Ok(results)
}

fn tasks_with_indent(entries: &[(usize, Task)]) -> Vec<Task> {
    entries
        .iter()
        .map(|(depth, task)| {
            let mut clone = task.clone();
            let prefix = "  ".repeat(*depth);
            clone.name = format!("{prefix}{}", clone.name);
            clone
        })
        .collect()
}

fn render_subtask_tree(entries: &[(usize, Task)]) -> String {
    let mut output = String::new();
    for (depth, task) in entries {
        let indent = "  ".repeat(*depth);
        let status = if task.completed { "[x]" } else { "[ ]" };
        let _ = writeln!(&mut output, "{indent}{status} {} ({})", task.name, task.gid);
    }
    if output.is_empty() {
        "No subtasks found.".into()
    } else {
        output
    }
}

fn output_task_refs(refs: Vec<TaskReference>, format: TaskOutputFormat) {
    if refs.is_empty() {
        println!("None.");
        return;
    }
    match format {
        TaskOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&refs).unwrap());
        }
        TaskOutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            for reference in refs {
                let name = reference.name.unwrap_or_else(|| reference.gid.clone());
                let record = [reference.gid, name];
                wtr.write_record(record).unwrap();
            }
            let bytes = wtr.into_inner().unwrap();
            println!("{}", String::from_utf8(bytes).unwrap());
        }
        TaskOutputFormat::Markdown | TaskOutputFormat::Table => {
            println!("{}", format_task_refs(&refs));
        }
    }
}

fn format_task_refs(refs: &[TaskReference]) -> String {
    let mut output = String::new();
    for reference in refs {
        let label = reference
            .name
            .clone()
            .unwrap_or_else(|| reference.gid.clone());
        let _ = writeln!(&mut output, "- {} ({})", label, reference.gid);
    }
    output
}

fn determine_output(value: Option<TaskOutputFormat>) -> TaskOutputFormat {
    value.unwrap_or_else(|| {
        if stdout().is_terminal() {
            TaskOutputFormat::Table
        } else {
            TaskOutputFormat::Json
        }
    })
}

fn ensure_tty() -> Result<()> {
    if !stdout().is_terminal() {
        bail!("interactive mode requires an interactive terminal");
    }
    Ok(())
}

fn parse_sort(value: Option<&str>) -> Result<Option<TaskSort>> {
    match value {
        None => Ok(None),
        Some("name") => Ok(Some(TaskSort::Name)),
        Some("due" | "due_on") => Ok(Some(TaskSort::DueOn)),
        Some("created" | "created_at") => Ok(Some(TaskSort::CreatedAt)),
        Some("modified" | "modified_at") => Ok(Some(TaskSort::ModifiedAt)),
        Some("assignee") => Ok(Some(TaskSort::Assignee)),
        Some(other) => Err(anyhow!(
            "unsupported sort value '{other}'; expected name, due_on, created_at, modified_at, or assignee"
        )),
    }
}

fn parse_custom_field_assignments(entries: &[String]) -> Result<Vec<(String, CustomFieldValue)>> {
    let mut assignments = Vec::new();
    for entry in entries {
        let (raw_key, raw_value) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid custom field '{entry}'; expected KEY=VALUE"))?;
        let key = raw_key.trim().to_string();
        let parsed = serde_json::from_str::<Value>(raw_value)
            .unwrap_or_else(|_| Value::String(raw_value.to_string()));
        let value = match parsed {
            Value::String(text) => CustomFieldValue::Text(text),
            Value::Number(number) => CustomFieldValue::Number(
                number
                    .as_f64()
                    .ok_or_else(|| anyhow!("custom field '{key}' contains non-finite number"))?,
            ),
            Value::Bool(flag) => CustomFieldValue::Bool(flag),
            Value::Array(values) => {
                if values.iter().all(Value::is_string) {
                    let options = values
                        .into_iter()
                        .filter_map(|value| value.as_str().map(str::to_string))
                        .collect();
                    CustomFieldValue::MultiEnum(options)
                } else {
                    CustomFieldValue::Json(Value::Array(values))
                }
            }
            Value::Null => CustomFieldValue::Json(Value::Null),
            Value::Object(map) => CustomFieldValue::Json(Value::Object(map)),
        };
        assignments.push((key, value));
    }
    Ok(assignments)
}

fn parse_date_input(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(date.format("%Y-%m-%d").to_string());
    }
    let parsed = dateparser::parse_with_timezone(trimmed, &chrono::Utc)
        .map_err(|err| anyhow!("failed to parse date '{trimmed}': {err}"))?;
    Ok(parsed.date_naive().format("%Y-%m-%d").to_string())
}

fn parse_datetime_input(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(dt.to_rfc3339());
    }
    let parsed = dateparser::parse_with_timezone(trimmed, &chrono::Utc)
        .map_err(|err| anyhow!("failed to parse date/time '{trimmed}': {err}"))?;
    Ok(parsed.to_rfc3339())
}

fn map_validation_error(err: &TaskValidationError, context: &str) -> anyhow::Error {
    match err {
        TaskValidationError::MissingName => anyhow!("task name is required to {context}"),
        TaskValidationError::MissingScope => {
            anyhow!("either --workspace or at least one --project must be provided to {context}")
        }
        TaskValidationError::EmptyUpdate => anyhow!("no fields were updated"),
    }
}

fn resolve_assignee(input: Option<String>, config: &Config, fallback_me: bool) -> Option<String> {
    input.map_or_else(
        || {
            config
                .default_assignee()
                .map(std::string::ToString::to_string)
                .or_else(|| {
                    if fallback_me {
                        Some("me".to_string())
                    } else {
                        None
                    }
                })
        },
        |value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.eq_ignore_ascii_case("me") {
                config
                    .default_assignee()
                    .map(std::string::ToString::to_string)
                    .or_else(|| Some("me".to_string()))
            } else {
                Some(trimmed.to_string())
            }
        },
    )
}

fn record_recent_task(config: &Config, task: &Task) -> Result<()> {
    let mut entries = load_recent_task_entries(config)?;
    entries.retain(|entry| entry.gid != task.gid);
    entries.insert(
        0,
        RecentTaskEntry {
            gid: task.gid.clone(),
            name: task.name.clone(),
            last_accessed: Utc::now().to_rfc3339(),
        },
    );
    if entries.len() > RECENT_TASKS_LIMIT {
        entries.truncate(RECENT_TASKS_LIMIT);
    }

    let path = recent_tasks_path(config);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create recent tasks directory {}",
                parent.display()
            )
        })?;
    }
    let serialized =
        serde_json::to_string_pretty(&entries).context("failed to serialize recent tasks cache")?;
    fs::write(&path, serialized)
        .with_context(|| format!("failed to write recent tasks cache {}", path.display()))?;
    Ok(())
}

fn load_recent_task_entries(config: &Config) -> Result<Vec<RecentTaskEntry>> {
    let path = recent_tasks_path(config);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read recent tasks cache {}", path.display()))?;
    let mut entries: Vec<RecentTaskEntry> = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse recent tasks cache {}", path.display()))?;
    entries.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    if entries.len() > RECENT_TASKS_LIMIT {
        entries.truncate(RECENT_TASKS_LIMIT);
    }
    Ok(entries)
}

fn recent_tasks_path(config: &Config) -> PathBuf {
    config.data_dir().join(RECENT_TASKS_FILE)
}

fn recent_entry_to_task(entry: &RecentTaskEntry) -> Task {
    blank_task(entry.gid.clone(), entry.name.clone())
}

#[allow(clippy::missing_const_for_fn)]
fn blank_task(gid: String, name: String) -> Task {
    Task {
        gid,
        name,
        resource_type: None,
        resource_subtype: None,
        notes: None,
        html_notes: None,
        completed: false,
        completed_at: None,
        completed_by: None,
        created_at: None,
        modified_at: None,
        due_on: None,
        due_at: None,
        start_on: None,
        start_at: None,
        assignee: None,
        assignee_status: None,
        workspace: None,
        parent: None,
        memberships: Vec::new(),
        projects: Vec::new(),
        tags: Vec::new(),
        followers: Vec::new(),
        dependencies: Vec::new(),
        dependents: Vec::new(),
        custom_fields: Vec::new(),
        attachments: Vec::new(),
        permalink_url: None,
        num_subtasks: None,
    }
}

fn detect_batch_format(path: &Path) -> Result<BatchFormat> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("json") => Ok(BatchFormat::Json),
        Some("csv") => Ok(BatchFormat::Csv),
        _ => Err(anyhow!(
            "unable to determine batch format for {}; specify --format",
            path.display()
        )),
    }
}

fn load_batch_records<T>(path: &Path, format: BatchFormat) -> Result<Vec<T>>
where
    T: DeserializeOwned,
{
    match format {
        BatchFormat::Json => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("failed to read batch file {}", path.display()))?;
            let value: Value = serde_json::from_str(&contents)
                .with_context(|| format!("failed to parse JSON batch file {}", path.display()))?;
            match value {
                Value::Array(entries) => entries
                    .into_iter()
                    .map(|entry| {
                        serde_json::from_value(entry).context("failed to decode batch record")
                    })
                    .collect(),
                other => Err(anyhow!(
                    "expected JSON array in batch file {}, got {other:?}",
                    path.display()
                )),
            }
        }
        BatchFormat::Csv => {
            let mut reader = csv::ReaderBuilder::new()
                .flexible(true)
                .trim(csv::Trim::All)
                .from_path(path)
                .with_context(|| format!("failed to open CSV batch file {}", path.display()))?;
            let mut records = Vec::new();
            for record in reader.deserialize() {
                let value: T = record.with_context(|| {
                    format!("failed to decode CSV batch record in {}", path.display())
                })?;
                records.push(value);
            }
            Ok(records)
        }
    }
}

fn deserialize_list_field<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    let list = match value {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items
            .into_iter()
            .filter_map(|item| match item {
                Value::String(s) => Some(s),
                Value::Number(n) => n.as_f64().map(|v| v.to_string()),
                Value::Bool(b) => Some(b.to_string()),
                _ => None,
            })
            .collect(),
        Some(Value::String(s)) => split_list_string(&s),
        Some(other) => {
            return Err(serde::de::Error::custom(format!(
                "expected string or array, found {other:?}"
            )));
        }
    };
    Ok(list)
}

fn split_list_string(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(std::string::ToString::to_string)
        .collect()
}

fn deserialize_map_field<'de, D>(
    deserializer: D,
) -> std::result::Result<Map<String, Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    let map = match value {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(map)) => map,
        Some(Value::String(s)) => parse_map_string(&s).map_err(serde::de::Error::custom)?,
        Some(other) => {
            return Err(serde::de::Error::custom(format!(
                "expected object or string, found {other:?}"
            )));
        }
    };
    Ok(map)
}

fn parse_map_string(input: &str) -> Result<Map<String, Value>> {
    if input.trim().is_empty() {
        return Ok(Map::new());
    }
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(input) {
        return Ok(map);
    }

    let mut map = Map::new();
    for segment in input.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let (key, value) = segment
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid custom field pair '{segment}'"))?;
        map.insert(
            key.trim().to_string(),
            Value::String(value.trim().to_string()),
        );
    }
    Ok(map)
}

fn to_custom_field_value(value: Value) -> CustomFieldValue {
    match value {
        Value::String(text) => CustomFieldValue::Text(text),
        Value::Number(number) => number.as_f64().map_or_else(
            || CustomFieldValue::Json(Value::Number(number)),
            CustomFieldValue::Number,
        ),
        Value::Bool(flag) => CustomFieldValue::Bool(flag),
        Value::Array(values) => {
            if values.iter().all(Value::is_string) {
                let items = values
                    .into_iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect();
                CustomFieldValue::MultiEnum(items)
            } else {
                CustomFieldValue::Json(Value::Array(values))
            }
        }
        Value::Object(map) => CustomFieldValue::Json(Value::Object(map)),
        Value::Null => CustomFieldValue::Json(Value::Null),
    }
}

fn build_create_request(record: &BatchCreateRecord, config: &Config) -> Result<TaskCreateRequest> {
    let mut builder = TaskCreateBuilder::new(record.name.clone());

    if let Some(notes) = record.notes.as_ref() {
        builder = builder.notes(notes.clone());
    }
    if let Some(html_notes) = record.html_notes.as_ref() {
        builder = builder.html_notes(html_notes.clone());
    }

    if let Some(workspace) = record.workspace.clone().or_else(|| {
        config
            .default_workspace()
            .map(std::string::ToString::to_string)
    }) {
        builder = builder.workspace(workspace);
    }
    let resolved_assignee = resolve_assignee(record.assignee.clone(), config, false);
    for project in &record.projects {
        builder = builder.project(project.clone());
    }
    if let Some(section) = record.section.as_ref() {
        builder = builder.section(section.clone());
    }
    if let Some(parent) = record.parent.as_ref() {
        builder = builder.parent(parent.clone());
    }
    if let Some(assignee) = resolved_assignee {
        builder = builder.assignee(assignee);
    }
    if let Some(value) = record.due_on.as_ref() {
        builder = builder.due_on(parse_date_input(value)?);
    }
    if let Some(value) = record.due_at.as_ref() {
        builder = builder.due_at(parse_datetime_input(value)?);
    }
    if let Some(value) = record.start_on.as_ref() {
        builder = builder.start_on(parse_date_input(value)?);
    }
    if let Some(value) = record.start_at.as_ref() {
        builder = builder.start_at(parse_datetime_input(value)?);
    }
    for tag in &record.tags {
        builder = builder.tag(tag.clone());
    }
    for follower in &record.followers {
        builder = builder.follower(follower.clone());
    }
    for (field, value) in &record.custom_fields {
        builder = builder.custom_field(field.clone(), to_custom_field_value(value.clone()));
    }

    builder
        .build()
        .map_err(|err| map_validation_error(&err, "create batch"))
}

fn build_update_request(record: &BatchUpdateRecord, config: &Config) -> Result<TaskUpdateRequest> {
    let mut builder = TaskUpdateBuilder::new();

    if let Some(name) = record.name.as_ref() {
        builder = builder.name(name.clone());
    }
    if record.clear_notes.unwrap_or(false) {
        builder = builder.clear_notes();
    } else if let Some(notes) = record.notes.as_ref() {
        builder = builder.notes(notes.clone());
    }
    if record.clear_html_notes.unwrap_or(false) {
        builder = builder.clear_html_notes();
    } else if let Some(notes) = record.html_notes.as_ref() {
        builder = builder.html_notes(notes.clone());
    }
    if let Some(completed) = record.completed {
        builder = builder.completed(completed);
    }
    if record.clear_assignee.unwrap_or(false) {
        builder = builder.clear_assignee();
    } else if let Some(assignee) = resolve_assignee(record.assignee.clone(), config, false) {
        builder = builder.assignee(assignee);
    }
    if record.clear_due_on.unwrap_or(false) {
        builder = builder.clear_due_on();
    } else if let Some(value) = record.due_on.as_ref() {
        builder = builder.due_on(parse_date_input(value)?);
    }
    if record.clear_due_at.unwrap_or(false) {
        builder = builder.clear_due_at();
    } else if let Some(value) = record.due_at.as_ref() {
        builder = builder.due_at(parse_datetime_input(value)?);
    }
    if record.clear_start_on.unwrap_or(false) {
        builder = builder.clear_start_on();
    } else if let Some(value) = record.start_on.as_ref() {
        builder = builder.start_on(parse_date_input(value)?);
    }
    if record.clear_start_at.unwrap_or(false) {
        builder = builder.clear_start_at();
    } else if let Some(value) = record.start_at.as_ref() {
        builder = builder.start_at(parse_datetime_input(value)?);
    }
    if record.clear_parent.unwrap_or(false) {
        builder = builder.clear_parent();
    } else if let Some(parent) = record.parent.as_ref() {
        builder = builder.parent(parent.clone());
    }
    if record.clear_tags.unwrap_or(false) {
        builder = builder.tags(Vec::<String>::new());
    } else if !record.tags.is_empty() {
        builder = builder.tags(record.tags.clone());
    }
    if record.clear_followers.unwrap_or(false) {
        builder = builder.followers(Vec::<String>::new());
    } else if !record.followers.is_empty() {
        builder = builder.followers(record.followers.clone());
    }
    if record.clear_projects.unwrap_or(false) {
        builder = builder.projects(Vec::<String>::new());
    } else if !record.projects.is_empty() {
        builder = builder.projects(record.projects.clone());
    }
    if !record.custom_fields.is_empty() {
        for (field, value) in &record.custom_fields {
            builder = builder.custom_field(field.clone(), to_custom_field_value(value.clone()));
        }
    }

    builder
        .build()
        .map_err(|err| map_validation_error(&err, "update batch"))
}

const fn true_by_default() -> bool {
    true
}
