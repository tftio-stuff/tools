//! Codex JSONL session parser.
//!
//! Codex sessions live at `~/.codex/sessions/YYYY/MM/DD/*.jsonl`.
//! One session per file.
//!
//! Line types:
//! - `session_meta`: session id, cwd, `cli_version`, instructions
//! - `event_msg`: user messages
//! - `response_item`: assistant messages, function calls, tool calls, reasoning
//! - `turn_context`: model/policy context per turn (ignored)

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::io::BufRead;
use std::path::Path;

use crate::models::{ParsedMessage, ParsedSession, Role};
use crate::strip::strip_markup;

/// Parse a single Codex JSONL file into sessions (typically one per file).
pub fn parse_codex_file(path: &Path) -> Result<Vec<ParsedSession>> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = std::io::BufReader::new(file);

    let mut session_id: Option<String> = None;
    let mut project: Option<String> = None;
    let mut started_at: Option<DateTime<Utc>> = None;
    let mut metadata = serde_json::json!({});
    let mut messages: Vec<ParsedMessage> = Vec::new();
    let mut ordinal: u32 = 0;

    for line_result in reader.lines() {
        let line = line_result.with_context(|| format!("reading {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }

        let parsed: Value = serde_json::from_str(&line)
            .with_context(|| format!("parsing JSON line in {}", path.display()))?;

        let line_type = parsed["type"].as_str().unwrap_or("");
        let timestamp = parse_timestamp(parsed["timestamp"].as_str());

        match line_type {
            "session_meta" => {
                let payload = &parsed["payload"];
                session_id = payload["id"].as_str().map(String::from);
                project = payload["cwd"].as_str().map(String::from);
                started_at = timestamp.or_else(|| parse_timestamp(payload["timestamp"].as_str()));
                metadata = serde_json::json!({
                    "cli_version": payload["cli_version"],
                    "originator": payload["originator"],
                    "model_provider": payload["model_provider"],
                });
            }
            "event_msg" => {
                let payload = &parsed["payload"];
                if let Some(text) = payload["message"].as_str() {
                    let stripped = strip_markup(text);
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
            }
            "response_item" => {
                let payload = &parsed["payload"];
                let item_type = payload["type"].as_str().unwrap_or("");

                match item_type {
                    "message" => {
                        let role = match payload["role"].as_str().unwrap_or("assistant") {
                            "user" => Role::User,
                            "system" => Role::System,
                            // "assistant" and anything else
                            _ => Role::Assistant,
                        };
                        let content = extract_content_text(payload);
                        let stripped = strip_markup(&content);
                        if !stripped.is_empty() {
                            messages.push(ParsedMessage {
                                ordinal,
                                role,
                                content: stripped,
                                timestamp,
                                metadata: serde_json::json!({}),
                            });
                            ordinal += 1;
                        }
                    }
                    "function_call" | "custom_tool_call" => {
                        let name = payload["name"].as_str().unwrap_or("unknown");
                        let args = payload["arguments"]
                            .as_str()
                            .or_else(|| payload["input"].as_str())
                            .unwrap_or("");
                        messages.push(ParsedMessage {
                            ordinal,
                            role: Role::Tool,
                            content: format!("[{name}] {args}"),
                            timestamp,
                            metadata: serde_json::json!({"tool_name": name}),
                        });
                        ordinal += 1;
                    }
                    "function_call_output" | "custom_tool_call_output" => {
                        let output = payload["output"].as_str().unwrap_or("");
                        let stripped = strip_markup(output);
                        if !stripped.is_empty() {
                            messages.push(ParsedMessage {
                                ordinal,
                                role: Role::Tool,
                                content: stripped,
                                timestamp,
                                metadata: serde_json::json!({"type": "tool_output"}),
                            });
                            ordinal += 1;
                        }
                    }
                    // reasoning, ghost_snapshot, turn_context -- skip
                    _ => {}
                }
            }
            // turn_context and unknown types -- skip
            _ => {}
        }
    }

    let sid = session_id.unwrap_or_else(|| {
        path.file_stem().map_or_else(
            || "unknown".to_string(),
            |s| s.to_string_lossy().to_string(),
        )
    });

    if messages.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![ParsedSession {
        source_session_id: sid,
        project,
        started_at,
        metadata,
        messages,
    }])
}

/// Extract text from a content array.
fn extract_content_text(payload: &Value) -> String {
    let Some(content) = payload["content"].as_array() else {
        return String::new();
    };
    content
        .iter()
        .filter_map(|item| item["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse an ISO 8601 timestamp string.
fn parse_timestamp(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}
