//! Extract `BlueSky` post history to `SQLite` via the AT Protocol.

pub mod cli;
pub mod client;
pub mod db;
pub mod error;
pub mod models;

pub use client::BskyClient;
pub use error::ExtractorError;

/// Run a complete extraction for the given handle.
///
/// Reads `BSKY_APP_PASSWORD` from the environment. If absent, uses the public
/// `AppView` API (`public.api.bsky.app`) with a warning about lower rate limits.
///
/// The `db_path` is where the `SQLite` database is created or opened. The
/// `since` parameter optionally limits extraction to posts after the given
/// `UTC` timestamp.
///
/// The `on_progress` callback, if provided, is invoked with the running
/// total of processed records after each post is stored.
///
/// # Errors
///
/// Returns `ExtractorError::AuthFailed` if authentication fails,
/// `ExtractorError::RateLimitExhausted` if the rate limit is exhausted,
/// or `ExtractorError::Db` if the database cannot be opened or written.
#[allow(clippy::future_not_send)]
pub async fn run_extraction(
    handle: &str,
    db_path: &std::path::Path,
    since: Option<chrono::DateTime<chrono::Utc>>,
    on_progress: Option<&dyn Fn(u64)>,
) -> Result<models::FetchSummary, error::ExtractorError> {
    // 1. Open + init DB
    let conn = db::open_db(db_path)?;
    db::init_db(&conn)?;

    // 2. Read BSKY_APP_PASSWORD from env
    let password = std::env::var("BSKY_APP_PASSWORD").ok();

    // 3. Create BskyClient (with or without credentials)
    let mut bsky_client = password.as_ref().map_or_else(
        || BskyClient::new(None),
        |pw| BskyClient::new(Some((handle, pw.as_str()))),
    );

    // 4. If credentials present: authenticate and get DID from response
    //    If no credentials: resolve handle to DID via public API
    let did = if let Some(ref pw) = password {
        bsky_client.authenticate(handle, pw).await?
    } else {
        bsky_client.resolve_handle(handle).await?
    };

    // 5. Fetch all posts with pagination and cursor persistence
    bsky_client
        .fetch_all_posts(&did, since, &conn, on_progress)
        .await
}
