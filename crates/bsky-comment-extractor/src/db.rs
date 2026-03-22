//! `SQLite` storage layer for post records and extraction metadata.

use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

use crate::error::ExtractorError;

/// Open (or create) a `SQLite` database at the given path.
///
/// Creates all parent directories if they do not exist. Enables WAL mode
/// and foreign key enforcement on the connection.
pub fn open_db(path: &Path) -> Result<Connection, ExtractorError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )?;
    Ok(conn)
}

/// Initialize the database schema.
///
/// Creates the `posts` and `extractions` tables along with their indexes
/// if they do not already exist. Safe to call on an already-initialized
/// database (idempotent).
pub fn init_db(conn: &Connection) -> Result<(), ExtractorError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS posts (
            uri        TEXT PRIMARY KEY,
            author_did TEXT NOT NULL,
            text       TEXT NOT NULL,
            created_at TEXT NOT NULL,
            raw_json   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS extractions (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            target_did   TEXT NOT NULL,
            started_at   TEXT NOT NULL,
            completed_at TEXT,
            record_count INTEGER NOT NULL DEFAULT 0,
            last_cursor  TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_posts_author_did
            ON posts(author_did);
        CREATE INDEX IF NOT EXISTS idx_posts_created_at
            ON posts(created_at);
        CREATE INDEX IF NOT EXISTS idx_extractions_target_did
            ON extractions(target_did);
        ",
    )?;
    Ok(())
}

/// Insert or replace a post record in the `posts` table.
///
/// Uses `INSERT OR REPLACE` semantics so that inserting the same AT URI a
/// second time updates the existing row rather than producing a duplicate.
///
/// Returns `true` if the record was newly inserted, or `false` if it already
/// existed and was replaced.
pub fn upsert_post(
    conn: &Connection,
    uri: &str,
    author_did: &str,
    text: &str,
    created_at: &str,
    raw_json: &str,
) -> Result<bool, ExtractorError> {
    let is_new = !db_has_uri(conn, uri)?;
    conn.execute(
        "INSERT OR REPLACE INTO posts (uri, author_did, text, created_at, raw_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![uri, author_did, text, created_at, raw_json],
    )?;
    Ok(is_new)
}

/// Return `true` if a post with the given AT URI exists in the database.
///
/// Returns `false` for an unknown URI without returning an error.
pub fn db_has_uri(conn: &Connection, uri: &str) -> Result<bool, ExtractorError> {
    let found: Option<i32> = conn
        .query_row(
            "SELECT 1 FROM posts WHERE uri = ?1 LIMIT 1",
            params![uri],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

/// Persist the current pagination cursor for a target DID.
///
/// If an incomplete extraction row already exists for `target_did`, its
/// `last_cursor` is updated. Otherwise a new extraction row is inserted
/// with `started_at` set to the current UTC time. The `cursor` value may
/// be `None` when the last page has been reached.
pub fn save_cursor(
    conn: &Connection,
    target_did: &str,
    cursor: Option<&str>,
) -> Result<(), ExtractorError> {
    // Check for an existing incomplete extraction row.
    let existing_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM extractions
             WHERE target_did = ?1 AND completed_at IS NULL
             ORDER BY id DESC LIMIT 1",
            params![target_did],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE extractions SET last_cursor = ?2, record_count = record_count + 1
             WHERE id = ?1",
            params![id, cursor],
        )?;
    } else {
        let started_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO extractions (target_did, started_at, last_cursor, record_count)
             VALUES (?1, ?2, ?3, 0)",
            params![target_did, started_at, cursor],
        )?;
    }
    Ok(())
}

/// Load the resume cursor for the most recent incomplete extraction of `target_did`.
///
/// Returns `None` if there is no incomplete extraction row or the stored
/// cursor value is `NULL`.
pub fn load_resume_cursor(
    conn: &Connection,
    target_did: &str,
) -> Result<Option<String>, ExtractorError> {
    let cursor: Option<Option<String>> = conn
        .query_row(
            "SELECT last_cursor FROM extractions
             WHERE target_did = ?1 AND completed_at IS NULL
             ORDER BY id DESC LIMIT 1",
            params![target_did],
            |row| row.get(0),
        )
        .optional()?;
    // Flatten Option<Option<String>> -> Option<String>
    Ok(cursor.flatten())
}

