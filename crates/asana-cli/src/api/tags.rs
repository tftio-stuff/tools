//! High level tag operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{Tag, TagCreateRequest, TagListParams, TagUpdateRequest},
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;
use tracing::debug;

/// Retrieve tags in a workspace according to the supplied parameters.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn list_tags(client: &ApiClient, params: TagListParams) -> Result<Vec<Tag>, ApiError> {
    let query = params.to_query();
    let max_items = params.limit;
    let endpoint = format!("/workspaces/{}/tags", params.workspace);
    let stream = client.paginate_with_limit::<Tag>(&endpoint, query, max_items);
    pin_mut!(stream);

    let mut tags = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        tags.append(&mut page);
    }

    debug!("Retrieved {} tags", tags.len());
    Ok(tags)
}

/// Retrieve a single tag by gid.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn get_tag(client: &ApiClient, gid: &str) -> Result<Tag, ApiError> {
    let response: SingleTagResponse = client
        .get_json_with_pairs(&format!("/tags/{gid}"), vec![])
        .await?;
    Ok(response.data)
}

/// Create a tag using the provided payload.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn create_tag(client: &ApiClient, request: TagCreateRequest) -> Result<Tag, ApiError> {
    let response: SingleTagResponse = client.post_json("/tags", &request).await?;
    Ok(response.data)
}

/// Update a tag using the given payload.
///
/// # Errors
///
/// Returns an error if the API request fails, if deserialization fails, or if the response is invalid.
pub async fn update_tag(
    client: &ApiClient,
    gid: &str,
    request: TagUpdateRequest,
) -> Result<Tag, ApiError> {
    let response: SingleTagResponse = client.put_json(&format!("/tags/{gid}"), &request).await?;
    Ok(response.data)
}

/// Delete a tag permanently.
///
/// # Errors
///
/// Returns an error if the API request fails or if the response is invalid.
pub async fn delete_tag(client: &ApiClient, gid: &str) -> Result<(), ApiError> {
    client.delete(&format!("/tags/{gid}"), Vec::new()).await
}

#[derive(Debug, Deserialize)]
struct SingleTagResponse {
    data: Tag,
}
