//! Claude Code history and session parser.
//!
//! Claude Code stores data in `~/.claude/`:
//! - `projects/{slug}/{session-uuid}.jsonl`: full conversation logs per session
//! - `projects/{slug}/{session-uuid}/subagents/agent-*.jsonl`: subagent conversations
//! - `history.jsonl`: one line per user prompt (user messages only)
//! - `sessions/*.json`: session metadata (`pid`, `sessionId`, `cwd`, `startedAt`)
//!
//! The `projects/` directory contains the primary conversation data. Each JSONL line has
//! a `type` field: `user`, `assistant`, `progress`, or `file-history-snapshot`.
//! User message content is either a string or an array of `{type: "text"/"tool_result"}`.
//! Assistant message content is an array of `{type: "text"/"thinking"/"tool_use"}`.

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use std::io::BufRead;
use std::path::Path;

use crate::models::{ParsedMessage, ParsedSession, Role};
use crate::strip::strip_markup;

/// Parse a Claude Code file. Dispatches based on path and extension.
pub fn parse_claude_code_file(path: &Path) -> Result<Vec<ParsedSession>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "jsonl" => {
            // Distinguish project conversation logs from history.jsonl
            if path.file_name().is_some_and(|f| f == "history.jsonl") {
                parse_history_jsonl(path)
            } else {
                parse_project_jsonl(path)
            }
        }
        "json" => parse_session_json(path),
        _ => Ok(Vec::new()),
    }
}

/// Parse a project session JSONL file -- full conversation with user/assistant messages.
///
/// Each line has `type` (user, assistant, progress, `file-history-snapshot`).
/// Returns a single session per file.
fn parse_project_jsonl(path: &Path) -> Result<Vec<ParsedSession>> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = std::io::BufReader::new(file);

    let mut session_id: Option<String> = None;
    let mut project: Option<String> = None;
    let mut started_at: Option<DateTime<Utc>> = None;
    let mut messages: Vec<ParsedMessage> = Vec::new();
    let mut ordinal: u32 = 0;
    let mut first_timestamp: Option<DateTime<Utc>> = None;

    for line_result in reader.lines() {
        let line = line_result.with_context(|| format!("reading {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }

        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let line_type = parsed["type"].as_str().unwrap_or("");
        let timestamp = parsed_millis_timestamp(&parsed["timestamp"]);

        // Capture session metadata from the first message
        if session_id.is_none() {
            session_id = parsed["sessionId"].as_str().map(String::from);
        }
        if project.is_none() {
            if let Some(cwd) = parsed["cwd"].as_str() {
                project = Some(cwd.to_string());
            }
        }
        if first_timestamp.is_none() {
            first_timestamp = timestamp;
        }

        match line_type {
            "user" => {
                let content = extract_message_content(&parsed["message"]);
                let stripped = strip_markup(&content);
                if !stripped.is_empty() {
                    messages.push(ParsedMessage {
                        ordinal,
                        role: Role::User,
                        content: stripped,
                        timestamp,
                        metadata: serde_json::json!({}),
                    });
                    ordinal += 1;
                }
            }
            "assistant" => {
                let content = extract_assistant_content(&parsed["message"]);
                let stripped = strip_markup(&content);
                if !stripped.is_empty() {
                    messages.push(ParsedMessage {
                        ordinal,
                        role: Role::Assistant,
                        content: stripped,
                        timestamp,
                        metadata: serde_json::json!({}),
                    });
                    ordinal += 1;
                }
            }
            // progress, file-history-snapshot -- skip
            _ => {}
        }
    }

    // Use session UUID from data, or derive from filename
    let sid = session_id.unwrap_or_else(|| {
        path.file_stem().map_or_else(
            || "unknown".to_string(),
            |s| s.to_string_lossy().to_string(),
        )
    });

    started_at = started_at.or(first_timestamp);

    if messages.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![ParsedSession {
        source_session_id: sid,
        project,
        started_at,
        metadata: serde_json::json!({"source_type": "project_conversation"}),
        messages,
    }])
}

