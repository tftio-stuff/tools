# Architecture

**Analysis Date:** 2026-03-23

## Pattern Overview

**Overall:** Cargo workspace monorepo of independent Rust CLI crates with one shared support library.

**Key Characteristics:**
- The workspace root in `Cargo.toml` owns dependency versions, lint policy, edition, and Rust version for all member crates under `crates/`.
- Most tools use a thin binary entry point in `crates/*/src/main.rs` that parses CLI input and delegates to library modules in `crates/*/src/lib.rs` or sibling modules.
- Shared CLI concerns are centralized in `crates/cli-common/src/`, while each product crate keeps its own domain models, command handlers, storage code, and output formatters.

## Layers

**Workspace Layer:**
- Purpose: Define crate boundaries, shared dependencies, lint settings, release metadata, and developer workflows.
- Location: `Cargo.toml`, `justfile`, `deny.toml`, `rustfmt.toml`, `release-please-config.json`, `.github/workflows/`.
- Contains: Workspace members, `[workspace.dependencies]`, task recipes, CI/release configuration.
- Depends on: Cargo workspace features and GitHub Actions.
- Used by: Every crate under `crates/`.

**Shared CLI Support Layer:**
- Purpose: Reusable utilities for completions, doctor checks, license output, terminal detection, and self-update flows.
- Location: `crates/cli-common/src/lib.rs`, `crates/cli-common/src/completions.rs`, `crates/cli-common/src/doctor.rs`, `crates/cli-common/src/output.rs`, `crates/cli-common/src/update.rs`.
- Contains: Re-exported helpers such as `generate_completions`, `run_doctor`, and `display_license`.
- Depends on: `clap`, `clap_complete`, `colored`, `is-terminal`.
- Used by: `crates/prompter/`, `crates/unvenv/`, `crates/asana-cli/`, `crates/gator/`, and `crates/bsky-comment-extractor/`.

**Per-Tool CLI Layer:**
- Purpose: Define clap commands and argument structures, then dispatch to tool-specific operations.
- Location: `crates/gator/src/cli.rs`, `crates/bsky-comment-extractor/src/cli.rs`, `crates/todoer/src/cli.rs`, `crates/silent-critic/src/cli.rs`, `crates/asana-cli/src/cli/`, `crates/prompter/src/lib.rs`, `crates/unvenv/src/main.rs`.
- Contains: `Parser`/`Subcommand` structs and enums, flag validation, help text, and command routing.
- Depends on: clap derive plus the crate’s domain modules.
- Used by: Binary entry points in each crate’s `src/main.rs`.

**Domain/Service Layer:**
- Purpose: Hold tool behavior behind the CLI surface.
- Location: `crates/bsky-comment-extractor/src/client.rs` and `src/db.rs`, `crates/gator/src/config.rs` and `src/sandbox.rs`, `crates/todoer/src/commands/` and `src/repo.rs`, `crates/silent-critic/src/commands/` and `src/discovery.rs`, `crates/asana-cli/src/api/`.
- Contains: API clients, repository functions, session/state logic, sandbox policy builders, and command handlers.
- Depends on: Models, config modules, external APIs or `rusqlite`.
- Used by: CLI dispatch code.

**Persistence/Model Layer:**
- Purpose: Represent structured data and persistence schemas.
- Location: `crates/bsky-comment-extractor/src/models.rs`, `crates/todoer/src/models.rs`, `crates/todoer/src/db.rs`, `crates/silent-critic/src/models.rs`, `crates/silent-critic/src/db.rs`, `crates/asana-cli/src/models/`.
- Contains: serde DTOs, SQLite schema setup, enum state machines, row conversion logic.
- Depends on: `serde`, `rusqlite`, `chrono`, `uuid`.
- Used by: Service and output layers.

## Data Flow

