# Coding Conventions

**Analysis Date:** 2026-03-23

## Naming Patterns

**Files:**
- Use `snake_case` for Rust module files such as `crates/gator/src/worktree.rs`, `crates/todoer/src/output.rs`, and `crates/bsky-comment-extractor/src/error.rs`.
- Use `kebab-case` for crate directories and binary names such as `crates/asana-cli`, `crates/silent-critic`, and the `bce` binary declared in `crates/bsky-comment-extractor/Cargo.toml`.
- Group command handlers under `src/commands/` when the CLI is multi-command, as in `crates/todoer/src/commands/` and `crates/silent-critic/src/commands/`.

**Functions:**
- Use `snake_case` for functions and helpers, including CLI helpers like `print_top_level_help` in `crates/bsky-comment-extractor/src/main.rs`, `run_validate_stdout` in `crates/prompter/src/lib.rs`, and `resolve_workdir` in `crates/gator/src/config.rs`.
- Keep `main` thin and delegate to `run` or `dispatch`, as in `crates/todoer/src/main.rs`, `crates/silent-critic/src/main.rs`, and `crates/gator/src/main.rs`.

**Variables:**
- Use descriptive local names over abbreviations in application code (`workdir`, `ungated_siblings`, `config_home`, `data_home` in `crates/gator/src/lib.rs` and `crates/asana-cli/tests/cli.rs`).
- Use short temporaries only in narrow scopes (`cfg`, `pb`, `out`, `err`) as seen in `crates/asana-cli/src/config.rs`, `crates/bsky-comment-extractor/src/main.rs`, and `crates/prompter/src/main.rs`.

**Types:**
- Use `PascalCase` for public structs and enums such as `Cli`, `Command`, `FetchArgs`, `ResolvedProject`, and `SessionCommand` in `crates/*/src/cli.rs`.
- Use ALL_CAPS for env-var constants where present, such as `ENV_CONFIG_HOME` and `ENV_TOKEN` in `crates/asana-cli/src/config.rs`.

## Code Style

**Formatting:**
- Format with `rustfmt` using `rustfmt.toml`: `max_width = 100`, spaces not tabs, `tab_spaces = 4`, `force_explicit_abi = true`, `use_field_init_shorthand = true`, and `use_try_shorthand = true`.
- `cargo +nightly fmt --all` is the canonical formatter entry point in `justfile` and in `.github/workflows/ci.yml`.

**Linting:**
- The workspace root `Cargo.toml` sets strict defaults: `unsafe_code = "warn"`, `missing_docs = "deny"`, `clippy::all = "deny"`, and `clippy::pedantic = "deny"`.
- Prefer inheriting workspace lints via `[lints] workspace = true`, as in `crates/gator/Cargo.toml`, `crates/prompter/Cargo.toml`, and `crates/bsky-comment-extractor/Cargo.toml`.
- Local lint escapes are explicit and narrow. Examples: `#[allow(clippy::struct_excessive_bools)]` in `crates/gator/src/cli.rs`, `#[allow(clippy::future_not_send)]` in `crates/bsky-comment-extractor/src/lib.rs`, and per-crate `missing_docs = "allow"` in `crates/todoer/Cargo.toml` and `crates/silent-critic/Cargo.toml`.
- When unsafe is required, include a nearby safety explanation, as in `crates/gator/src/lib.rs` and `crates/todoer/tests/config_resolution.rs`.

## Import Organization

**Order:**
1. Standard library imports first, e.g. `use std::path::{Path, PathBuf};` in `crates/bsky-comment-extractor/src/main.rs`.
2. Third-party crates second, e.g. `use anyhow::{Context, Result, bail};` and `use clap::{CommandFactory, Parser};` in `crates/bsky-comment-extractor/src/main.rs`.
3. Local crate imports last, e.g. `use bsky_comment_extractor::db::{count_posts, open_existing_db, query_posts};` in `crates/bsky-comment-extractor/src/main.rs`.

**Path Aliases:**
- Use explicit crate paths instead of relative aliases. Examples: `use todoer::commands::new::run_new` in `crates/todoer/tests/commands_new.rs` and `use silent_critic::commands::{contract, criterion, decide, log, project, session};` in `crates/silent-critic/src/main.rs`.
- Re-export shared surface area from `src/lib.rs` when a crate is meant to be consumed by others, as in `crates/cli-common/src/lib.rs`.

## Error Handling

**Patterns:**
- Use typed error enums when the crate has a meaningful domain model, e.g. `ExtractorError` in `crates/bsky-comment-extractor/src/error.rs`.
- Use `anyhow::Result` at binary boundaries or orchestration layers, e.g. `crates/asana-cli/src/error.rs`, `crates/silent-critic/src/main.rs`, and `crates/bsky-comment-extractor/src/main.rs`.
- Add context at I/O and parsing boundaries, as in `with_context` calls in `crates/asana-cli/src/config.rs`.
- Convert failures into exit codes in `src/main.rs` instead of panicking. Examples: `crates/gator/src/main.rs`, `crates/todoer/src/main.rs`, and `crates/prompter/src/main.rs`.
- Support structured JSON errors for machine-facing CLIs. Examples: `ok_response` and `err_response` in `crates/todoer/src/output.rs` and `crates/silent-critic/src/output.rs`, plus JSON error lines in `crates/bsky-comment-extractor/src/main.rs`.

