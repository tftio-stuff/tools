//! Stats command: counts per source.

use anyhow::Result;
use rusqlite::Connection;

use crate::repo::{self, SourceStats};

/// Compute stats for all or one source.
pub fn run_stats(conn: &Connection, source: Option<&str>) -> Result<Vec<SourceStats>> {
    repo::compute_stats(conn, source)
}
