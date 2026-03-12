//! High level section operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{AddTaskToSectionData, AddTaskToSectionRequest, Section, SectionCreateRequest, Task},
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;

/// Retrieve sections for a project.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn list_sections(
    client: &ApiClient,
    project_gid: &str,
) -> Result<Vec<Section>, ApiError> {
    let stream = client.paginate_with_limit::<Section>(
        &format!("/projects/{project_gid}/sections"),
        Vec::new(),
        None,
    );
    pin_mut!(stream);

    let mut sections = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        sections.append(&mut page);
    }

    Ok(sections)
}

/// Retrieve a single section by gid.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn get_section(
    client: &ApiClient,
    section_gid: &str,
    fields: Vec<String>,
) -> Result<Section, ApiError> {
    let mut query = Vec::new();
    if !fields.is_empty() {
        query.push(("opt_fields".into(), fields.join(",")));
    }

    let response: SingleSectionResponse = client
        .get_json_with_pairs(&format!("/sections/{section_gid}"), query)
        .await?;
    Ok(response.data)
}

/// Create a section in a project.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn create_section(
    client: &ApiClient,
    project_gid: &str,
    request: SectionCreateRequest,
) -> Result<Section, ApiError> {
    let response: SingleSectionResponse = client
        .post_json(&format!("/projects/{project_gid}/sections"), &request)
        .await?;
    Ok(response.data)
}

/// Get tasks within a section (board view only).
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn get_section_tasks(
    client: &ApiClient,
    section_gid: &str,
    fields: Vec<String>,
) -> Result<Vec<Task>, ApiError> {
    let mut query = Vec::new();
    if !fields.is_empty() {
        query.push(("opt_fields".into(), fields.join(",")));
    }

    let stream =
        client.paginate_with_limit::<Task>(&format!("/sections/{section_gid}/tasks"), query, None);
    pin_mut!(stream);

    let mut tasks = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        tasks.append(&mut page);
    }

    Ok(tasks)
}

/// Add a task to a section.
///
/// This will remove the task from other sections of the project.
/// The task will be inserted at the top of the section unless
/// `insert_before` or `insert_after` is specified.
///
/// # Errors
///
/// Returns an error if the API request fails or if the response is invalid.
pub async fn add_task_to_section(
    client: &ApiClient,
    section_gid: &str,
    task_gid: String,
    insert_before: Option<String>,
    insert_after: Option<String>,
) -> Result<(), ApiError> {
    let request = AddTaskToSectionRequest {
        data: AddTaskToSectionData {
            task: task_gid,
            insert_before,
            insert_after,
        },
    };

    client
        .post_void(&format!("/sections/{section_gid}/addTask"), &request)
        .await
}

#[derive(Debug, Deserialize)]
struct SingleSectionResponse {
    data: Section,
}
