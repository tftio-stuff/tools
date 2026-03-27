//! Export command: dump a session as JSON or markdown.

use anyhow::{Result, bail};
use rusqlite::Connection;
use serde_json::json;
use std::fmt::Write;

use crate::repo;

/// Export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON output.
    Json,
    /// Reconstructed markdown.
    Markdown,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            other => Err(format!("unknown export format: {other}")),
        }
    }
}

/// Export a session.
pub fn run_export(conn: &Connection, session_id: &str, format: ExportFormat) -> Result<String> {
    let session = repo::get_session(conn, session_id)?;
    let Some(session) = session else {
        bail!("session not found: {session_id}");
    };

    let messages = repo::get_session_messages(conn, session_id)?;

    match format {
        ExportFormat::Json => {
            let msg_json: Vec<_> = messages
                .iter()
                .map(|m| {
                    json!({
                        "ordinal": m.ordinal,
                        "role": m.role,
                        "content": m.content,
                        "timestamp": m.timestamp,
                    })
                })
                .collect();

            let output = json!({
                "session_id": session.id,
                "project": session.project,
                "started_at": session.started_at,
                "message_count": session.message_count,
                "messages": msg_json,
            });

            Ok(serde_json::to_string_pretty(&output)?)
        }
        ExportFormat::Markdown => {
            let mut md = String::new();
            let _ = write!(md, "# Session: {}\n\n", session.id);
            if let Some(ref project) = session.project {
                let _ = write!(md, "**Project:** {project}\n\n");
            }
            if let Some(ref started) = session.started_at {
                let _ = write!(md, "**Started:** {started}\n\n");
            }
            md.push_str("---\n\n");

            for msg in &messages {
                let _ = write!(md, "### {} ({})\n\n", msg.role, msg.ordinal);
                md.push_str(&msg.content);
                md.push_str("\n\n");
            }

            Ok(md)
        }
    }
}
