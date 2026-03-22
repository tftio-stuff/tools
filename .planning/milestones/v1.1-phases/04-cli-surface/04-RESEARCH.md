# Phase 4: CLI Surface - Research

**Researched:** 2026-03-22
**Domain:** Rust CLI binary wrapping an async library (clap, indicatif, directories, dateparser)
**Confidence:** HIGH

## Summary

Phase 4 is a thin CLI wrapper around the `run_extraction` async library function from Phase 3. All
dependencies are already declared in the workspace root `Cargo.toml`. The workspace has established
patterns for every required capability: flat clap parsing (todoer), sync-main-with-tokio-runtime
(asana-cli), indicatif spinner with TTY detection (prompter), and ProjectDirs for XDG paths
(asana-cli). No new patterns need to be invented.

The one gap between the CONTEXT.md summary line spec ("1,200 new, 1,250 existing") and the current
Phase 3 `FetchSummary` struct is that `FetchSummary` only carries `count: u64` (total processed)
and `done: bool`. The `new_count` and `existing_count` breakdown is not tracked. The implementation
plan must extend `FetchSummary` (or the client loop) to track how many posts were actually inserted
versus updated. This is a contained change to `models.rs` and `client.rs`.

The async boundary requires `tokio::runtime::Builder::new_current_thread().enable_all().build()`
with `runtime.block_on(...)` inside a sync `fn main()`. The `#[tokio::main]` attribute macro is NOT
the workspace pattern -- asana-cli constructs the runtime manually.

**Primary recommendation:** Follow asana-cli's async-in-sync-main pattern; follow prompter's
indicatif spinner pattern; wire a progress callback (or shared `Arc<AtomicU64>`) through to the
spinner. Extend `FetchSummary` with `new_count` and `existing_count` fields before wiring the
summary line.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Invocation design:**
- Flat invocation, no subcommands: `bce alice.bsky.social`
- Binary name: `bce` (crate remains `bsky-comment-extractor`, package `tftio-bsky-comment-extractor`)
- Handle as positional argument (required)
- `--db <path>` for database path override
- `--since <date>` for date cutoff, human-friendly via `dateparser` (accepts "2025-01-01", "3 months ago", etc.)
- `-q` / `--quiet` flag suppresses progress bar

**XDG paths:**
- Default database location: `~/.local/share/bce/bsky-posts.db` via `directories` crate (`ProjectDirs::data_dir()`)
- `--db` overrides the default path
- Create parent directories automatically if they don't exist

**Progress and output:**
- `indicatif` spinner with live record count during extraction: "Fetching posts for alice.bsky.social... 2,450 records"
- Spinner is a spinner (not a progress bar) because total record count is unknown upfront
- Auto-suppress spinner when output is not a TTY (piped) via `is-terminal`
- `-q` / `--quiet` also suppresses spinner
- On completion: single summary line to stdout: "Extracted 2,450 posts for alice.bsky.social to ~/.local/share/bce/bsky-posts.db (1,200 new, 1,250 existing)"

**Credential handling:**
- `BSKY_APP_PASSWORD` is REQUIRED. The CLI errors out if not set.
- No public API fallback in v1 -- drop the unauthenticated path from the CLI layer
- Error message: "Error: BSKY_APP_PASSWORD not set. Create an app password at https://bsky.app/settings/app-passwords"
- `--help` includes `after_help` text explaining how to create and set the app password
- Note: The library layer (`run_extraction`) still supports unauthenticated mode -- the CLI simply doesn't expose it

### Claude's Discretion

- Exact spinner style (indicatif template)
- How to wire the `indicatif` progress into the async extraction loop (callback, channel, or Arc counter)
- `tracing-subscriber` initialization (if any logging beyond the spinner)
- Exit codes (0 = success, 1 = error, convention from workspace)

### Deferred Ideas (OUT OF SCOPE)