**Generic CLI Flow:**
1. `crates/<tool>/src/main.rs` parses arguments or calls a `run()` function.
2. CLI definitions in `crates/<tool>/src/cli.rs` or `crates/<tool>/src/cli/` map subcommands to domain actions.
3. Service modules perform I/O against local config, local SQLite, Git metadata, or remote APIs.
4. Output modules or main functions emit human-readable text or JSON and select a process exit code.

**`bce` Extraction Flow (`crates/bsky-comment-extractor/`):**
1. `crates/bsky-comment-extractor/src/main.rs` matches `Fetch` or `Query`.
2. `execute_fetch()` builds a single-threaded Tokio runtime, then calls `bsky_comment_extractor::run_extraction()` from `crates/bsky-comment-extractor/src/lib.rs`.
3. `crates/bsky-comment-extractor/src/client.rs` authenticates or resolves a handle, paginates `com.atproto.repo.listRecords`, and sends records to `crates/bsky-comment-extractor/src/db.rs`.
4. `crates/bsky-comment-extractor/src/db.rs` persists posts and extraction cursors in SQLite, then `src/main.rs` prints summaries or JSONL query results.

**`gator` Harness Flow (`crates/gator/`):**
1. `crates/gator/src/main.rs` parses flags and validates session-specific conflicts.
2. `crates/gator/src/lib.rs` resolves a workdir from CLI, Git, or `silent-critic` session metadata.
3. `crates/gator/src/config.rs`, `src/worktree.rs`, and `src/session.rs` assemble directory grants and deny lists.
4. `crates/gator/src/sandbox.rs` generates SBPL policy text and `crates/gator/src/agent.rs` execs `sandbox-exec` with agent-specific prompt injection.

**`silent-critic` Session Flow (`crates/silent-critic/`):**
1. `crates/silent-critic/src/main.rs` loads config and dispatches subcommands.
2. `crates/silent-critic/src/commands/session.rs` advances a session through `Discovering -> Composing -> Ready -> Executing -> AwaitingAdjudication -> Adjudicated`.
3. `crates/silent-critic/src/discovery.rs` records repository context and `crates/silent-critic/src/db.rs` persists projects, contracts, evidence, and decisions.
4. Output is serialized through `crates/silent-critic/src/output.rs` as text or structured JSON.

## State Management

**State Management:**
- Workspace-wide state is mostly static configuration in `Cargo.toml`, `justfile`, `.github/workflows/`, and release metadata files at the repository root.
- Durable runtime state lives in SQLite for `crates/todoer/src/db.rs`, `crates/silent-critic/src/db.rs`, and `crates/bsky-comment-extractor/src/db.rs`.
- Ephemeral execution state is kept in process memory for `crates/gator/src/lib.rs` and `crates/asana-cli/src/api/client.rs`.
- Planning state for current product work is tracked outside the crates in `.planning/STATE.md` and `.planning/milestones/`.

## Key Abstractions

**Workspace Member as Product Boundary:**
- Purpose: Each directory in `crates/` is the deployable or reusable boundary.
- Examples: `crates/cli-common/`, `crates/prompter/`, `crates/unvenv/`, `crates/asana-cli/`, `crates/todoer/`, `crates/silent-critic/`, `crates/gator/`, `crates/bsky-comment-extractor/`.
- Pattern: One Cargo package per tool, with optional `lib.rs` for reusable internal APIs.

**Command Enum Dispatch:**
- Purpose: Model the CLI surface as typed subcommands.
- Examples: `crates/bsky-comment-extractor/src/cli.rs`, `crates/todoer/src/cli.rs`, `crates/silent-critic/src/cli.rs`, `crates/asana-cli/src/cli/mod.rs`.
- Pattern: Clap `Parser` + `Subcommand` enums feeding `match` dispatch in `src/main.rs` or `cli::run()`.

**Repository/Storage Abstraction:**
- Purpose: Keep SQL and persistence isolated from CLI code.
- Examples: `crates/todoer/src/repo.rs`, `crates/todoer/src/db.rs`, `crates/silent-critic/src/db.rs`, `crates/bsky-comment-extractor/src/db.rs`.
- Pattern: Open/init database separately, then use focused functions for row reads/writes.

