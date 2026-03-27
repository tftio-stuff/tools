//! Ingest command: walk source files, hash, parse, and insert.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use walkdir::WalkDir;

use crate::models::SourceFormat;
use crate::parsers;
use crate::repo;

/// Result of an ingest operation.
#[derive(Debug)]
pub struct IngestResult {
    /// Files scanned.
    pub scanned: u64,
    /// New files ingested.
    pub new: u64,
    /// Files re-ingested due to hash change.
    pub updated: u64,
    /// Files skipped (hash unchanged).
    pub skipped: u64,
    /// Sessions inserted.
    pub sessions: u64,
    /// Messages inserted.
    pub messages: u64,
}

/// Run ingest for a source.
pub fn run_ingest(
    conn: &Connection,
    source_name: &str,
    parser: SourceFormat,
    base_path: &Path,
    full: bool,
    description: Option<&str>,
) -> Result<IngestResult> {
    let source_id = repo::ensure_source(
        conn,
        source_name,
        parser,
        Some(&base_path.to_string_lossy()),
        description,
    )?;

    if full {
        repo::delete_source_data(conn, source_id)?;
    }

    let mut result = IngestResult {
        scanned: 0,
        new: 0,
        updated: 0,
        skipped: 0,
        sessions: 0,
        messages: 0,
    };

    for entry in WalkDir::new(base_path)
        .follow_links(true)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        if !parsers::is_relevant_path(parser, path, base_path) {
            continue;
        }

        result.scanned += 1;
        let path_str = path.to_string_lossy().to_string();

        // Compute blake3 hash
        let file_bytes =
            std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let hash = blake3::hash(&file_bytes).to_hex().to_string();
        let size_bytes = i64::try_from(file_bytes.len()).unwrap_or(0);

        // Check existing file
        let existing = repo::get_file(conn, source_id, &path_str)?;

        match existing {
            Some((_file_id, ref existing_hash)) if *existing_hash == hash && !full => {
                result.skipped += 1;
                continue;
            }
            Some((file_id, _)) => {
                // Hash changed or full rebuild — delete old data
                repo::delete_file_data(conn, file_id)?;
                result.updated += 1;
            }
            None => {
                result.new += 1;
            }
        }

        // Parse and insert within a transaction
        let tx = conn.unchecked_transaction()?;
        let file_id = repo::upsert_file(&tx, source_id, &path_str, &hash, size_bytes)?;

        let sessions = parsers::parse_file(parser, path)
            .with_context(|| format!("parsing {}", path.display()))?;

        for session in &sessions {
            if session.messages.is_empty() {
                continue;
            }
            result.messages += u64::try_from(session.messages.len()).unwrap_or(0);
            repo::insert_session(&tx, source_id, file_id, session)?;
            result.sessions += 1;
        }

        tx.commit()?;
    }

    Ok(result)
}