- Public API fallback (`--public` flag) -- user explicitly chose to defer this
- `--json` flag for machine-readable output -- possible v2 addition
- Shell completions via `cli-common` -- nice-to-have, not v1
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLI-01 | Accept user handle or DID as positional argument | clap positional `handle: String` with `#[arg()]`; dateparser for --since |
| CLI-02 | `--db` flag for database path | `--db <path>` optional arg; default from `ProjectDirs::data_dir().join("bsky-posts.db")` |
| CLI-03 | Progress indicator during extraction | indicatif ProgressBar::new_spinner() + is_terminal() guard; requires wiring progress into async loop |
| CLI-04 | Follow workspace conventions (clap, cli-common integration) | clap 4 with derive, sync fn main(), tokio runtime block_on, workspace lints, [[bin]] in Cargo.toml |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4 (workspace) | Argument parsing | Workspace standard; all binaries use it |
| indicatif | 0.18 (workspace) | Spinner/progress display | Workspace standard; prompter reference impl |
| is-terminal | 0.4 (workspace) | TTY detection for auto-quiet | Workspace standard; cli-common uses it |
| directories | 6 (workspace) | XDG data dir for default DB path | Workspace standard; asana-cli reference impl |
| dateparser | 0.2 (workspace) | Human-friendly `--since` parsing | Workspace standard; asana-cli reference impl |
| anyhow | 1 (workspace) | CLI-level error handling | Workspace standard for all binaries |
| tokio | 1 (workspace) | Async runtime for block_on | Already in workspace; current-thread builder pattern |
| tftio-cli-common | 0.5.0 (workspace) | Shared is_tty() utility | Workspace shared library |

### No New Dependencies Needed
All required dependencies are already in `[workspace.dependencies]`. The crate `Cargo.toml` only
needs to add `[[bin]]` section and declare the new workspace deps.

**Additions to `crates/bsky-comment-extractor/Cargo.toml`:**
```toml
[[bin]]
name = "bce"
path = "src/main.rs"

[dependencies]
# existing
chrono = { workspace = true, features = ["std", "clock"] }
reqwest.workspace = true
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["time"] }
tracing.workspace = true
# new for CLI
anyhow.workspace = true
clap = { workspace = true }
dateparser.workspace = true
directories.workspace = true
indicatif.workspace = true
is-terminal.workspace = true
tftio-cli-common.workspace = true
```

## Architecture Patterns

### Recommended File Structure
```
crates/bsky-comment-extractor/
├── src/
│   ├── lib.rs          # existing - run_extraction(), module declarations
│   ├── main.rs         # NEW - CLI entry point (sync fn main + tokio runtime)
│   ├── cli.rs          # NEW - clap Parser struct (Cli, args)
│   ├── client.rs       # existing - fetch_all_posts (may need progress wiring)
│   ├── db.rs           # existing
│   ├── error.rs        # existing
│   └── models.rs       # existing - FetchSummary (needs new_count/existing_count fields)
```

### Pattern 1: Flat Clap CLI (from todoer)
**What:** `#[derive(Parser)]` struct with positional and optional args, no subcommands.
**When to use:** Single-command tools with no subcommand branching.
**Example:**
```rust
// Source: crates/todoer/src/cli.rs pattern + phase decisions
use clap::Parser;

/// Extract BlueSky post history to SQLite.
#[derive(Parser, Debug)]
#[command(name = "bce")]
#[command(about = "Extract a BlueSky user's post history to SQLite")]
#[command(after_help = "CREDENTIALS:\n  Set BSKY_APP_PASSWORD before running.\n  Create one at https://bsky.app/settings/app-passwords")]
pub struct Cli {
    /// BlueSky handle or DID (e.g. alice.bsky.social)
    pub handle: String,

    /// Path to SQLite database file [default: ~/.local/share/bce/bsky-posts.db]
    #[arg(long, value_name = "PATH")]
    pub db: Option<std::path::PathBuf>,

    /// Only extract posts after this date (e.g. "2025-01-01", "3 months ago")
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Suppress progress spinner
    #[arg(short, long)]
    pub quiet: bool,
}
```

### Pattern 2: Sync main with Manual Tokio Runtime (from asana-cli)
**What:** `fn main()` creates a `current_thread` tokio runtime and calls `block_on`.
**When to use:** Any async operation in a workspace binary (the workspace does NOT use `#[tokio::main]`).
**Example:**
```rust
// Source: crates/asana-cli/src/cli/mod.rs pattern
use tokio::runtime::Builder as RuntimeBuilder;

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    let runtime = match RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error: failed to start async runtime: {e}");
            return 1;
        }
    };

    match runtime.block_on(run_async(cli)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}
```

