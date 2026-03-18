# Architecture

**Analysis Date:** 2026-03-23

## Pattern Overview

**Overall:** Modular Rust CLI workspace using layered architecture pattern.

This is a **Cargo workspace monorepo** containing 7 independently-versioned Rust crates. Each binary crate follows a consistent layered design: CLI parsing → command execution → API/data layer → models. Shared functionality is extracted into `tftio-cli-common` library crate.

**Key Characteristics:**
- Centralized dependency management in root `Cargo.toml` via `[workspace.dependencies]`
- Separation of concerns: CLI layer, command execution, API abstraction, models
- Async/await throughout (tokio runtime)
- Strong error handling via `anyhow::Result<T>` type alias
- Configuration stored in user home directory (typically `~/.config/` or `~/.local/share/`)
- Tracing-based observability across all crates

## Layers

**CLI Layer:**
- Purpose: Parse command-line arguments, route to subcommands, handle help/version/completions
- Location: `src/cli.rs` or `src/cli/mod.rs`
- Contains: `clap` Parser/Subcommand enums, command dispatch logic
- Depends on: Command implementations, models
- Used by: `main.rs` entry point

**Command Execution Layer:**
- Purpose: Execute business logic for each subcommand
- Location: `src/commands/` directory (per-command module files)
- Contains: Functions that orchestrate API calls, format output, handle user interaction
- Depends on: API layer, models, output formatting
- Used by: CLI layer dispatch

**API/Service Layer:**
- Purpose: Encapsulate HTTP client, authentication, API calls, pagination, retry logic
- Location: `src/api/` (external service integrations) or specific handler modules
- Contains: HTTP client initialization, request construction, response parsing, error translation
- Depends on: Models, `reqwest`/`tokio`, configuration
- Used by: Command execution layer

**Models & Types Layer:**
- Purpose: Define data structures for serialization/deserialization and business logic
- Location: `src/models/` or `src/models.rs`
- Contains: `serde` structs for JSON/TOML, enums for state/types
- Depends on: `serde`, `serde_json`
- Used by: All layers for type-safe data passing

**Configuration & State Layer:**
- Purpose: Load and manage runtime configuration, database connections, project state
- Location: `src/config.rs`, `src/db.rs`, `src/project.rs`
- Contains: Config file parsing, database initialization, project metadata
- Depends on: Models, `rusqlite`/TOML parsing
- Used by: CLI and command layers

**Output & Formatting Layer:**
- Purpose: Format results for display (tables, JSON, markdown)
- Location: `src/output/` or `src/output.rs`
- Contains: Display traits, formatting helpers, template rendering
- Depends on: Models, `tabled`, `colored`
- Used by: Command execution layer

**Shared Library:**
- Purpose: Provide common CLI functionality to all binary crates
- Location: `crates/cli-common/src/`
- Contains: Completions generation, doctor (health check) framework, license display
- Depends on: `clap`, `colored`, filesystem access
- Used by: All binary crates that need completions/doctor/license commands

## Data Flow

**Typical CLI execution flow:**

1. `main.rs` → calls `init_tracing()` to set up logging, then calls `cli::run()`
2. `cli::run()` → parses args with clap, dispatches to command handler
3. Command handler → loads config, initializes API client/database, executes business logic
4. Command → makes API calls or database queries, collects results
5. Output layer → formats results (table, JSON, colored text)
6. Result → printed to stdout, exit code returned

**Example: asana-cli task list flow:**

```
main.rs
  → cli::run()
    → Commands::Task { command } dispatch
      → task::run_list()
        → ApiClient::new(config)
        → list_tasks(filters, pagination)
        → TaskListParams → reqwest GET → json decode → Vec<Task>
        → output::format_tasks(tasks, format)
        → println!("{formatted_output}")
```

**State Management:**

- **Configuration:** Loaded once from `~/.config/<tool>/config.toml` or env vars, passed through command call stack
- **API Client:** Constructed with auth token, reused for multiple API calls within a command
- **Database State:** Persistent via SQLite, opened at command start, committed at end
- **Output State:** Streaming (stdout) - no buffering except for table formatting

## Key Abstractions

