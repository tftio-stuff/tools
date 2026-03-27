# Chronicle: Interaction History Corpus Indexer

## Purpose

A tool for indexing interaction histories from coding agents (Codex, Claude Code, Claude web) into a queryable SQLite database. Provides a discovery surface for agentic analysis of past interactions -- agents search indexed content to find relevant sessions, then read source files for full context when needed.

The indexed content is deliberately lossy: stripped of markup, system prompts, and tool-call detail. Source files are ground truth. The DB is a discovery layer, not a replacement for the corpus.

## Crate

- Directory: `crates/chronicle/`
- Package: `tftio-chronicle`
- Binary: `chronicle`
- DB location: `~/.local/share/chronicle/db.sqlite`

## Schema

```sql
CREATE TABLE sources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    base_path   TEXT,
    parser      TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE files (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id   INTEGER NOT NULL REFERENCES sources(id),
    path        TEXT NOT NULL,
    file_hash   TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL,
    ingested_at TEXT NOT NULL,
    UNIQUE(source_id, path)
);

CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    source_id   INTEGER NOT NULL REFERENCES sources(id),
    file_id     INTEGER NOT NULL REFERENCES files(id),
    project     TEXT,
    started_at  TEXT,
    metadata    TEXT,
    summary     TEXT,
    vector      BLOB,
    ingested_at TEXT NOT NULL
);

CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    ordinal     INTEGER NOT NULL,
    role        TEXT NOT NULL,
    content     TEXT NOT NULL,
    timestamp   TEXT,
    metadata    TEXT,
    vector      BLOB,
    UNIQUE(session_id, ordinal)
);

CREATE TABLE messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='id'
);
```

### Notes

- `sources.parser` maps to a `SourceFormat` enum variant.
- `sources.base_path` stores the default ingest path so `reindex` works without re-specifying.
- `files.file_hash` is blake3. Used for incremental ingest.
- `sessions.metadata` and `messages.metadata` are JSON blobs for source-specific extras.
- `vector` columns are reserved (NULL) until an embedding strategy is chosen.
- No `raw_content` column. Source files are available via `files.path` for ground-truth reads.

## Parser Architecture

```
crates/chronicle/src/parsers/
    mod.rs          -- SourceFormat enum, registry, common types
    codex.rs        -- Codex JSONL sessions
    claude_code.rs  -- Claude Code history.jsonl + session JSON files
```

### Common Types

```rust
pub struct ParsedSession {
    pub source_session_id: String,
    pub project: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub messages: Vec<ParsedMessage>,
}

pub struct ParsedMessage {
    pub ordinal: u32,
    pub role: Role,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

pub enum SourceFormat {
    Codex,
    ClaudeCode,
}

impl SourceFormat {
    pub fn parse_file(&self, path: &Path) -> Result<Vec<ParsedSession>>;
}
```

### Content Stripping

The `content` field strips system reminders, XML tags, and tool-call markup to produce clean text for FTS indexing. Stripping is best-effort and format-specific -- each parser knows what to strip.

### Source Formats

**Codex** (`~/.codex/sessions/YYYY/MM/DD/*.jsonl`):
- One session per file.
- First line: `session_meta` with id, cwd, cli_version, instructions.
- Subsequent lines: `response_item` with role, content array.
- Content items have `type` (input_text, output_text, etc.) and `text` fields.

**Claude Code** (`~/.claude/`):
- `history.jsonl`: one line per prompt. Fields: `display`, `timestamp`, `project`. User messages only -- no assistant responses.
- `sessions/*.json`: minimal session metadata (pid, sessionId, cwd, startedAt).
- These are complementary: history.jsonl provides the user-side corpus, session files provide metadata linkage.

## CLI Commands

```
chronicle ingest --source <name> [--path <dir>] [--parser <format>] [--full]
chronicle reindex [--source <name>] [--full]
chronicle search <query> [--source <name>] [--role <role>] [--project <path>] [--limit N]
chronicle stats [--source <name>]
chronicle export --session <id> [--format json|markdown]
chronicle sources list
chronicle sources add --name <name> --parser <format> --path <dir> [--description <text>]
```

### ingest

1. Look up or auto-create the `sources` row.
2. Walk source path with `walkdir`, filtered by expected extensions.
3. Per file: compute blake3 hash, compare against `files` table.
   - New file: parse and insert.
   - Hash changed: delete old sessions/messages for that file_id, re-parse, insert, update hash.
   - Hash matches: skip.
4. Transaction per file: insert/update file row, insert sessions, insert messages, update FTS.
5. Report: files scanned, new, updated, skipped.

### reindex

Re-ingests all sources (or one with `--source`) using stored `base_path`. `--full` drops all data for the source and rebuilds.

### search

FTS5 query against message content. Filters by source, role, project. Returns matching messages with session context. Default output is a table; `--json` for machine consumption.

### stats

Counts per source: files, sessions, messages. Date ranges. Largest sessions.

### export

Dump a full session as JSON or reconstructed markdown.

### sources

Manage the sources table. `list` shows registered sources. `add` registers a new one.

## Incremental Ingest

Idempotency guarantee: running `ingest` twice with no file changes is a no-op. New files are picked up automatically. Changed files (e.g., appended-to JSONL) are detected by hash change and fully re-ingested.

`--full` flag on either `ingest` or `reindex` drops and rebuilds from scratch.

## Workspace Integration

- Workspace dependencies: `rusqlite` (bundled), `clap`, `serde`, `serde_json`, `chrono`, `blake3`, `walkdir`
- New dependency: `sqlite-vec` (vector column support, queries deferred until embeddings exist)
- `cli-common` integration: standard metadata commands, doctor checks, completions, output formatting
- Workspace lints: inherit defaults, no overrides

### Doctor Checks

- DB exists and is readable
- Source/session/message counts
- Flag registered sources whose `base_path` no longer exists

## Scope for Initial Implementation

Parsers: Codex and Claude Code only. Claude web export parser deferred.

Vector computation: schema reserves columns, no embedding logic yet. Backfill via future `chronicle embed` command.

## Design Principle: Discovery Surface

Agents use chronicle to find relevant context, then read source files for full detail. This parallels Silent Critic's differential acceptance surface -- the indexed content is what agents search against, but the source files are ground truth. The deliberate information loss in stripping is a feature: it forces a second deliberate act to access full context, preventing shallow pattern-matching on raw markup.
