//! Serde structs for AT Protocol API requests and responses.

use serde::{Deserialize, Serialize};

/// Request body for `com.atproto.server.createSession`.
#[derive(Serialize)]
pub struct CreateSessionRequest<'a> {
    /// The user's handle or DID used as the login identifier.
    pub identifier: &'a str,
    /// The app password for the account.
    pub password: &'a str,
}

/// Response body from `com.atproto.server.createSession`.
#[derive(Deserialize)]
pub struct CreateSessionResponse {
    /// Short-lived JWT used to authenticate API requests.
    #[serde(rename = "accessJwt")]
    pub access_jwt: String,
    /// Long-lived JWT used to refresh the access token.
    #[serde(rename = "refreshJwt")]
    pub refresh_jwt: String,
    /// The user's handle (e.g., `user.bsky.social`).
    pub handle: String,
    /// The user's decentralized identifier (DID).
    pub did: String,
    /// Whether the account is currently active.
    pub active: Option<bool>,
}

/// Response body from `com.atproto.identity.resolveHandle`.
#[derive(Deserialize)]
pub struct ResolveHandleResponse {
    /// The resolved decentralized identifier (DID) for the handle.
    pub did: String,
}

/// Response body from `com.atproto.repo.listRecords`.
#[derive(Deserialize)]
pub struct ListRecordsResponse {
    /// Opaque cursor for fetching the next page; absent when on the last page.
    pub cursor: Option<String>,
    /// The records returned for this page.
    pub records: Vec<RepoRecord>,
}

/// A single record entry returned by `com.atproto.repo.listRecords`.
#[derive(Deserialize)]
pub struct RepoRecord {
    /// The AT URI identifying this record (e.g., `at://did:plc:.../app.bsky.feed.post/tid`).
    pub uri: String,
    /// The content identifier (CID) for the record.
    pub cid: String,
    /// The raw record value as a JSON value for flexible deserialization.
    pub value: serde_json::Value,
}

/// The parsed value of an `app.bsky.feed.post` record.
#[derive(Deserialize, Serialize)]
pub struct PostValue {
    /// The text content of the post.
    pub text: String,
    /// The ISO 8601 creation timestamp of the post.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Reply reference, if this post is a reply.
    pub reply: Option<serde_json::Value>,
    /// Embedded content (image, link card, etc.), if present.
    pub embed: Option<serde_json::Value>,
    /// List of BCP-47 language tags for the post content.
    pub langs: Option<Vec<String>>,
}

/// Summary of a completed or interrupted fetch operation.
pub struct FetchSummary {
    /// Total number of post records processed during the fetch.
    pub count: u64,
    /// Whether the fetch reached the end of the user's post history.
    pub done: bool,
    /// Number of posts that were newly inserted.
    pub new_count: u64,
    /// Number of posts that already existed and were updated.
    pub existing_count: u64,
}
