//! AT Protocol HTTP client with authentication, rate-limit backoff, and pagination.

use std::time::Duration;

use reqwest::{Method, StatusCode, header::AUTHORIZATION};

use crate::error::ExtractorError;
use crate::models::{
    CreateSessionRequest, CreateSessionResponse, FetchSummary, ListRecordsResponse,
    ResolveHandleResponse,
};

/// Maximum consecutive 429 retry attempts before giving up.
const MAX_RETRIES: u32 = 5;

/// Base URL for the public `BlueSky` `AppView` API (no auth required).
const PUBLIC_API_BASE: &str = "https://public.api.bsky.app";

/// Base URL for the `bsky.social` PDS (used when authenticating with app password).
const PDS_BASE: &str = "https://bsky.social";

/// AT Protocol collection lexicon ID for feed posts.
const COLLECTION_FEED_POST: &str = "app.bsky.feed.post";

/// Maximum number of records per page for `listRecords`.
const PAGE_LIMIT: &str = "100";

/// Async AT Protocol HTTP client.
///
/// Handles `BlueSky` authentication, handle resolution, exhaustive pagination,
/// rate-limit backoff, and token refresh. Writes posts directly to `SQLite`
/// via the `db` module.
pub struct BskyClient {
    http: reqwest::Client,
    base_url: String,
    access_jwt: Option<String>,
    refresh_jwt: Option<String>,
}

impl BskyClient {
    /// Create a new `BskyClient`.
    ///
    /// If `credentials` is `Some((handle, password))`, the client targets the
    /// `bsky.social` PDS and will call `authenticate` before fetching.
    /// If `credentials` is `None`, the client targets the public `AppView` API
    /// and emits a warning about lower rate limits.
    #[must_use]
    pub fn new(credentials: Option<(&str, &str)>) -> Self {
        let base_url = if credentials.is_some() {
            PDS_BASE.to_string()
        } else {
            tracing::warn!("No BSKY_APP_PASSWORD set; using public API with lower rate limits");
            PUBLIC_API_BASE.to_string()
        };
        Self {
            http: reqwest::Client::new(),
            base_url,
            access_jwt: None,
            refresh_jwt: None,
        }
    }

    /// Authenticate with `BlueSky` using the given handle and app password.
    ///
    /// On success, stores the access and refresh JWTs internally and returns
    /// the authenticated account's DID. On failure returns
    /// `ExtractorError::AuthFailed` with the HTTP status and body.
    pub async fn authenticate(
        &mut self,
        handle: &str,
        password: &str,
    ) -> Result<String, ExtractorError> {
        let body = serde_json::to_vec(&CreateSessionRequest {
            identifier: handle,
            password,
        })?;
        let resp = self
            .http
            .post(self.build_url("/xrpc/com.atproto.server.createSession"))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(ExtractorError::Network)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(ExtractorError::AuthFailed(format!("{status}: {body_text}")));
        }

