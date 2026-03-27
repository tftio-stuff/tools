//! Database repository — all SQL queries.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::{ParsedMessage, ParsedSession, SourceFormat};

/// A registered source row.
#[derive(Debug, Clone)]
pub struct SourceRow {
    /// Primary key.
    pub id: i64,
    /// Unique name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Default ingest path.
    pub base_path: Option<String>,
    /// Parser format identifier.
    pub parser: String,
    /// When the source was created.
    pub created_at: String,
}

/// A file row.
#[derive(Debug, Clone)]
pub struct FileRow {
    /// Primary key.
    pub id: i64,
    /// Source foreign key.
    pub source_id: i64,
    /// File path on disk.
    pub path: String,
    /// Blake3 hash.
    pub file_hash: String,
    /// File size.
    pub size_bytes: i64,
}

/// A session row for display.
#[derive(Debug, Clone)]
pub struct SessionRow {
    /// Session identifier.
    pub id: String,
    /// Source foreign key.
    pub source_id: i64,
    /// Project path or name.
    pub project: Option<String>,
    /// Start timestamp.
    pub started_at: Option<String>,
    /// Message count (populated by joins).
    pub message_count: i64,
}

/// A message row for search results.
#[derive(Debug, Clone)]
pub struct MessageRow {
    /// Primary key.
    pub id: i64,
    /// Session foreign key.
    pub session_id: String,
    /// Position in session.
    pub ordinal: i64,
    /// Message role.
    pub role: String,
    /// Text content.
    pub content: String,
    /// Timestamp.
    pub timestamp: Option<String>,
}

