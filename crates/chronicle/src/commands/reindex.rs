//! Reindex command: re-ingest using stored base paths.

use anyhow::{Context, Result, bail};
use rusqlite::Connection;
use std::path::Path;

use crate::commands::ingest::{IngestResult, run_ingest};
use crate::models::SourceFormat;
use crate::repo;

/// Reindex all sources, or a single named source.
pub fn run_reindex(
    conn: &Connection,
    source_name: Option<&str>,
    full: bool,
) -> Result<Vec<(String, IngestResult)>> {
    let sources = repo::list_sources(conn)?;
    let mut results = Vec::new();

    for source in &sources {
        if let Some(filter) = source_name {
            if source.name != filter {
                continue;
            }
        }

        let base_path = source
            .base_path
            .as_deref()
            .context(format!("source '{}' has no stored base_path", source.name))?;

        let path = Path::new(base_path);
        if !path.exists() {
            bail!(
                "source '{}' base_path does not exist: {}",
                source.name,
                base_path
            );
        }

        let parser: SourceFormat = source
            .parser
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        let result = run_ingest(conn, &source.name, parser, path, full, None)?;
        results.push((source.name.clone(), result));
    }

    Ok(results)
}