### Pattern 3: indicatif Spinner with TTY Guard (from prompter)
**What:** Create spinner only when TTY; skip entirely when piped or `--quiet`.
**When to use:** Long-running operations with unknown completion time.
**Example:**
```rust
// Source: crates/prompter/src/lib.rs (init_scaffold, lines 948-962)
use indicatif::{ProgressBar, ProgressStyle};
use is_terminal::IsTerminal;

fn make_spinner(quiet: bool, handle: &str) -> Option<ProgressBar> {
    if quiet || !std::io::stderr().is_terminal() {
        return None;
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Fetching posts for {handle}... 0 records"));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    Some(pb)
}
```
Note: The CONTEXT.md decision uses `is-terminal` to check stdout, not stderr -- use `std::io::stdout().is_terminal()` consistent with `tftio_cli_common::output::is_tty()`.

### Pattern 4: XDG Data Dir via directories (from asana-cli)
**What:** `ProjectDirs::from("com", "tftio", "bce")` gives platform-correct `data_dir()`.
**When to use:** Any binary that needs a default user-writable data path.
**Example:**
```rust
// Source: crates/asana-cli/src/config.rs pattern
use directories::ProjectDirs;

fn default_db_path() -> anyhow::Result<std::path::PathBuf> {
    let dirs = ProjectDirs::from("com", "tftio", "bce")
        .ok_or_else(|| anyhow::anyhow!("unable to resolve user data directory"))?;
    Ok(dirs.data_dir().join("bsky-posts.db"))
}
```

### Pattern 5: dateparser for --since (from asana-cli)
**What:** Parse human-friendly date strings to `DateTime<Utc>`.
**When to use:** The `--since` CLI arg before passing to `run_extraction`.
**Example:**
```rust
// Source: crates/asana-cli/src/cli/task.rs (parse_datetime_input pattern)
fn parse_since(value: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    dateparser::parse_with_timezone(value.trim(), &chrono::Utc)
        .map_err(|e| anyhow::anyhow!("failed to parse date '{value}': {e}"))
}
```

### Anti-Patterns to Avoid
- **`#[tokio::main]`**: Not the workspace pattern. Use manual `RuntimeBuilder::new_current_thread()`.
- **`eprintln!` for the spinner line**: The spinner writes to stderr by default via indicatif; the final summary line goes to stdout. Don't conflate them.
- **Checking env var for password inside lib.rs at CLI time**: The lib reads `BSKY_APP_PASSWORD` itself. The CLI's job is to verify it is set before calling `run_extraction`, so the error message is user-friendly.
- **`std::process::exit` without `ProgressBar::finish_and_clear`**: Always clear/finish the spinner before exiting, or the terminal state will be dirty.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TTY detection | `libc::isatty()` or `/dev/tty` check | `is_terminal::IsTerminal` | Handles Windows, pipes, redirects correctly |
| Animated spinner | Manual `\r` cursor tricks | `indicatif::ProgressBar::new_spinner()` | Handles TTY resize, SIGWINCH, cleanup on drop |
| Date parsing | Custom regex or chrono format strings | `dateparser::parse_with_timezone` | Handles "3 months ago", "yesterday", ISO 8601, etc. |
| XDG data dir | Hard-coded `~/.local/share/` | `directories::ProjectDirs` | Cross-platform; respects `XDG_DATA_HOME` override |
| Argument parsing | Manual `std::env::args()` | `clap` derive | Generates `--help`, error messages, completions |

**Key insight:** Every "simple" version of these utilities has platform edge cases. The workspace
already vendors correct implementations -- use them.

## Common Pitfalls

### Pitfall 1: FetchSummary Missing "new vs existing" Counts
**What goes wrong:** The summary line requires "(1,200 new, 1,250 existing)" but `FetchSummary`
only has `count: u64` (total processed) and `done: bool`. The summary line cannot be rendered.
**Why it happens:** Phase 3 was spec'd before the completion summary format was finalized.
**How to avoid:** Extend `FetchSummary` in `models.rs` to add `new_count: u64` and
`existing_count: u64`. Update `fetch_all_posts` in `client.rs` to track these -- `upsert_post`
currently does not return whether it inserted or updated. The `db::upsert_post` function uses
`INSERT OR REPLACE` semantics; to distinguish new vs existing, check `conn.changes()` after each
upsert (returns 1 for insert, 2 for replace, or query the row count before/after).
**Warning signs:** If the plan has a summary line with new/existing counts but no task to extend FetchSummary, the plan is incomplete.