/// Ensure a source exists. Returns its id.
pub fn ensure_source(
    conn: &Connection,
    name: &str,
    parser: SourceFormat,
    base_path: Option<&str>,
    description: Option<&str>,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO sources (name, parser, base_path, description, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, parser.as_str(), base_path, description, now],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM sources WHERE name = ?1",
        params![name],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// List all sources.
pub fn list_sources(conn: &Connection) -> Result<Vec<SourceRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, base_path, parser, created_at FROM sources ORDER BY name",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SourceRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                base_path: row.get(3)?,
                parser: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Get a source by name.
pub fn get_source_by_name(conn: &Connection, name: &str) -> Result<Option<SourceRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, base_path, parser, created_at FROM sources WHERE name = ?1",
    )?;
    let mut rows = stmt.query_map(params![name], |row| {
        Ok(SourceRow {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            base_path: row.get(3)?,
            parser: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Look up a file by source and path. Returns `(id, hash)` if found.
pub fn get_file(conn: &Connection, source_id: i64, path: &str) -> Result<Option<(i64, String)>> {
    let mut stmt =
        conn.prepare("SELECT id, file_hash FROM files WHERE source_id = ?1 AND path = ?2")?;
    let mut rows = stmt.query_map(params![source_id, path], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Insert or update a file row. Returns the file id.
pub fn upsert_file(
    conn: &Connection,
    source_id: i64,
    path: &str,
    file_hash: &str,
    size_bytes: i64,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO files (source_id, path, file_hash, size_bytes, ingested_at) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT(source_id, path) DO UPDATE SET \
         file_hash = excluded.file_hash, \
         size_bytes = excluded.size_bytes, \
         ingested_at = excluded.ingested_at",
        params![source_id, path, file_hash, size_bytes, now],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM files WHERE source_id = ?1 AND path = ?2",
        params![source_id, path],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Delete all sessions and messages for a given file.
pub fn delete_file_data(conn: &Connection, file_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE session_id IN \
         (SELECT id FROM sessions WHERE file_id = ?1)",
        params![file_id],
    )?;
    conn.execute("DELETE FROM sessions WHERE file_id = ?1", params![file_id])?;
    Ok(())
}

/// Delete all data for a source (files, sessions, messages).
pub fn delete_source_data(conn: &Connection, source_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM messages WHERE session_id IN \
         (SELECT id FROM sessions WHERE source_id = ?1)",
        params![source_id],
    )?;
    conn.execute(
        "DELETE FROM sessions WHERE source_id = ?1",
        params![source_id],
    )?;
    conn.execute("DELETE FROM files WHERE source_id = ?1", params![source_id])?;
    Ok(())
}

/// Insert a parsed session and its messages.
///
/// The stored session ID combines the source session ID with the file ID
/// to handle cases where the same logical session appears in multiple files
/// (e.g., Codex session continuations).
pub fn insert_session(
    conn: &Connection,
    source_id: i64,
    file_id: i64,
    session: &ParsedSession,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let started_at = session.started_at.map(|dt| dt.to_rfc3339());
    let metadata = serde_json::to_string(&session.metadata)?;
    // Combine session ID with file_id for uniqueness across files
    let unique_id = format!("{}:{file_id}", session.source_session_id);

    conn.execute(
        "INSERT INTO sessions (id, source_id, file_id, project, started_at, metadata, ingested_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            unique_id,
            source_id,
            file_id,
            session.project,
            started_at,
            metadata,
            now
        ],
    )
    .with_context(|| format!("inserting session {}", session.source_session_id))?;

    insert_messages(conn, &unique_id, &session.messages)?;
    Ok(())
}

/// Insert messages for a session.
fn insert_messages(conn: &Connection, session_id: &str, messages: &[ParsedMessage]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO messages (session_id, ordinal, role, content, timestamp, metadata) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )?;
    for msg in messages {
        let timestamp = msg.timestamp.map(|dt| dt.to_rfc3339());
        let metadata = serde_json::to_string(&msg.metadata)?;
        stmt.execute(params![
            session_id,
            msg.ordinal,
            msg.role.as_str(),
            msg.content,
            timestamp,
            metadata
        ])?;
    }
    Ok(())
}

/// Full-text search on message content.
pub fn search_messages(
    conn: &Connection,
    query: &str,
    source_name: Option<&str>,
    role: Option<&str>,
    project: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let mut sql = String::from(
        "SELECT m.id, m.session_id, m.ordinal, m.role, m.content, m.timestamp \
         FROM messages m \
         JOIN messages_fts fts ON fts.rowid = m.id \
         JOIN sessions s ON s.id = m.session_id",
    );

    // Build params and conditions together
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    param_values.push(Box::new(query.to_string()));
    let mut conditions = vec!["fts.messages_fts MATCH ?1".to_string()];

    if let Some(src) = source_name {
        sql.push_str(" JOIN sources src ON src.id = s.source_id");
        param_values.push(Box::new(src.to_string()));
        conditions.push(format!("src.name = ?{}", param_values.len()));
    }
    if let Some(r) = role {
        param_values.push(Box::new(r.to_string()));
        conditions.push(format!("m.role = ?{}", param_values.len()));
    }
    if let Some(p) = project {
        param_values.push(Box::new(p.to_string()));
        conditions.push(format!("s.project = ?{}", param_values.len()));
    }
    param_values.push(Box::new(limit));
    let limit_idx = param_values.len();

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));
    sql.push_str(" ORDER BY fts.rank LIMIT ?");
    sql.push_str(&limit_idx.to_string());

    let mut stmt = conn.prepare(&sql)?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| &**p).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(MessageRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                ordinal: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Stats for a single source.
#[derive(Debug, Clone)]
pub struct SourceStats {
    /// Source name.
    pub name: String,
    /// Number of indexed files.
    pub file_count: i64,
    /// Number of sessions.
    pub session_count: i64,
    /// Number of messages.
    pub message_count: i64,
    /// Earliest session start.
    pub earliest: Option<String>,
    /// Latest session start.
    pub latest: Option<String>,
}

/// Compute stats per source (or a single source).
pub fn compute_stats(conn: &Connection, source_name: Option<&str>) -> Result<Vec<SourceStats>> {
    let sql = if source_name.is_some() {
        "SELECT src.name, \
             (SELECT COUNT(*) FROM files f WHERE f.source_id = src.id), \
             (SELECT COUNT(*) FROM sessions s WHERE s.source_id = src.id), \
             (SELECT COUNT(*) FROM messages m JOIN sessions s2 ON s2.id = m.session_id WHERE s2.source_id = src.id), \
             (SELECT MIN(s3.started_at) FROM sessions s3 WHERE s3.source_id = src.id), \
             (SELECT MAX(s4.started_at) FROM sessions s4 WHERE s4.source_id = src.id) \
         FROM sources src WHERE src.name = ?1"
    } else {
        "SELECT src.name, \
             (SELECT COUNT(*) FROM files f WHERE f.source_id = src.id), \
             (SELECT COUNT(*) FROM sessions s WHERE s.source_id = src.id), \
             (SELECT COUNT(*) FROM messages m JOIN sessions s2 ON s2.id = m.session_id WHERE s2.source_id = src.id), \
             (SELECT MIN(s3.started_at) FROM sessions s3 WHERE s3.source_id = src.id), \
             (SELECT MAX(s4.started_at) FROM sessions s4 WHERE s4.source_id = src.id) \
         FROM sources src ORDER BY src.name"
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(name) = source_name {
        stmt.query_map(params![name], |row| {
            Ok(SourceStats {
                name: row.get(0)?,
                file_count: row.get(1)?,
                session_count: row.get(2)?,
                message_count: row.get(3)?,
                earliest: row.get(4)?,
                latest: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], |row| {
            Ok(SourceStats {
                name: row.get(0)?,
                file_count: row.get(1)?,
                session_count: row.get(2)?,
                message_count: row.get(3)?,
                earliest: row.get(4)?,
                latest: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?
    };
    Ok(rows)
}

/// Get all messages for a session, ordered by ordinal.
pub fn get_session_messages(conn: &Connection, session_id: &str) -> Result<Vec<MessageRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, ordinal, role, content, timestamp \
         FROM messages WHERE session_id = ?1 ORDER BY ordinal",
    )?;
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(MessageRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                ordinal: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Get session metadata by id.
pub fn get_session(conn: &Connection, session_id: &str) -> Result<Option<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.source_id, s.project, s.started_at, \
         (SELECT COUNT(*) FROM messages m WHERE m.session_id = s.id) \
         FROM sessions s WHERE s.id = ?1",
    )?;
    let mut rows = stmt.query_map(params![session_id], |row| {
        Ok(SessionRow {
            id: row.get(0)?,
            source_id: row.get(1)?,
            project: row.get(2)?,
            started_at: row.get(3)?,
            message_count: row.get(4)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}
