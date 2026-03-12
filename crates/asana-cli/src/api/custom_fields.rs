//! High level custom field operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::CustomField,
};
use futures_util::{StreamExt, pin_mut};
use serde::Deserialize;

/// List custom fields in a workspace.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn list_custom_fields(
    client: &ApiClient,
    workspace_gid: &str,
    limit: Option<usize>,
) -> Result<Vec<CustomField>, ApiError> {
    let endpoint = format!("/workspaces/{workspace_gid}/custom_fields");
    let stream = client.paginate_with_limit::<CustomField>(&endpoint, vec![], limit);
    pin_mut!(stream);

    let mut fields = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        fields.append(&mut page);
    }

    Ok(fields)
}

/// Get a single custom field by GID.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_custom_field(
    client: &ApiClient,
    field_gid: &str,
) -> Result<CustomField, ApiError> {
    let response: SingleCustomFieldResponse = client
        .get_json_with_pairs(&format!("/custom_fields/{field_gid}"), vec![])
        .await?;
    Ok(response.data)
}

#[derive(Debug, Deserialize)]
struct SingleCustomFieldResponse {
    data: CustomField,
}
