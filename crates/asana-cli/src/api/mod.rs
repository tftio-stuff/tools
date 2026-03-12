//! Asana API client module providing authenticated HTTP access, pagination,
//! and rate-limit aware retry logic.

pub mod attachments;
pub mod auth;
pub mod client;
pub mod custom_fields;
pub mod error;
pub mod pagination;
pub mod projects;
pub mod sections;
pub mod stories;
pub mod tags;
pub mod tasks;
pub mod users;
pub mod workspaces;

pub use attachments::{
    delete_attachment, download_attachment, get_attachment, list_attachments, upload_attachment,
};
pub use auth::{AuthToken, StaticTokenProvider, TokenProvider};
pub use client::{ApiClient, ApiClientBuilder, ApiClientOptions};
pub use custom_fields::{get_custom_field, list_custom_fields};
pub use error::{ApiError, RateLimitInfo};
pub use pagination::{ListResponse, PaginationInfo};
pub use projects::{
    add_members, create_project, delete_project, get_project, list_members, list_projects,
    list_statuses, remove_members, update_member, update_project,
};
pub use sections::{
    add_task_to_section, create_section, get_section, get_section_tasks, list_sections,
};
pub use stories::{create_story, delete_story, get_story, list_stories, update_story};
pub use tags::{create_tag, delete_tag, get_tag, list_tags, update_tag};
pub use tasks::{
    add_dependencies, add_dependents, add_followers, add_project, add_tag, create_task,
    delete_task, get_task, list_dependencies, list_dependents, list_subtasks, list_tasks,
    remove_dependencies, remove_dependents, remove_followers, remove_project, remove_tag,
    search_tasks, update_task,
};
pub use users::{get_current_user, get_user, list_users};
pub use workspaces::{get_workspace, list_workspaces};
