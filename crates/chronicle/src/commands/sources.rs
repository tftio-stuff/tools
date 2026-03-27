//! Sources management commands.

use anyhow::Result;
use rusqlite::Connection;

use crate::models::SourceFormat;
use crate::repo::{self, SourceRow};

/// List registered sources.
pub fn run_list(conn: &Connection) -> Result<Vec<SourceRow>> {
    repo::list_sources(conn)
}

/// Add a new source.
pub fn run_add(
    conn: &Connection,
    name: &str,
    parser: SourceFormat,
    path: &str,
    description: Option<&str>,
) -> Result<i64> {
    repo::ensure_source(conn, name, parser, Some(path), description)
}
