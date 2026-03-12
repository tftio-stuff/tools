//! Pagination helpers matching Asana's REST API structure.

use serde::Deserialize;

/// Metadata describing the next page of a list response.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationInfo {
    /// Offset token for subsequent calls.
    pub offset: Option<String>,
    /// Canonical path for the next page.
    pub path: Option<String>,
    /// Canonical URI when provided.
    pub uri: Option<String>,
}

impl PaginationInfo {
    /// Return the offset token if present.
    #[must_use]
    pub fn offset(&self) -> Option<&str> {
        self.offset.as_deref()
    }
}

/// Generic Asana list response where data is wrapped with pagination metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct ListResponse<T> {
    /// Data payload.
    pub data: Vec<T>,
    /// Pagination metadata, when more items are available.
    #[serde(default)]
    pub next_page: Option<PaginationInfo>,
}

impl<T> ListResponse<T> {
    /// Determine whether a subsequent request is required.
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_page
            .as_ref()
            .and_then(PaginationInfo::offset)
            .is_some()
    }
}
