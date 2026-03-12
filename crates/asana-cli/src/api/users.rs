//! High level user operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{User, UserListParams},
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;

/// List users in a workspace.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn list_users(client: &ApiClient, params: UserListParams) -> Result<Vec<User>, ApiError> {
    let endpoint = format!("/workspaces/{}/users", params.workspace_gid);
    let stream = client.paginate_with_limit::<User>(&endpoint, vec![], params.limit);
    pin_mut!(stream);

    let mut users = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        users.append(&mut page);
    }

    Ok(users)
}

/// Get a single user by GID.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_user(client: &ApiClient, gid: &str) -> Result<User, ApiError> {
    let response: SingleUserResponse = client
        .get_json_with_pairs(&format!("/users/{gid}"), vec![])
        .await?;
    Ok(response.data)
}

/// Get the current authenticated user.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_current_user(client: &ApiClient) -> Result<User, ApiError> {
    let response: SingleUserResponse = client.get_json_with_pairs("/users/me", vec![]).await?;
    Ok(response.data)
}

#[derive(Debug, Deserialize)]
struct SingleUserResponse {
    data: User,
}
