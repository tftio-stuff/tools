# Phase 4: CLI Surface - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Thin CLI wrapper around the `run_extraction` library function from Phase 3. Handles argument parsing, credential validation, progress display, and workspace integration (`just ci` compliance). This phase does NOT change the extraction logic.

</domain>

<decisions>
## Implementation Decisions

### Invocation design
- Flat invocation, no subcommands: `bce alice.bsky.social`
- Binary name: `bce` (crate remains `bsky-comment-extractor`, package `tftio-bsky-comment-extractor`)
- Handle as positional argument (required)
- `--db <path>` for database path override
- `--since <date>` for date cutoff, human-friendly via `dateparser` (accepts "2025-01-01", "3 months ago", etc.)
- `-q` / `--quiet` flag suppresses progress bar

### XDG paths
- Default database location: `~/.local/share/bce/bsky-posts.db` via `directories` crate (`ProjectDirs::data_dir()`)
- `--db` overrides the default path
- Create parent directories automatically if they don't exist

### Progress & output
- `indicatif` spinner with live record count during extraction: "Fetching posts for alice.bsky.social... 2,450 records"
- Spinner is a spinner (not a progress bar) because total record count is unknown upfront
- Auto-suppress spinner when output is not a TTY (piped) via `is-terminal`
- `-q` / `--quiet` also suppresses spinner
- On completion: single summary line to stdout: "Extracted 2,450 posts for alice.bsky.social to ~/.local/share/bce/bsky-posts.db (1,200 new, 1,250 existing)"

### Credential handling
- `BSKY_APP_PASSWORD` is REQUIRED. The CLI errors out if not set.
- No public API fallback in v1 — drop the unauthenticated path from the CLI layer
- Error message: "Error: BSKY_APP_PASSWORD not set. Create an app password at https://bsky.app/settings/app-passwords"
- `--help` includes `after_help` text explaining how to create and set the app password
- Note: The library layer (`run_extraction`) still supports unauthenticated mode — the CLI simply doesn't expose it

### Claude's Discretion
- Exact spinner style (indicatif template)
- How to wire the `indicatif` progress into the async extraction loop (callback, channel, or Arc counter)
- `tracing-subscriber` initialization (if any logging beyond the spinner)
- Exit codes (0 = success, 1 = error, convention from workspace)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 3 library API (what the CLI wraps)
- `crates/bsky-comment-extractor/src/lib.rs` — `run_extraction(handle, db_path, since)` signature and doc comments
- `crates/bsky-comment-extractor/src/models.rs` — `FetchSummary` struct (return value with counts for the summary line)
- `crates/bsky-comment-extractor/src/error.rs` — `ExtractorError` variants for error message formatting

### Workspace CLI patterns (reference implementations)
- `crates/todoer/src/cli.rs` — Simple clap Parser pattern (closest match for flat CLI)
- `crates/todoer/src/main.rs` — Sync main with exit codes, error formatting pattern
- `crates/prompter/src/main.rs` — indicatif progress bar usage (the workspace reference for progress display)

### Workspace infrastructure
- `Cargo.toml` (root) — Workspace dependency declarations; `dateparser`, `directories`, `indicatif`, `is-terminal` all present
- `crates/cli-common/src/lib.rs` — Shared completions, doctor, output utilities

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `indicatif` (workspace dep) — Used by `prompter` for progress bars; reuse for extraction spinner
- `is-terminal` (workspace dep) — Used by `cli-common` for TTY detection; reuse for auto-quiet
- `dateparser` (workspace dep) — Used by `asana-cli` for human-friendly date parsing; reuse for `--since`
- `directories` (workspace dep) — Used by `asana-cli` for `ProjectDirs`; reuse for XDG data dir
- `tftio-cli-common` — Shell completions generator, could integrate for `bce` completions

### Established Patterns
- `todoer` CLI: `clap::Parser`, `#[command(name = "...")]`, sync `fn main()` calling `run()` returning exit code
- `asana-cli`: `ProjectDirs::from("com", "tftio", "asana-cli")` for XDG dirs
- All workspace binaries: `thiserror` for typed errors, `anyhow` for CLI-level error handling

### Integration Points
- Root `Cargo.toml` already has `bsky-comment-extractor` in workspace members
- `release-please-config.json` — may need entry for the new binary
- `crates/bsky-comment-extractor/Cargo.toml` — add `dateparser`, `directories`, `indicatif`, `is-terminal`, `clap` as dependencies

</code_context>

<specifics>
## Specific Ideas

- The `run_extraction` function currently takes `Option<DateTime<Utc>>` for since — the CLI parses human-friendly dates via `dateparser` and converts to `DateTime<Utc>` before calling the library
- The `FetchSummary` struct from Phase 3 provides counts needed for the completion summary line — check its fields and extend if needed (may need "new vs existing" breakdown)
- The library's unauthenticated path still works but the CLI doesn't expose it — a future `--public` flag could re-enable it

</specifics>

<deferred>
## Deferred Ideas

- Public API fallback (`--public` flag) — user explicitly chose to defer this
- `--json` flag for machine-readable output — possible v2 addition
- Shell completions via `cli-common` — nice-to-have, not v1

</deferred>

---

*Phase: 04-cli-surface*
*Context gathered: 2026-03-22*