## Logging

**Framework:** mixed
- `tracing` is used where API/network diagnostics matter, especially `crates/asana-cli/src/main.rs`.
- Most other crates use plain `println!`/`eprintln!` output with optional JSON modes, such as `crates/gator/src/main.rs`, `crates/todoer/src/main.rs`, and `crates/silent-critic/src/main.rs`.

**Patterns:**
- Human-readable output goes to stdout; failures go to stderr unless the command explicitly emits structured JSON.
- JSON modes are exposed as `--json` flags in `crates/todoer/src/cli.rs`, `crates/gator/src/cli.rs`, `crates/prompter/src/lib.rs`, and `crates/silent-critic/src/cli.rs`.

## Comments

**When to Comment:**
- Use crate-level or module-level `//!` docs on public files, especially in crates that inherit `missing_docs = "deny"`. Examples: `crates/cli-common/src/lib.rs`, `crates/gator/src/lib.rs`, `crates/prompter/src/lib.rs`, and `crates/bsky-comment-extractor/src/cli.rs`.
- Use `///` on public CLI structs, fields, and functions. `crates/bsky-comment-extractor/src/cli.rs` is the clearest reference pattern.
- Use inline comments for constraints and safety notes rather than narration. Examples: the PATH mutation comment in `crates/gator/src/lib.rs` and the env-isolation notes in `crates/unvenv/tests/integration_test.rs`.

**JSDoc/TSDoc:**
- Not applicable. The repository is Rust-only in the inspected workspace.

## Function Design

**Size:**
- Small command routers live in `src/main.rs`; larger business logic lives in `src/lib.rs` or `src/commands/*.rs`.
- Large functions are tolerated only with explicit lint waivers, such as `#[allow(clippy::too_many_lines)]` in `crates/unvenv/src/main.rs` and `crates/asana-cli/src/api/client.rs`.

**Parameters:**
- Prefer typed Clap structs (`FetchArgs`, `QueryArgs`, `TaskCommand`) for CLI inputs in `crates/bsky-comment-extractor/src/cli.rs`, `crates/todoer/src/cli.rs`, and `crates/silent-critic/src/cli.rs`.
- Prefer borrowing references into command-layer functions, e.g. `run(cli: &Cli)` in `crates/gator/src/lib.rs` and `run_new(&config, &project, "do thing")` in `crates/todoer/tests/commands_new.rs`.

**Return Values:**
- Library functions usually return `Result<T, E>`.
- Binary `run` functions return exit codes (`i32`) when they need to centralize stdout/stderr policy, as in `crates/todoer/src/main.rs`, `crates/silent-critic/src/main.rs`, and `crates/bsky-comment-extractor/src/main.rs`.

## Module Design

**Exports:**
- Keep `src/lib.rs` as the module registry and re-export surface, as in `crates/cli-common/src/lib.rs`, `crates/gator/src/lib.rs`, and `crates/silent-critic/src/lib.rs`.
- Put CLI definitions in `src/cli.rs` or `src/cli/mod.rs`; put execution logic elsewhere.

**Barrel Files:**
- `mod.rs` is used selectively for grouped domains such as `crates/todoer/src/commands/mod.rs`, `crates/silent-critic/src/commands/mod.rs`, and `crates/asana-cli/src/cli/mod.rs`.
- Most crates otherwise favor flat `*.rs` module files over deep barrel hierarchies.

## Config and CLI Conventions

- Prefer XDG-aware config/data resolution and env overrides, as seen in `crates/todoer/src/config.rs`, `crates/silent-critic/src/config.rs`, and `crates/asana-cli/src/config.rs`.
- Expose top-level operational recipes through `justfile`: `just format`, `just lint`, `just test`, `just dev`, and `just ci`.
- Keep repository workflow policy in checked-in automation: `.github/workflows/ci.yml` defines formatting, lint, test, MSRV, audit, and deny gates; `.github/workflows/release-please.yml` and `.github/workflows/release.yml` define release flow from `main` and `*-v*` tags.

## Repository Workflow Conventions

- Treat `main` as the integration branch: `.github/workflows/ci.yml` and `.github/workflows/release-please.yml` trigger on pushes to `main`.
- Run `just dev` locally before shipping changes and `just ci` when matching the full CI pipeline from `justfile`.
- Keep dependency policy centralized in `deny.toml` and dependency versions centralized in the workspace root `Cargo.toml`.

---

*Convention analysis: 2026-03-23*