/// Parse `history.jsonl` -- one user message per line.
///
/// Groups messages by project to form sessions. Each unique project
/// gets its own session with messages ordered by timestamp.
fn parse_history_jsonl(path: &Path) -> Result<Vec<ParsedSession>> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = std::io::BufReader::new(file);

    let mut project_messages: std::collections::HashMap<String, Vec<ParsedMessage>> =
        std::collections::HashMap::new();
    let mut project_starts: std::collections::HashMap<String, DateTime<Utc>> =
        std::collections::HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.with_context(|| format!("reading {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }

        let parsed: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let Some(display) = parsed["display"].as_str() else {
            continue;
        };

        let project = parsed["project"].as_str().unwrap_or("unknown").to_string();
        let timestamp = parsed_millis_timestamp(&parsed["timestamp"]);

        let stripped = strip_markup(display);
        if stripped.is_empty() {
            continue;
        }

        let messages = project_messages.entry(project.clone()).or_default();
        let ordinal = u32::try_from(messages.len()).unwrap_or(u32::MAX);
        messages.push(ParsedMessage {
            ordinal,
            role: Role::User,
            content: stripped,
            timestamp,
            metadata: serde_json::json!({}),
        });

        if let Some(ts) = timestamp {
            project_starts
                .entry(project)
                .and_modify(|existing| {
                    if ts < *existing {
                        *existing = ts;
                    }
                })
                .or_insert(ts);
        }
    }

    let mut sessions: Vec<ParsedSession> = project_messages
        .into_iter()
        .map(|(project, messages)| {
            let started_at = project_starts.get(&project).copied();
            ParsedSession {
                source_session_id: format!("cc-history-{}", path_hash(&project)),
                project: Some(project),
                started_at,
                metadata: serde_json::json!({"source_file": "history.jsonl"}),
                messages,
            }
        })
        .collect();

    sessions.sort_by(|a, b| a.source_session_id.cmp(&b.source_session_id));
    Ok(sessions)
}

/// Parse a session JSON file -- minimal metadata, no messages.
fn parse_session_json(path: &Path) -> Result<Vec<ParsedSession>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let parsed: Value =
        serde_json::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;

    let session_id = parsed["sessionId"]
        .as_str()
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
        })
        .to_string();

    let project = parsed["cwd"].as_str().map(String::from);
    let started_at = parsed_millis_timestamp(&parsed["startedAt"]);

    let metadata = serde_json::json!({
        "pid": parsed["pid"],
        "kind": parsed["kind"],
        "entrypoint": parsed["entrypoint"],
        "name": parsed["name"],
    });

    Ok(vec![ParsedSession {
        source_session_id: session_id,
        project,
        started_at,
        metadata,
        messages: Vec::new(),
    }])
}

/// Extract text content from a user message.
///
/// Content is either a plain string or an array of content blocks.
/// We extract text from `text` blocks and skip `tool_result` blocks.
fn extract_message_content(message: &Value) -> String {
    let content = &message["content"];
    // Plain string content
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    // Array of content blocks
    let Some(blocks) = content.as_array() else {
        return String::new();
    };
    blocks
        .iter()
        .filter_map(|block| {
            let block_type = block["type"].as_str()?;
            match block_type {
                "text" => block["text"].as_str().map(String::from),
                _ => None, // skip tool_result, etc.
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract text content from an assistant message.
///
/// Content is an array of blocks. We extract `text` blocks and `tool_use` names.
/// Skip `thinking` blocks (internal reasoning).
fn extract_assistant_content(message: &Value) -> String {
    let Some(blocks) = message["content"].as_array() else {
        return String::new();
    };
    let mut parts = Vec::new();
    for block in blocks {
        let block_type = block["type"].as_str().unwrap_or("");
        match block_type {
            "text" => {
                if let Some(text) = block["text"].as_str() {
                    parts.push(text.to_string());
                }
            }
            "tool_use" => {
                if let Some(name) = block["name"].as_str() {
                    parts.push(format!("[tool: {name}]"));
                }
            }
            // thinking, etc. -- skip
            _ => {}
        }
    }
    parts.join("\n")
}

/// Parse a JSON value as milliseconds-since-epoch timestamp.
#[allow(clippy::cast_possible_truncation)]
fn parsed_millis_timestamp(value: &Value) -> Option<DateTime<Utc>> {
    let ms = value
        .as_i64()
        .or_else(|| value.as_f64().map(|f| f as i64))?;
    Utc.timestamp_millis_opt(ms).single()
}

/// Create a short unique identifier from a string using blake3.
fn path_hash(s: &str) -> String {
    let hash = blake3::hash(s.as_bytes());
    hash.to_hex()[..16].to_string()
}
