//! High level story (comment) operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{Story, StoryCreateRequest, StoryListParams, StoryUpdateRequest},
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;

/// List stories for a task.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn list_stories(
    client: &ApiClient,
    params: StoryListParams,
) -> Result<Vec<Story>, ApiError> {
    let endpoint = format!("/tasks/{}/stories", params.task_gid);
    let stream = client.paginate_with_limit::<Story>(&endpoint, vec![], params.limit);
    pin_mut!(stream);

    let mut stories = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        stories.append(&mut page);
    }

    Ok(stories)
}

/// Get a single story.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_story(client: &ApiClient, gid: &str) -> Result<Story, ApiError> {
    let response: SingleStoryResponse = client
        .get_json_with_pairs(&format!("/stories/{gid}"), vec![])
        .await?;
    Ok(response.data)
}

/// Create a story (comment) on a task.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn create_story(
    client: &ApiClient,
    task_gid: &str,
    request: StoryCreateRequest,
) -> Result<Story, ApiError> {
    let response: SingleStoryResponse = client
        .post_json(&format!("/tasks/{task_gid}/stories"), &request)
        .await?;
    Ok(response.data)
}

/// Update a story.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn update_story(
    client: &ApiClient,
    gid: &str,
    request: StoryUpdateRequest,
) -> Result<Story, ApiError> {
    let response: SingleStoryResponse = client
        .put_json(&format!("/stories/{gid}"), &request)
        .await?;
    Ok(response.data)
}

/// Delete a story.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn delete_story(client: &ApiClient, gid: &str) -> Result<(), ApiError> {
    client.delete(&format!("/stories/{gid}"), Vec::new()).await
}

#[derive(Debug, Deserialize)]
struct SingleStoryResponse {
    data: Story,
}
