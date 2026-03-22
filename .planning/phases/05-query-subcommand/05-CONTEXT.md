# Phase 5: Query Subcommand - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `query` subcommand to `bce` that reads stored posts from the local SQLite database and outputs paginated JSONL to stdout. Also restructure the CLI from flat invocation to subcommands (`fetch`, `query`), and add a `--agent-help` top-level flag. The query subcommand is read-only against the local DB -- it never fetches from the network.

</domain>

<decisions>
## Implementation Decisions

### CLI structure
- Switch from flat CLI to subcommands: `bce fetch <handle>` and `bce query`
- `bce` alone (no subcommand) shows help with subcommand list
- `--agent-help` is a top-level flag (`bce --agent-help`), not a subcommand
- `fetch` inherits all current flags: `--db`, `--since`, `-q`
- `query` has only `--db`, `--limit`, `--offset` -- no `--since` or other filters
- This is a breaking change to the CLI interface: `bce <handle>` no longer works, must use `bce fetch <handle>`

### JSON output shape
- JSONL format: first line is metadata envelope, subsequent lines are one post per line
- Curated fields per post: `uri`, `author_did`, `text`, `created_at` (the 4 DB columns only)
- No `raw_json` field in output -- agents get clean structured data
- No `reply_parent` extraction -- agents use the 4 core fields
- Envelope fields: `total` (int), `offset` (int), `limit` (int), `has_more` (bool)

### Output behavior
- Query output goes exclusively to stdout (no `--output` flag)
- Errors as JSON objects on stderr: `{"error": "db_not_found", "message": "..."}`
- Non-zero exit code on errors
- Empty DB (zero posts) is not an error: output envelope with `total: 0, has_more: false`, exit 0
- Fetch subcommand keeps current spinner behavior unchanged (stderr when TTY, suppressed with `-q` or non-TTY)

### Pagination defaults
- `--limit` default: 50
- `--offset` default: 0
- Posts ordered by `created_at` descending (newest first)

### Claude's Discretion
- Exact error code strings (e.g., "db_not_found", "invalid_offset")
- Exit code numbers for different error types
- Internal query SQL optimization
- Whether to add `--count` flag for total-only queries (no posts)

</decisions>

<canonical_refs>
## Canonical References

No external specs -- requirements fully captured in decisions above.

### Existing implementation
- `crates/bsky-comment-extractor/src/cli.rs` -- Current flat CLI struct, must be restructured to subcommands
- `crates/bsky-comment-extractor/src/db.rs` -- SQLite functions (open_db, init_db, upsert_post); no query function exists yet
- `crates/bsky-comment-extractor/src/main.rs` -- Current entry point with sync main + RuntimeBuilder pattern
- `crates/bsky-comment-extractor/src/models.rs` -- FetchSummary struct; will need QueryResult or similar for query output

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `db.rs::open_db(path)` -- Opens SQLite connection at any path with parent dir creation
- `cli.rs::Cli` -- Current clap Parser; will be restructured into subcommand enums
- `main.rs::default_db_path()` -- XDG path resolution via `directories::ProjectDirs`; reusable for query's `--db` default
- `main.rs::make_spinner()` -- TTY-aware spinner; not needed for query but preserved for fetch

### Established Patterns
- Sync `fn main()` + `tokio::runtime::Builder::new_current_thread()` -- query doesn't need async (pure SQLite), but fetch still does
- `anyhow::Result` for error propagation in main
- `serde::Serialize` on models for JSON output (already on FetchSummary)
- Workspace lints: `missing_docs = "deny"`, `clippy::pedantic = "deny"`

### Integration Points
- `main.rs` routes to fetch or query based on subcommand match
- `db.rs` gets new `query_posts(conn, limit, offset)` function returning Vec of post structs
- `models.rs` gets serializable post output struct and envelope struct
- `cli.rs` restructured: top-level `Cli` with `#[command(subcommand)]`, `FetchArgs` and `QueryArgs` substructs

</code_context>

<specifics>
## Specific Ideas

- The primary consumer is LLM agents running `bce` as a tool -- output must be deterministic and parseable without ambiguity
- JSONL chosen over single JSON object for streamability -- agents can process line by line
- First line is always metadata, even for empty results -- agents don't need conditional parsing logic

</specifics>

<deferred>
## Deferred Ideas

- `--since` filter on query -- agents filter client-side for now
- `--output <file>` flag -- agents use shell redirection
- Raw AT Protocol JSON in output -- curated fields only for now
- Reply parent extraction -- keep output minimal
- `--count` flag for total-only queries -- Claude's discretion

</deferred>

---

*Phase: 05-query-subcommand*
*Context gathered: 2026-03-22*
