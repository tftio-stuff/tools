//! Error types for the Asana API client.

use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

/// Structured information about Asana rate-limit headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Total allowed requests in the current window.
    pub limit: Option<u32>,
    /// Remaining requests available before throttling.
    pub remaining: Option<u32>,
    /// Epoch seconds when the quota resets, if supplied by the API.
    pub reset: Option<u64>,
    /// Suggested delay before retrying (for 429 responses).
    pub retry_after: Option<Duration>,
}

/// Errors that can occur while interacting with the Asana API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// General networking failure.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    /// Response payload could not be deserialised.
    #[error("failed to parse response: {0}")]
    Deserialize(#[from] serde_json::Error),
    /// HTTP status code returned an error.
    #[error("HTTP {status}: {message}")]
    Http {
        /// HTTP status returned by Asana.
        status: StatusCode,
        /// Message extracted from the response body or canonical reason.
        message: String,
        /// Optional structured payload returned alongside the error.
        details: Option<Value>,
    },
    /// Authentication failed (401/403).
    #[error("authentication failed: {0}")]
    Authentication(String),
    /// Rate limit was hit and retries exhausted.
    #[error("rate limited after {retry_after:?}: {body}")]
    RateLimited {
        /// Recommended wait duration before retrying.
        retry_after: Duration,
        /// Raw response body returned with the 429.
        body: String,
    },
    /// Cache layer failure.
    #[error("cache error: {0}")]
    Cache(#[from] std::io::Error),
    /// Offline mode requested data that was not cached.
    #[error("offline mode enabled and no cached response available for {resource}")]
    Offline {
        /// Resource identifier, typically the request path.
        resource: String,
    },
    /// Request could not be cloned for retry attempts.
    #[error("request could not be cloned for retry")]
    UnclonableRequest,
    /// Catch-all error message.
    #[error("{0}")]
    Other(String),
}

impl ApiError {
    /// Convenience constructor for HTTP errors with an optional JSON payload.
    #[must_use]
    pub const fn http(status: StatusCode, message: String, details: Option<Value>) -> Self {
        Self::Http {
            status,
            message,
            details,
        }
    }
}