**Remote API Client Abstraction:**
- Purpose: Encapsulate auth, retries, pagination, and serialization.
- Examples: `crates/asana-cli/src/api/client.rs`, `crates/bsky-comment-extractor/src/client.rs`.
- Pattern: Stateful client structs with helper methods rather than inline HTTP calls from command handlers.

## Entry Points

**Workspace Build/Test Entrypoints:**
- Location: `justfile`
- Triggers: Manual local commands such as `just build`, `just test`, `just ci`.
- Responsibilities: Wrap workspace-wide Cargo build, lint, test, audit, and release-style checks.

**Binary Entrypoints:**
- `crates/asana-cli/src/main.rs`: initializes tracing, then delegates to `asana_cli::cli::run()`.
- `crates/bsky-comment-extractor/src/main.rs`: parses `Cli`, then routes to `fetch` or `query`.
- `crates/gator/src/main.rs`: validates CLI invariants, then runs sandbox harness logic.
- `crates/prompter/src/main.rs`: resolves `AppMode` and dispatches render/list/init/doctor flows.
- `crates/silent-critic/src/main.rs`: loads config and dispatches project, criterion, session, contract, decide, and log commands.
- `crates/todoer/src/main.rs`: routes CRUD-style task commands and project discovery.
- `crates/unvenv/src/main.rs`: handles default scan mode plus doctor/completions/update helpers.

## Error Handling

**Strategy:** Parse early, centralize domain errors, and convert failures to text or JSON at the final CLI boundary.

**Patterns:**
- Typed domain errors live in files such as `crates/bsky-comment-extractor/src/error.rs` and `crates/asana-cli/src/api/error.rs`.
- CLI binaries usually convert errors into exit code `1` or a domain-specific non-zero status, for example `crates/unvenv/src/main.rs` returns `2` when unignored virtual environments are found.
- JSON error envelopes are used when machine-readable output is part of the contract, for example `crates/silent-critic/src/output.rs`, `crates/todoer/src/output.rs`, and query error handling in `crates/bsky-comment-extractor/src/main.rs`.

## Cross-Cutting Concerns

**Logging:** `crates/asana-cli/src/lib.rs` installs `tracing_subscriber`; `crates/bsky-comment-extractor/src/client.rs` uses `tracing::warn!`; most other crates rely on explicit stdout/stderr output in `src/main.rs`.

**Validation:** CLI validation sits close to clap structs, such as `Cli::validate()` in `crates/gator/src/cli.rs` and field-specific argument constraints in `crates/bsky-comment-extractor/src/cli.rs`.

**Authentication:** External-service auth is localized inside client/config modules: Asana token handling in `crates/asana-cli/src/api/auth.rs` and config logic, BlueSky app-password auth in `crates/bsky-comment-extractor/src/client.rs`, and worker/operator token checks in `crates/silent-critic/src/commands/session.rs`.

**Shared Code:** New reusable CLI helpers belong in `crates/cli-common/src/` first; tool-specific code stays inside its crate unless another crate imports it through the workspace dependency graph.

**CLI Design Patterns:**
- Use `src/main.rs` as a thin boundary and keep real work in library modules.
- Prefer nested clap enums for command families, as in `crates/silent-critic/src/cli.rs` and `crates/asana-cli/src/cli/mod.rs`.
- Keep output formatting separate from data retrieval when a tool supports multiple formats, as in `crates/asana-cli/src/output/` and `crates/todoer/src/output.rs`.
- Use a library crate plus binary pair when the crate exposes reusable functions internally, as in `crates/prompter/`, `crates/asana-cli/`, `crates/gator/`, `crates/todoer/`, `crates/silent-critic/`, and `crates/bsky-comment-extractor/`.

---

*Architecture analysis: 2026-03-23*
