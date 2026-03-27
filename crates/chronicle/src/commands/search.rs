//! Search command: FTS5 query against message content.

use anyhow::Result;
use rusqlite::Connection;

use crate::repo::{self, MessageRow};

/// Run a search query.
pub fn run_search(
    conn: &Connection,
    query: &str,
    source: Option<&str>,
    role: Option<&str>,
    project: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    repo::search_messages(conn, query, source, role, project, limit)
}
