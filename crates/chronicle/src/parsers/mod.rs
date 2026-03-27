//! Parser registry and common types.

pub mod claude_code;
pub mod codex;

use crate::models::{ParsedSession, SourceFormat};
use anyhow::Result;
use std::path::Path;

/// Parse a file according to the given source format.
pub fn parse_file(format: SourceFormat, path: &Path) -> Result<Vec<ParsedSession>> {
    match format {
        SourceFormat::Codex => codex::parse_codex_file(path),
        SourceFormat::ClaudeCode => claude_code::parse_claude_code_file(path),
    }
}

/// Return the file extensions expected for a source format.
#[must_use]
pub fn expected_extensions(format: SourceFormat) -> &'static [&'static str] {
    match format {
        SourceFormat::Codex => &["jsonl"],
        SourceFormat::ClaudeCode => &["jsonl", "json"],
    }
}

/// Check whether a file path is relevant for the given source format.
///
/// Claude Code has many JSON files beyond session data (telemetry, settings, etc.).
/// This filter restricts to known data paths.
#[must_use]
pub fn is_relevant_path(format: SourceFormat, path: &Path, base_path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !expected_extensions(format).contains(&ext) {
        return false;
    }
    match format {
        SourceFormat::Codex => true,
        SourceFormat::ClaudeCode => {
            let Ok(rel) = path.strip_prefix(base_path) else {
                return false;
            };
            if ext == "jsonl" {
                // history.jsonl at root, or projects/**/*.jsonl (conversation logs + subagents)
                let is_history = path.file_name().is_some_and(|f| f == "history.jsonl")
                    && rel.components().count() == 1;
                let is_project = rel.starts_with("projects");
                return is_history || is_project;
            }
            // For .json files: sessions/*.json only
            rel.starts_with("sessions") && rel.components().count() == 2
        }
    }
}