/// Mark the most recent incomplete extraction for `target_did` as complete.
///
/// Sets `completed_at` to the current UTC time and records the final
/// `record_count`.
pub fn complete_extraction(
    conn: &Connection,
    target_did: &str,
    record_count: u64,
) -> Result<(), ExtractorError> {
    let completed_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE extractions
         SET completed_at = ?2, record_count = ?3
         WHERE id = (
             SELECT id FROM extractions
             WHERE target_did = ?1 AND completed_at IS NULL
             ORDER BY id DESC LIMIT 1
         )",
        params![target_did, completed_at, record_count.cast_signed()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn test_init_db_creates_tables() {
        let conn = test_db();
        let mut names: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'
                 ORDER BY name",
            )
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        names.sort();
        assert!(names.contains(&"posts".to_string()));
        assert!(names.contains(&"extractions".to_string()));
    }

    #[test]
    fn test_upsert_post_insert() {
        let conn = test_db();
        upsert_post(
            &conn,
            "at://did:plc:abc/app.bsky.feed.post/001",
            "did:plc:abc",
            "Hello world",
            "2024-01-01T00:00:00Z",
            r#"{"text":"Hello world"}"#,
        )
        .unwrap();

        let (uri, author, text, created_at, raw_json): (String, String, String, String, String) =
            conn.query_row(
                "SELECT uri, author_did, text, created_at, raw_json FROM posts LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(uri, "at://did:plc:abc/app.bsky.feed.post/001");
        assert_eq!(author, "did:plc:abc");
        assert_eq!(text, "Hello world");
        assert_eq!(created_at, "2024-01-01T00:00:00Z");
        assert_eq!(raw_json, r#"{"text":"Hello world"}"#);
    }

    #[test]
    fn test_upsert_post_idempotent() {
        let conn = test_db();
        let uri = "at://did:plc:abc/app.bsky.feed.post/001";
        upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "First",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "Updated",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM posts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            count, 1,
            "inserting same URI twice must not create a duplicate row"
        );

        let text: String = conn
            .query_row(
                "SELECT text FROM posts WHERE uri = ?1",
                params![uri],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "Updated");
    }

    #[test]
    fn test_upsert_extraction() {
        let conn = test_db();
        save_cursor(&conn, "did:plc:xyz", Some("cursor-abc")).unwrap();

        let (target_did, started_at): (String, String) = conn
            .query_row(
                "SELECT target_did, started_at FROM extractions ORDER BY id DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(target_did, "did:plc:xyz");
        assert!(!started_at.is_empty());
    }

    #[test]
    fn test_save_and_load_cursor() {
        let conn = test_db();
        save_cursor(&conn, "did:plc:xyz", Some("my-cursor-value")).unwrap();
        let loaded = load_resume_cursor(&conn, "did:plc:xyz").unwrap();
        assert_eq!(loaded, Some("my-cursor-value".to_string()));
    }

    #[test]
    fn test_load_cursor_missing() {
        let conn = test_db();
        let loaded = load_resume_cursor(&conn, "did:plc:nonexistent").unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_db_has_uri_true() {
        let conn = test_db();
        let uri = "at://did:plc:abc/app.bsky.feed.post/001";
        upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "text",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        assert!(db_has_uri(&conn, uri).unwrap());
    }

    #[test]
    fn test_db_has_uri_false() {
        let conn = test_db();
        assert!(!db_has_uri(&conn, "at://did:plc:abc/app.bsky.feed.post/999").unwrap());
    }

    #[test]
    fn test_upsert_post_returns_true_for_new() {
        let conn = test_db();
        let uri = "at://did:plc:abc/app.bsky.feed.post/new001";
        let is_new = upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "text",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        assert!(is_new, "upsert_post must return true when the URI is new");
    }

    #[test]
    fn test_upsert_post_returns_false_for_existing() {
        let conn = test_db();
        let uri = "at://did:plc:abc/app.bsky.feed.post/existing001";
        upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "first",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        let is_new = upsert_post(
            &conn,
            uri,
            "did:plc:abc",
            "updated",
            "2024-01-01T00:00:00Z",
            "{}",
        )
        .unwrap();
        assert!(
            !is_new,
            "upsert_post must return false when the URI already exists"
        );
    }

    #[test]
    fn test_open_db_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("a").join("b").join("c").join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        assert!(db_path.exists());
    }
}