### Pitfall 2: Spinner on stdout Instead of stderr
**What goes wrong:** Spinner output on stdout interferes with piped output.
**Why it happens:** `ProgressBar::new_spinner()` writes to stdout by default.
**How to avoid:** Construct spinner with `ProgressBar::with_draw_target(ProgressBar::new_spinner(), indicatif::ProgressDrawTarget::stderr())` -- or use `ProgressBar::new_spinner()` which in recent indicatif versions defaults to stderr. Verify the draw target in the indicatif 0.18 docs.
**Warning signs:** `bce handle | wc -l` corrupts output with spinner chars.

### Pitfall 3: Progress Update From Async Loop
**What goes wrong:** The `fetch_all_posts` function is async and owns the loop; the spinner lives
in `main.rs`. Updating the spinner from inside the extraction loop requires bridging the two.
**Why it happens:** The library doesn't know about CLI progress display.
**How to avoid:** Three viable patterns (Claude's discretion):
  1. `Arc<AtomicU64>` counter: pass into `fetch_all_posts`, increment in loop, poll from a spawned task or callback.
  2. Callback closure: add `progress_fn: Option<impl Fn(u64)>` parameter to `fetch_all_posts`.
  3. Post-hoc update: update spinner message on the per-page boundary using a channel (more complex).
  Option 2 (callback) is cleanest: the library stays async, the callback is sync, and no channel overhead is needed. Note `fetch_all_posts` currently has `#[allow(clippy::future_not_send)]` -- adding a callback parameter must preserve this or use `Send`-safe types.
**Warning signs:** Spinner shows "0 records" throughout the entire run.

### Pitfall 4: doc-comment Compliance with Pedantic Clippy
**What goes wrong:** `main.rs` and `cli.rs` compile but fail `cargo clippy` with `missing_docs`.
**Why it happens:** `[workspace.lints] missing_docs = "deny"` -- applies to all items in a lib crate.
  For binary crates (main.rs), `missing_docs` applies to the binary's own items unless overridden.
**How to avoid:** Add `//!` module doc comment at top of `main.rs` and `cli.rs`. Add `///` doc
  comments on the `Cli` struct and all fields. Check the existing pattern in `crates/todoer/src/cli.rs`
  (note: todoer has `lints.rust.missing_docs = "allow"` locally; bsky-comment-extractor does not -- all items need docs).
**Warning signs:** `cargo clippy -p tftio-bsky-comment-extractor` fails after adding main.rs.

### Pitfall 5: `is_terminal` Import Confusion
**What goes wrong:** Two paths exist: `is_terminal::IsTerminal` (the trait from the `is-terminal`
crate) and `tftio_cli_common::output::is_tty()` (the wrapper function).
**Why it happens:** Both are available as workspace deps.
**How to avoid:** Prefer `tftio_cli_common::output::is_tty()` for stdout checks (consistent with
the workspace pattern). Use `is_terminal::IsTerminal` directly only if checking stderr.

## Code Examples

Verified patterns from workspace source:

### Spinner Creation and TTY Guard (prompter/src/lib.rs:949-962)
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = if is_terminal() {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Initializing prompter...");
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    Some(pb)
} else {
    None
};
```

### Tokio Runtime (asana-cli/src/cli/mod.rs:446-451)
```rust
let runtime = RuntimeBuilder::new_current_thread()
    .enable_all()
    .build()
    .context("failed to initialise async runtime")?;

runtime.block_on(async move { ... })
```

### ProjectDirs Data Dir (asana-cli/src/config.rs:403-404)
```rust
ProjectDirs::from("com", "asana", "asana-cli")
    .ok_or_else(|| anyhow!("unable to resolve standard project directories"))?
```

### dateparser Usage (asana-cli/src/cli/task.rs:2406-2408)
```rust
let parsed = dateparser::parse_with_timezone(trimmed, &chrono::Utc)
    .map_err(|err| anyhow!("failed to parse date '{trimmed}': {err}"))?;
```

### Sync main -> run() -> exit code (todoer/src/main.rs:19-23)
```rust
fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| `#[tokio::main]` macro | Manual `RuntimeBuilder::new_current_thread()` | Workspace pattern; more explicit runtime control |
| `dirs` crate for home dir | `directories::ProjectDirs` for XDG-aware paths | Full XDG compliance, not just home dir expansion |
| `atty` crate | `is-terminal` crate | `atty` was yanked; `is-terminal` is the replacement |

## Open Questions

1. **How to track new vs existing in upsert_post**
   - What we know: `rusqlite::Connection::changes()` returns number of rows modified by last statement. For an `INSERT OR REPLACE`, it returns 1 if the row was new and 2 if it replaced (because DELETE + INSERT = 2 changes).
   - What's unclear: Whether this is reliable across all rusqlite 0.38 versions.
   - Recommendation: Check `conn.changes()` after each `upsert_post` call; if `== 1` it was new, if `== 2` it was a replace. Return new/existing counts in `FetchSummary`. Alternatively, query `SELECT COUNT(*) FROM posts WHERE uri = ?` before upsert (cleaner but 2x queries per record). The `changes()` approach is faster.

2. **Spinner draw target in indicatif 0.18**
   - What we know: `ProgressBar::new_spinner()` default draw target varies by crate version.
   - What's unclear: Whether 0.18 defaults to stderr.
   - Recommendation: Explicitly set `ProgressDrawTarget::stderr()` to be safe.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none -- workspace uses `just test` |
| Quick run command | `cargo test -p tftio-bsky-comment-extractor` |
| Full suite command | `just ci` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-01 | Positional handle arg parsed and passed to run_extraction | unit | `cargo test -p tftio-bsky-comment-extractor test_cli_parse` | No -- Wave 0 |
| CLI-02 | --db flag overrides default path | unit | `cargo test -p tftio-bsky-comment-extractor test_db_path_override` | No -- Wave 0 |
| CLI-03 | Spinner suppressed when quiet=true or not TTY | unit | `cargo test -p tftio-bsky-comment-extractor test_spinner_quiet` | No -- Wave 0 |
| CLI-04 | Binary compiles and passes clippy/fmt | smoke | `just ci` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p tftio-bsky-comment-extractor`
- **Per wave merge:** `just ci`
- **Phase gate:** `just ci` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/bsky-comment-extractor/src/main.rs` -- binary entry point (currently missing entirely)
- [ ] `crates/bsky-comment-extractor/src/cli.rs` -- clap Cli struct
- [ ] Unit tests in `src/cli.rs` or `src/main.rs` for argument parsing and path resolution

## Sources

### Primary (HIGH confidence)
- `crates/bsky-comment-extractor/src/lib.rs` -- `run_extraction` signature, FetchSummary return type
- `crates/bsky-comment-extractor/src/models.rs` -- FetchSummary fields (count, done only)
- `crates/bsky-comment-extractor/src/client.rs` -- fetch_all_posts loop structure
- `crates/bsky-comment-extractor/src/error.rs` -- ExtractorError variants
- `crates/todoer/src/cli.rs` -- clap Parser flat CLI pattern
- `crates/todoer/src/main.rs` -- sync main + run() -> exit code pattern
- `crates/prompter/src/lib.rs` -- indicatif spinner with TTY guard (lines 948-962)
- `crates/asana-cli/src/cli/mod.rs` -- tokio RuntimeBuilder::new_current_thread pattern
- `crates/asana-cli/src/config.rs` -- ProjectDirs::from() for XDG paths
- `crates/asana-cli/src/cli/task.rs` -- dateparser::parse_with_timezone usage
- `crates/cli-common/src/output.rs` -- is_tty() utility (is_terminal wrapper)
- `Cargo.toml` (workspace root) -- all dependency versions confirmed

### Secondary (MEDIUM confidence)
- indicatif 0.18 draw target behavior -- not directly verified from docs, recommendation to set explicitly

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all deps confirmed in workspace Cargo.toml, all versions direct-read
- Architecture: HIGH -- patterns directly read from workspace reference implementations
- Pitfalls: HIGH for doc/lint issues (confirmed from STATE.md history); MEDIUM for indicatif draw target

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (stable dependencies, workspace patterns unlikely to change)
