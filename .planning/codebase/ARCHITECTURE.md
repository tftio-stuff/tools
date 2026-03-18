# Architecture

**Analysis Date:** 2026-03-17

## Pattern Overview

**Overall:** Cargo workspace monorepo of independent CLI tools sharing a common library

**Key Characteristics:**
- Each tool is an independently versioned, independently deployable binary crate
- One shared library crate (`cli-common`) provides cross-cutting CLI utilities
- Tools that expose library APIs provide both `lib.rs` and `main.rs`
- State persistence uses SQLite (bundled) via `rusqlite` for stateful tools
- No shared runtime or daemon -- every binary is standalone

## Layers

**Shared Library (`cli-common`):**
- Purpose: Cross-cutting CLI utilities reused by all tools
- Location: `crates/cli-common/src/`
- Contains: Shell completions, doctor/health-check framework, license display, terminal output helpers, update checks
- Depends on: `clap`, `clap_complete`, `colored`, `is-terminal`
- Used by: `asana-cli`, `unvenv`, and any crate needing standard CLI chrome

**CLI Layer (`cli.rs` or `cli/mod.rs`):**
- Purpose: Argument parsing and command dispatch
- Location: `crates/<name>/src/cli.rs` or `crates/<name>/src/cli/mod.rs`
- Contains: `clap` `Parser`/`Subcommand` structs, top-level `run()` dispatcher
- Depends on: config, commands
- Used by: `main.rs` only

**Command Layer (`commands/`):**
- Purpose: Business logic for each subcommand, one module per command group
- Location: `crates/<name>/src/commands/<cmd>.rs`
- Contains: `run_*` functions that accept parsed inputs and return typed results
- Depends on: models, db (for stateful tools), config
- Used by: CLI layer / main.rs dispatch

**Data Layer (`db.rs`):**
- Purpose: SQLite CRUD operations, schema initialization
- Location: `crates/<name>/src/db.rs`
- Contains: `open_db`, `init_db`, and typed CRUD functions per model
- Depends on: `rusqlite`, models
- Used by: commands, main.rs (for connection management)

**Models Layer (`models.rs` or `models/`):**
- Purpose: Typed data representations with serde serialization
- Location: `crates/<name>/src/models.rs` or `crates/asana-cli/src/models/`
- Contains: Domain structs/enums, `Display`/`FromStr` impls for DB round-trips
- Depends on: `serde`
- Used by: db, commands, output

**Config Layer (`config.rs`):**
- Purpose: Load tool configuration from disk, resolve paths
- Location: `crates/<name>/src/config.rs`
- Contains: Config struct, `load_config()`, path resolution helpers
- Depends on: `directories`, `toml`/`serde`, filesystem
- Used by: main.rs, commands

**Output Layer (`output.rs`):**
- Purpose: Format results for human (plain text) or machine (JSON) consumption
- Location: `crates/<name>/src/output.rs`
- Contains: Render functions, `ok_response`/`err_response` JSON envelope helpers
- Depends on: `serde_json`, `tabled`, `colored`
- Used by: main.rs dispatch (after command execution)

**API Layer (`api/`) -- asana-cli only:**
- Purpose: Async HTTP client abstractions over the Asana REST API
- Location: `crates/asana-cli/src/api/`
- Contains: `client.rs` (central `ApiClient`), per-resource modules (tasks, projects, etc.), auth, pagination, error types
- Depends on: `reqwest`, `tokio`, `tracing`, `secrecy`
- Used by: CLI command handlers via `build_api_client()`

## Data Flow

**Standard synchronous tool (todoer, silent-critic):**

1. `main()` calls `Cli::parse()` (clap) and dispatches to `run(cli)`
2. `run()` calls `load_config()` to get config/paths
3. `run()` opens SQLite DB via `open_db(&path)` (stateful tools)
4. `run()` delegates to a `commands::<module>::run_<action>(&conn, ...)` function
5. Command function reads/writes models through `db::*` CRUD functions
6. `run()` formats result: if `--json` flag, uses `ok_response()`/`err_response()` JSON envelope; otherwise plain text
7. `main()` calls `std::process::exit(code)`

**Asana CLI (async tool):**

1. `main()` initializes tracing, calls `cli::run()`
2. `cli::run()` parses args, loads config, builds `ApiClient` via `build_api_client()`
3. Command handler creates a `tokio` runtime (`RuntimeBuilder::new_current_thread()`) and calls `block_on(async { client.method().await })`
4. `ApiClient` streams paginated responses via `async_stream`; handles caching, rate limits, retries
5. Output rendered via `output/` modules

**Gator (process exec tool):**

1. `main()` calls `Cli::parse()`, validates, calls `lib::run(&cli)`
2. `run()` resolves workdir (explicit > git root > cwd), loads `.safehouse` config and named policies
3. Optionally fetches silent-critic session sandbox via `session::fetch_session_sandbox()`
4. Assembles macOS `sandbox-exec` policy string via `sandbox::assemble_policy()`
5. Composes prompter prompt via `prompt::compose_prompt()`
6. Writes policy to tempfile, calls `agent::exec_command()` which replaces the process via `execv`

