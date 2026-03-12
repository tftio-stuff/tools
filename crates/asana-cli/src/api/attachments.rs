//! High level attachment operations built on the core API client.

use crate::{
    api::{ApiClient, ApiError},
    models::{Attachment, AttachmentListParams, AttachmentUploadParams},
};
use futures_util::{StreamExt, pin_mut};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use std::path::Path;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

/// List attachments for a task.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn list_attachments(
    client: &ApiClient,
    params: AttachmentListParams,
) -> Result<Vec<Attachment>, ApiError> {
    let endpoint = format!("/tasks/{}/attachments", params.task_gid);
    let stream = client.paginate_with_limit::<Attachment>(&endpoint, vec![], params.limit);
    pin_mut!(stream);

    let mut attachments = Vec::new();
    while let Some(page) = stream.next().await {
        let mut page = page?;
        attachments.append(&mut page);
    }

    Ok(attachments)
}

/// Get a single attachment.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn get_attachment(client: &ApiClient, gid: &str) -> Result<Attachment, ApiError> {
    let response: SingleAttachmentResponse = client
        .get_json_with_pairs(&format!("/attachments/{gid}"), vec![])
        .await?;
    Ok(response.data)
}

/// Upload an attachment to a task.
///
/// # Errors
/// Returns [`ApiError`] if the file cannot be read, the upload fails, or network errors occur.
pub async fn upload_attachment(
    client: &ApiClient,
    params: AttachmentUploadParams,
) -> Result<Attachment, ApiError> {
    // Read file
    let file = File::open(&params.file_path)
        .await
        .map_err(|e| ApiError::Other(format!("failed to open file: {e}")))?;

    // Get filename
    let filename = params.name.unwrap_or_else(|| {
        params
            .file_path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("attachment")
            .to_string()
    });

    // Create multipart form
    let stream = FramedRead::new(file, BytesCodec::new());
    let file_body = reqwest::Body::wrap_stream(stream);
    let file_part = Part::stream(file_body).file_name(filename);

    let form = Form::new().part("file", file_part);

    // POST multipart form
    let endpoint = format!("/tasks/{}/attachments", params.task_gid);
    let response: SingleAttachmentResponse = client.post_multipart(&endpoint, form).await?;

    Ok(response.data)
}

/// Delete an attachment.
///
/// # Errors
/// Returns [`ApiError`] if the API request fails or network errors occur.
pub async fn delete_attachment(client: &ApiClient, gid: &str) -> Result<(), ApiError> {
    client
        .delete(&format!("/attachments/{gid}"), Vec::new())
        .await
}

/// Download an attachment to a local file.
///
/// # Errors
/// Returns [`ApiError`] if the attachment cannot be fetched, downloaded, or written to disk.
pub async fn download_attachment(
    client: &ApiClient,
    gid: &str,
    output_path: &Path,
) -> Result<(), ApiError> {
    // Get attachment metadata to get download URL
    let attachment = get_attachment(client, gid).await?;

    let download_url = attachment
        .download_url
        .ok_or_else(|| ApiError::Other("attachment has no download URL".into()))?;

    // Download file content
    let bytes = client.download_file(&download_url).await?;

    // Write to disk
    tokio::fs::write(output_path, bytes)
        .await
        .map_err(|e| ApiError::Other(format!("failed to write file: {e}")))?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SingleAttachmentResponse {
    data: Attachment,
}
