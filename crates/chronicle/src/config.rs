//! Configuration and path resolution.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Resolve the database path: `~/.local/share/chronicle/db.sqlite`.
pub fn resolve_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().context("cannot determine data directory")?;
    Ok(data_dir.join("chronicle").join("db.sqlite"))
}

/// Ensure the parent directory for the database exists.
pub fn ensure_db_dir(db_path: &std::path::Path) -> Result<()> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create directory: {}", parent.display()))?;
    }
    Ok(())
}