**State Management:**
- `asana-cli`: in-memory + on-disk JSON cache for API responses; no persistent app state
- `todoer`: SQLite at path from `.todoer.toml` config; per-project databases
- `silent-critic`: SQLite at `~/.local/share/silent-critic/<repo-hash>/db.sqlite`; project identified by SHA-2 hash of repo root path
- `prompter`: stateless; reads TOML profiles from disk at invocation time
- `unvenv`, `gator`: stateless

## Key Abstractions

**`DoctorChecks` trait (`cli-common`):**
- Purpose: Standard health-check interface implemented by each tool
- Examples: `crates/cli-common/src/doctor.rs`, inline impl in `crates/asana-cli/src/cli/mod.rs`, `crates/unvenv/src/main.rs`
- Pattern: Implement `repo_info()`, `current_version()`, `tool_checks()` returning `Vec<DoctorCheck>`; call `run_doctor(&self)`

**`ok_response` / `err_response` JSON envelope:**
- Purpose: Machine-readable output for LLM agent consumption
- Examples: `crates/todoer/src/output.rs`, `crates/silent-critic/src/output.rs`
- Pattern: `{"ok": true, "command": "...", "data": {...}}` / `{"ok": false, "error": "...", ...}`

**`ApiClient` builder (`asana-cli`):**
- Purpose: Configurable async HTTP client with auth, caching, rate-limiting
- Examples: `crates/asana-cli/src/api/client.rs`
- Pattern: `ApiClient::builder(token).base_url(...).cache_dir(...).build()`

**Session state machine (`silent-critic`):**
- Purpose: Enforce valid lifecycle transitions for agentic supervision sessions
- Examples: `crates/silent-critic/src/models.rs` (`SessionStatus`), `crates/silent-critic/src/db.rs` (`transition_session`)
- Pattern: `SessionStatus::can_transition_to()` checked before every `UPDATE`; states: `discovering -> composing -> ready -> executing -> awaiting_adjudication -> adjudicated`

**Sandbox policy assembly (`gator`):**
- Purpose: Compose macOS `sandbox-exec` SBPL policy from workdir, worktrees, and extra grants
- Examples: `crates/gator/src/sandbox.rs`
- Pattern: `assemble_policy(&workdir, &wt_info, &extras, &denies)` returns policy string written to tempfile before exec

## Entry Points

**`crates/asana-cli/src/main.rs`:**
- Triggers: `asana-cli <subcommand>`
- Responsibilities: Initialize tracing, call `cli::run()`, exit with returned code

**`crates/todoer/src/main.rs`:**
- Triggers: `todoer <subcommand> [--json]`
- Responsibilities: Parse CLI, load config, open DB, dispatch to command modules, format output

**`crates/silent-critic/src/main.rs`:**
- Triggers: `silent-critic <subcommand> [--json]`
- Responsibilities: Parse CLI, dispatch to `commands::*`, open project DB per command, format output

**`crates/gator/src/main.rs`:**
- Triggers: `gator <agent> [profiles...] [flags]`
- Responsibilities: Parse and validate CLI, call `lib::run(&cli)` which execs agent process (never returns on success)

**`crates/unvenv/src/main.rs`:**
- Triggers: `unvenv [scan|doctor|completions|...]`
- Responsibilities: Single-file tool; scan for unignored `pyvenv.cfg` files via `walkdir` + `git2`

**`crates/prompter/src/main.rs`:**
- Triggers: `prompter <subcommand>`
- Responsibilities: Load TOML profiles, resolve recursive dependencies, concatenate markdown snippets

## Error Handling

**Strategy:** `anyhow` for application-level errors with context chaining; typed `ApiError` enum for HTTP errors in `asana-cli`

**Patterns:**
- All fallible functions return `anyhow::Result<T>` (aliased as `type Result<T> = anyhow::Result<T>` in `crates/asana-cli/src/error.rs`)
- `.context("description")` and `.with_context(|| ...)` used to add location information
- `main()` functions convert errors to exit codes: success = 0, error = 1, policy violation = 2 (unvenv)
- JSON output mode wraps errors in `err_response()` envelope instead of `eprintln!`
- `silent-critic` enforces state machine validity at the DB layer (`transition_session` bails on invalid transitions)

## Cross-Cutting Concerns

**Logging:** `tracing` + `tracing-subscriber` with `EnvFilter`; initialized in `main()` for `asana-cli`. Other tools use `eprintln!` for errors. Level controlled via `RUST_LOG` env var.

**Validation:** CLI validation via `clap` derive macros (types, required args). Business-rule validation in command modules. `gator` has post-parse `cli.validate()` for mutual-exclusion checks (`--session` vs `--workdir`/`--policy`).

**Authentication:** `asana-cli` uses `secrecy::SecretString` for PAT; stored in config file or `ASANA_TOKEN` env var. `silent-critic` uses opaque session tokens (`SILENT_CRITIC_TOKEN` env var) to distinguish worker vs operator roles.

---

*Architecture analysis: 2026-03-17*
