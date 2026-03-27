//! Domain types for parsed sessions and messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A role in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Human user.
    User,
    /// AI assistant.
    Assistant,
    /// System prompt or instruction.
    System,
    /// Tool invocation or result.
    Tool,
}

impl Role {
    /// String representation for storage.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::Tool => "tool",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            "tool" => Ok(Self::Tool),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

/// Source format identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    /// Codex CLI sessions.
    Codex,
    /// Anthropic Claude Code sessions.
    ClaudeCode,
}

impl SourceFormat {
    /// String representation for storage.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude_code",
        }
    }
}

impl fmt::Display for SourceFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SourceFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "codex" => Ok(Self::Codex),
            "claude_code" | "claude-code" => Ok(Self::ClaudeCode),
            other => Err(format!("unknown source format: {other}")),
        }
    }
}

/// A parsed session extracted from a source file.
#[derive(Debug, Clone)]
pub struct ParsedSession {
    /// Identifier from the source (session UUID, filename, etc.).
    pub source_session_id: String,
    /// Project directory or name, if known.
    pub project: Option<String>,
    /// When the session started.
    pub started_at: Option<DateTime<Utc>>,
    /// Source-specific metadata as JSON.
    pub metadata: serde_json::Value,
    /// Ordered messages in this session.
    pub messages: Vec<ParsedMessage>,
}

/// A single message within a session.
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    /// Zero-based position in the session.
    pub ordinal: u32,
    /// Who sent this message.
    pub role: Role,
    /// Stripped text content for FTS indexing.
    pub content: String,
    /// When the message was sent.
    pub timestamp: Option<DateTime<Utc>>,
    /// Source-specific metadata as JSON.
    pub metadata: serde_json::Value,
}