        let session: CreateSessionResponse = resp.json().await.map_err(ExtractorError::Network)?;
        self.access_jwt = Some(session.access_jwt);
        self.refresh_jwt = Some(session.refresh_jwt);
        Ok(session.did)
    }

    /// Refresh the access token using the stored refresh JWT.
    ///
    /// On success, updates both `access_jwt` and `refresh_jwt` in-place.
    /// Returns `ExtractorError::AuthExpired` if the refresh fails.
    pub async fn refresh_auth(&mut self) -> Result<(), ExtractorError> {
        let refresh_jwt = match &self.refresh_jwt {
            Some(t) => t.clone(),
            None => return Err(ExtractorError::AuthExpired),
        };

        let resp = self
            .http
            .post(self.build_url("/xrpc/com.atproto.server.refreshSession"))
            .header(AUTHORIZATION, format!("Bearer {refresh_jwt}"))
            .send()
            .await
            .map_err(ExtractorError::Network)?;

        if !resp.status().is_success() {
            return Err(ExtractorError::AuthExpired);
        }

        let session: CreateSessionResponse =
            resp.json().await.map_err(|_| ExtractorError::AuthExpired)?;
        self.access_jwt = Some(session.access_jwt);
        self.refresh_jwt = Some(session.refresh_jwt);
        Ok(())
    }

    /// Resolve a `BlueSky` handle to its decentralized identifier (DID).
    ///
    /// Returns `ExtractorError::InvalidHandle` if the resolution fails.
    pub async fn resolve_handle(&self, handle: &str) -> Result<String, ExtractorError> {
        let resp = self
            .http
            .get(self.build_url("/xrpc/com.atproto.identity.resolveHandle"))
            .query(&[("handle", handle)])
            .send()
            .await
            .map_err(ExtractorError::Network)?;

        if !resp.status().is_success() {
            return Err(ExtractorError::InvalidHandle(handle.to_string()));
        }

        let result: ResolveHandleResponse = resp
            .json()
            .await
            .map_err(|_| ExtractorError::InvalidHandle(handle.to_string()))?;
        Ok(result.did)
    }

    /// Execute an HTTP request with retry logic for 429, 401, and network errors.
    ///
    /// Retries up to `MAX_RETRIES` times on rate-limit responses (429), reading
    /// `Retry-After` or `ratelimit-reset` headers to determine the wait duration.
    /// On 401, attempts a single token refresh before failing with
    /// `ExtractorError::AuthExpired`. Network timeouts and connection errors also
    /// trigger exponential backoff retries.
    pub async fn execute(
        &mut self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, ExtractorError> {
        let mut attempt = 0u32;
        let mut refreshed = false;

        loop {
            let mut req = self
                .http
                .request(method.clone(), self.build_url(path))
                .query(query);

            if let Some(token) = &self.access_jwt {
                req = req.header(AUTHORIZATION, format!("Bearer {token}"));
            }

            if let Some(ref body_bytes) = body {
                req = req
                    .header("Content-Type", "application/json")
                    .body(body_bytes.clone());
            }

            match req.send().await {
                Err(e) if (e.is_timeout() || e.is_connect()) && attempt < MAX_RETRIES => {
                    tokio::time::sleep(backoff_delay(attempt)).await;
                    attempt += 1;
                }
                Err(e) => return Err(ExtractorError::Network(e)),
                Ok(resp) if resp.status() == StatusCode::TOO_MANY_REQUESTS => {
                    let wait =
                        parse_retry_after(resp.headers()).unwrap_or_else(|| backoff_delay(attempt));
                    if attempt >= MAX_RETRIES {
                        return Err(ExtractorError::RateLimitExhausted);
                    }
                    tokio::time::sleep(wait).await;
                    attempt += 1;
                }
                Ok(resp) if resp.status() == StatusCode::UNAUTHORIZED => {
                    if refreshed {
                        return Err(ExtractorError::AuthExpired);
                    }
                    self.refresh_auth().await?;
                    refreshed = true;
                    // Do not increment attempt -- retry immediately after refresh
                }
                Ok(resp) if resp.status().is_success() => {
                    let raw = resp.bytes().await?;
                    return Ok(raw.to_vec());
                }
                Ok(resp) => return Err(ExtractorError::Http(resp.status())),
            }
        }
    }

    /// Fetch all posts for the given DID, writing each to the `SQLite` database.
    ///
    /// Resumes from a saved cursor if one exists in the database. Stops
    /// early if an already-seen AT URI is encountered (incremental mode) or
    /// if the post's `createdAt` is before the optional `since` cutoff. Saves
    /// the pagination cursor to the database after each page for resilience.
    ///
    /// The `on_progress` callback, if provided, is invoked with the running
    /// total of processed records after each post is stored.
    ///
    /// Note: `rusqlite::Connection` is not `Send`; this future is intentionally
    /// single-threaded.
    #[allow(clippy::future_not_send)]
    pub async fn fetch_all_posts(
        &mut self,
        did: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        conn: &rusqlite::Connection,
        on_progress: Option<&dyn Fn(u64)>,
    ) -> Result<FetchSummary, ExtractorError> {
        let mut cursor: Option<String> = crate::db::load_resume_cursor(conn, did)?;
        let mut count = 0u64;
        let mut new_count = 0u64;
        let mut existing_count = 0u64;

        loop {
            let mut params: Vec<(&str, String)> = vec![
                ("repo", did.to_string()),
                ("collection", COLLECTION_FEED_POST.to_string()),
                ("limit", PAGE_LIMIT.to_string()),
            ];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }

            // Convert params to slices for execute
            let param_refs: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();

            let page_bytes = self
                .execute(
                    Method::GET,
                    "/xrpc/com.atproto.repo.listRecords",
                    &param_refs,
                    None,
                )
                .await?;
            let page: ListRecordsResponse = serde_json::from_slice(&page_bytes)?;

            for record in &page.records {
                // Incremental stop: hit a URI we already have
                if crate::db::db_has_uri(conn, &record.uri)? {
                    crate::db::complete_extraction(conn, did, count)?;
                    return Ok(FetchSummary {
                        count,
                        done: true,
                        new_count,
                        existing_count,
                    });
                }

                // Extract text and created_at from the record value
                let Some((text, created_at)) = extract_post_fields(&record.value) else {
                    tracing::warn!(uri = %record.uri, "skipping record: missing text or createdAt");
                    continue;
                };

                // Since cutoff: stop if post is older than the cutoff
                if let Some(cutoff) = since {
                    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&created_at) {
                        if ts.with_timezone(&chrono::Utc) < cutoff {
                            crate::db::complete_extraction(conn, did, count)?;
                            return Ok(FetchSummary {
                                count,
                                done: true,
                                new_count,
                                existing_count,
                            });
                        }
                    }
                }

                let raw_json = serde_json::to_string(&record.value)?;
                let is_new =
                    crate::db::upsert_post(conn, &record.uri, did, &text, &created_at, &raw_json)?;
                count += 1;
                if is_new {
                    new_count += 1;
                } else {
                    existing_count += 1;
                }
                if let Some(cb) = on_progress {
                    cb(count);
                }
            }

            // Save cursor after every page for resilience
            crate::db::save_cursor(conn, did, page.cursor.as_deref())?;

            match page.cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }

        crate::db::complete_extraction(conn, did, count)?;
        Ok(FetchSummary {
            count,
            done: true,
            new_count,
            existing_count,
        })
    }

    /// Build a full URL from a path relative to `base_url`.
    #[must_use]
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

