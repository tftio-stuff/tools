//! High level workspace operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{Workspace, WorkspaceListParams},
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;

/// List workspaces for the current user.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn list_workspaces(
    client: &ApiClient,
    params: WorkspaceListParams,
) -> Result<Vec<Workspace>, ApiError> {
    let stream = client.paginate_with_limit::<Workspace>("/workspaces", vec![], params.limit);
    pin_mut!(stream);

    let mut workspaces = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        workspaces.append(&mut page);
    }

    Ok(workspaces)
}

/// Get a single workspace by GID.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_workspace(client: &ApiClient, gid: &str) -> Result<Workspace, ApiError> {
    let response: SingleWorkspaceResponse = client
        .get_json_with_pairs(&format!("/workspaces/{gid}"), vec![])
        .await?;
    Ok(response.data)
}

#[derive(Debug, Deserialize)]
struct SingleWorkspaceResponse {
    data: Workspace,
}
