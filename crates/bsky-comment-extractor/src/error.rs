//! Error types for the bsky-comment-extractor crate.

/// All errors that can occur during BlueSky post extraction.
#[derive(thiserror::Error, Debug)]
pub enum ExtractorError {
    /// An HTTP response with an unexpected status code was received.
    #[error("HTTP error: {0}")]
    Http(reqwest::StatusCode),

    /// A network-level error occurred (connection refused, timeout, etc.).
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// A SQLite database error occurred.
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    /// A JSON serialization or deserialization error occurred.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// An I/O error occurred (e.g., creating parent directories).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The API rate limit was exhausted after all retry attempts.
    #[error("Rate limit exhausted after maximum retry attempts")]
    RateLimitExhausted,

    /// The access token has expired and could not be refreshed.
    #[error("Authentication token expired")]
    AuthExpired,

    /// Authentication failed with the given reason.
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// The provided handle could not be resolved to a DID.
    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    /// The stored pagination cursor is no longer valid.
    #[error("Pagination cursor expired; restart extraction from the beginning")]
    CursorExpired,
}