/// Compute exponential backoff delay for the given attempt number.
///
/// Returns `1s * 2^attempt`, capped at 60 seconds.
/// `backoff_delay(0)` = 1s, `backoff_delay(1)` = 2s, `backoff_delay(6)` = 60s.
#[must_use]
pub fn backoff_delay(attempt: u32) -> Duration {
    Duration::from_secs((1u64 << attempt.min(63)).min(60))
}

/// Parse a retry wait duration from response headers.
///
/// Checks `Retry-After` (seconds) first, then falls back to `ratelimit-reset`
/// (Unix epoch). Returns `None` if neither header is present or parseable.
#[must_use]
pub fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    // Retry-After header takes priority
    if let Some(val) = headers.get(reqwest::header::RETRY_AFTER) {
        if let Ok(secs) = val.to_str().ok()?.parse::<u64>() {
            return Some(Duration::from_secs(secs));
        }
    }
    // Fall back to ratelimit-reset (Unix epoch)
    let reset_str = headers.get("ratelimit-reset")?.to_str().ok()?;
    let reset_epoch: u64 = reset_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(Duration::from_secs(reset_epoch.saturating_sub(now).max(1)))
}

/// Extract the `text` and `createdAt` fields from a raw post record `Value`.
///
/// Returns `None` if either field is missing or not a string.
#[must_use]
pub fn extract_post_fields(value: &serde_json::Value) -> Option<(String, String)> {
    let text = value["text"].as_str()?.to_string();
    let created_at = value["createdAt"].as_str()?.to_string();
    Some((text, created_at))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // backoff_delay tests
    // ------------------------------------------------------------------

    #[test]
    fn test_backoff_delay() {
        assert_eq!(backoff_delay(0), Duration::from_secs(1));
        assert_eq!(backoff_delay(1), Duration::from_secs(2));
        assert_eq!(backoff_delay(2), Duration::from_secs(4));
        // 2^6 = 64 > 60, so must be capped at 60
        assert_eq!(backoff_delay(6), Duration::from_secs(60));
    }

    // ------------------------------------------------------------------
    // parse_retry_after tests
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_retry_after_seconds() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RETRY_AFTER,
            reqwest::header::HeaderValue::from_static("5"),
        );
        let result = parse_retry_after(&headers);
        assert_eq!(result, Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_parse_retry_after_missing() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn test_parse_rate_limit_reset() {
        let future_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 10;
        let epoch_str = future_epoch.to_string();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "ratelimit-reset",
            reqwest::header::HeaderValue::from_str(&epoch_str).unwrap(),
        );
        let result = parse_retry_after(&headers).unwrap();
        // Should be approximately 10 seconds; allow 2s slack for test timing
        assert!(
            result >= Duration::from_secs(8),
            "expected ~10s, got {result:?}"
        );
        assert!(
            result <= Duration::from_secs(12),
            "expected ~10s, got {result:?}"
        );
    }

    // ------------------------------------------------------------------
    // BskyClient constructor tests
    // ------------------------------------------------------------------

    #[test]
    fn test_client_new_public_mode() {
        let client = BskyClient::new(None);
        assert_eq!(client.base_url, PUBLIC_API_BASE);
        assert!(client.access_jwt.is_none());
        assert!(client.refresh_jwt.is_none());
    }

    #[test]
    fn test_client_new_auth_mode() {
        let client = BskyClient::new(Some(("user.bsky.social", "secret")));
        assert_eq!(client.base_url, PDS_BASE);
    }

    // ------------------------------------------------------------------
    // build_url tests
    // ------------------------------------------------------------------

    #[test]
    fn test_build_url() {
        let client = BskyClient::new(Some(("h", "p")));
        assert_eq!(
            client.build_url("/xrpc/foo"),
            format!("{PDS_BASE}/xrpc/foo")
        );
    }

    // ------------------------------------------------------------------
    // extract_post_fields tests
    // ------------------------------------------------------------------

    #[test]
    fn test_extract_post_fields() {
        let value = serde_json::json!({
            "text": "Hello world",
            "createdAt": "2024-01-01T00:00:00Z"
        });
        let result = extract_post_fields(&value);
        assert_eq!(
            result,
            Some((
                "Hello world".to_string(),
                "2024-01-01T00:00:00Z".to_string()
            ))
        );
    }

    #[test]
    fn test_extract_post_fields_missing_text() {
        let value = serde_json::json!({ "createdAt": "2024-01-01T00:00:00Z" });
        assert_eq!(extract_post_fields(&value), None);
    }

    #[test]
    fn test_extract_post_fields_missing_created_at() {
        let value = serde_json::json!({ "text": "Hello" });
        assert_eq!(extract_post_fields(&value), None);
    }

    // ------------------------------------------------------------------
    // DB integration tests (Task 2 behaviour)
    // ------------------------------------------------------------------

    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_fetch_all_posts_db_roundtrip() {
        let conn = test_db();
        let did = "did:plc:test123";
        let uri = "at://did:plc:test123/app.bsky.feed.post/001";
        let raw = r#"{"text":"Hello","createdAt":"2024-01-01T00:00:00Z"}"#;

        crate::db::upsert_post(&conn, uri, did, "Hello", "2024-01-01T00:00:00Z", raw).unwrap();

        let text: String = conn
            .query_row(
                "SELECT text FROM posts WHERE uri = ?1",
                rusqlite::params![uri],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_incremental_stop_on_known_uri() {
        let conn = test_db();
        let uri = "at://did:plc:abc/app.bsky.feed.post/001";
        crate::db::upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "text",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        assert!(crate::db::db_has_uri(&conn, uri).unwrap());
    }

    #[test]
    fn test_cursor_persistence_roundtrip() {
        let conn = test_db();
        let did = "did:plc:roundtrip";
        crate::db::save_cursor(&conn, did, Some("my-cursor-abc")).unwrap();
        let loaded = crate::db::load_resume_cursor(&conn, did).unwrap();
        assert_eq!(loaded, Some("my-cursor-abc".to_string()));
    }

    #[test]
    fn test_fetch_summary_has_new_and_existing_count() {
        // Compile test: FetchSummary must have new_count and existing_count fields.
        let summary = FetchSummary {
            count: 5,
            done: true,
            new_count: 3,
            existing_count: 2,
        };
        assert_eq!(summary.new_count, 3);
        assert_eq!(summary.existing_count, 2);
    }

    #[test]
    fn test_complete_extraction_sets_completed_at() {
        let conn = test_db();
        let did = "did:plc:complete-test";
        // Insert an extraction row first via save_cursor
        crate::db::save_cursor(&conn, did, Some("cursor-xyz")).unwrap();
        crate::db::complete_extraction(&conn, did, 42).unwrap();

        let completed_at: Option<String> = conn
            .query_row(
                "SELECT completed_at FROM extractions WHERE target_did = ?1 ORDER BY id DESC LIMIT 1",
                rusqlite::params![did],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            completed_at.is_some(),
            "completed_at must be non-null after complete_extraction"
        );
    }
}