**API Client (`asana_cli::api::ApiClient`):**
- Purpose: Abstract HTTP layer, handle auth, pagination, caching, retries
- Examples: `crates/asana-cli/src/api/client.rs`
- Pattern: Builder pattern for client construction, generic request/response methods, rate-limit aware retry logic

**Command Struct (trait-like):**
- Purpose: Represent a subcommand with associated options
- Examples: `crates/asana-cli/src/cli/task.rs` defines `TaskCommand` with variants
- Pattern: `clap::Subcommand` enum, methods for execution

**Models with Builders:**
- Purpose: Type-safe data construction with fluent interface
- Examples: `crates/asana-cli/src/models/task.rs` defines `TaskCreateBuilder`, `TaskUpdateBuilder`
- Pattern: Builder pattern allows optional fields while maintaining immutable model

**Configuration Provider:**
- Purpose: Load tool-specific config from home directory
- Examples: `crates/todoer/src/config.rs`, `crates/asana-cli/src/config.rs`
- Pattern: Single function to load from standard locations, fallback to defaults

**Database Abstraction:**
- Purpose: Encapsulate SQL operations behind domain-specific functions
- Examples: `crates/todoer/src/db.rs` provides `open_db()`, `init_db()`
- Pattern: rusqlite Connection wrapper, single module with all SQL

## Entry Points

**Binary entry point:**
- Location: `src/main.rs` (present in all binary crates)
- Triggers: Invoked by shell when user runs binary
- Responsibilities:
  1. Initialize tracing/logging
  2. Call library `lib.rs` public API (typically `cli::run()`)
  3. Handle Result and exit with appropriate code

**CLI dispatch:**
- Location: `src/cli/mod.rs` or `src/cli.rs`, function `run() -> Result<i32>`
- Triggers: Called from `main.rs`
- Responsibilities:
  1. Parse command-line arguments
  2. Load configuration and initialize services
  3. Dispatch to appropriate command handler
  4. Return exit code

**Command handlers:**
- Location: `src/commands/<name>.rs`
- Triggers: Dispatched from CLI layer based on subcommand
- Responsibilities:
  1. Extract options from command struct
  2. Call API/database layer
  3. Format and display results
  4. Return error if operation failed

## Error Handling

**Strategy:** Contextual error propagation with anyhow.

**Patterns:**

1. **Result type alias:** All functions return `Result<T>` = `anyhow::Result<T>`
   - File: `crates/asana-cli/src/error.rs`
   - Provides automatic error context with `.context("operation failed")`

2. **Error translation:** Domain-specific errors converted to anyhow
   - Example: `rusqlite::Error` → `anyhow::Result` via `map_err()`
   - Example: `reqwest::Error` → `ApiError` → display via tracing

3. **Error logging:** Errors logged with tracing before returning
   - File: `crates/asana-cli/src/main.rs` shows pattern: `tracing::error!(error = %err, "command execution failed")`
   - Used for observability while propagating to user

4. **User-facing errors:** Printed to stderr and exit code set to 1
   - Example: `crates/asana-cli/src/main.rs` catches `Err(err)` and prints `{err:?}` with exit(1)

## Cross-Cutting Concerns

**Logging:**
- Framework: `tracing` with `tracing-subscriber` for filtering
- Usage: `tracing::debug!()`, `tracing::info!()`, `tracing::error!()` throughout
- Configuration: `RUST_LOG` env var, defaults to "info" level
- Initialization: `init_tracing()` called in `main.rs` before any work

**Validation:**
- File: `crates/asana-cli/src/models/` - builder pattern validates during construction
- Pattern: Custom error types (e.g., `StoryValidationError`, `TagValidationError`) ensure invalid states unreachable
- Timing: Validation at API model construction time, before serialization

**Authentication:**
- Framework: Token-based (Asana Bearer token, Todoist API key)
- Storage: Config file `~/.config/<tool>/config.toml` or environment variables
- Pattern: `TokenProvider` trait in `asana_cli::api::auth`, `StaticTokenProvider` implementation
- Secret handling: `secrecy` crate prevents accidental logging of sensitive values

---

*Architecture analysis: 2026-03-23*
