# Technology Stack

**Analysis Date:** 2026-03-23

## Languages

**Primary:**
- Rust 2024 edition - entire workspace defined in `/Users/jfb/Projects/tools/main/Cargo.toml` and implemented under `/Users/jfb/Projects/tools/main/crates/`.

**Secondary:**
- TOML - workspace and crate manifests in `/Users/jfb/Projects/tools/main/Cargo.toml`, crate-level `Cargo.toml` files under `/Users/jfb/Projects/tools/main/crates/*/Cargo.toml`, policy/config parsing in `/Users/jfb/Projects/tools/main/crates/gator/src/config.rs`, `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`, `/Users/jfb/Projects/tools/main/crates/todoer/src/config.rs`, and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/config.rs`.
- YAML - CI/CD automation in `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`, `/Users/jfb/Projects/tools/main/.github/workflows/release-please.yml`, and `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`.
- Markdown - operator and product docs in `/Users/jfb/Projects/tools/main/README.md`, `/Users/jfb/Projects/tools/main/CLAUDE.md`, `/Users/jfb/Projects/tools/main/CRATES.md`, and `/Users/jfb/Projects/tools/main/docs/`.
- JSON - release-please metadata in `/Users/jfb/Projects/tools/main/release-please-config.json` and `/Users/jfb/Projects/tools/main/.release-please-manifest.json`, plus schema output in `/Users/jfb/Projects/tools/main/crates/todoer/schema/todoer-output.schema.json`.

## Runtime

**Environment:**
- Rust toolchain with MSRV 1.94.0 from `[workspace.package]` in `/Users/jfb/Projects/tools/main/Cargo.toml`.
- Nightly Rust is required for formatting via `cargo +nightly fmt` in `/Users/jfb/Projects/tools/main/justfile` and `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`.

**Package Manager:**
- Cargo workspace with resolver `"3"` in `/Users/jfb/Projects/tools/main/Cargo.toml`.
- Lockfile: present at `/Users/jfb/Projects/tools/main/Cargo.lock`.

## Frameworks

**Core:**
- `clap` 4 - command-line parsing across the workspace via `/Users/jfb/Projects/tools/main/Cargo.toml` and crate entrypoints such as `/Users/jfb/Projects/tools/main/crates/unvenv/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs`, and `/Users/jfb/Projects/tools/main/crates/gator/src/main.rs`.
- `serde` + `serde_json` + `toml` - serialization, JSON output, and config/profile parsing in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`, `/Users/jfb/Projects/tools/main/crates/todoer/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs`, and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/main.rs`.
- `reqwest` + `tokio` - async HTTP clients in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs`.
- `rusqlite` with bundled SQLite - local persistence in `/Users/jfb/Projects/tools/main/crates/todoer/src/db.rs`, `/Users/jfb/Projects/tools/main/crates/silent-critic/src/db.rs`, and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/db.rs`.
- `git2` with `vendored-libgit2` - repository discovery and git-aware behavior in `/Users/jfb/Projects/tools/main/crates/unvenv/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/gator/src/config.rs`, and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/project.rs`.

**Testing:**
- Built-in `cargo test` workspace test runner in `/Users/jfb/Projects/tools/main/justfile` and `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`.
- `mockito` and `serial_test` for API/integration-style tests in `/Users/jfb/Projects/tools/main/crates/asana-cli/Cargo.toml` and `/Users/jfb/Projects/tools/main/crates/asana-cli/tests/`.
- `tempfile` for filesystem-isolated tests across crates, declared in manifests such as `/Users/jfb/Projects/tools/main/crates/todoer/Cargo.toml` and `/Users/jfb/Projects/tools/main/crates/gator/Cargo.toml`.

**Build/Dev:**
- `just` task runner in `/Users/jfb/Projects/tools/main/justfile`.
- `clippy` linting and `rustfmt` formatting in `/Users/jfb/Projects/tools/main/Cargo.toml`, `/Users/jfb/Projects/tools/main/rustfmt.toml`, and `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`.
- `cargo-deny` and `cargo-audit` dependency/security checks configured in `/Users/jfb/Projects/tools/main/deny.toml`, `/Users/jfb/Projects/tools/main/justfile`, and `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`.
- Release Please automation in `/Users/jfb/Projects/tools/main/release-please-config.json` and `/Users/jfb/Projects/tools/main/.github/workflows/release-please.yml`.

## Key Dependencies

**Critical:**
- `tftio-cli-common` - shared CLI completions, doctor checks, license output, and update flow used by crates such as `/Users/jfb/Projects/tools/main/crates/unvenv/src/main.rs` and `/Users/jfb/Projects/tools/main/crates/gator/Cargo.toml`.
- `tftio-prompter` - reusable prompt composition library consumed directly by `/Users/jfb/Projects/tools/main/crates/gator/Cargo.toml` and implemented in `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs`.
- `tracing` and `tracing-subscriber` - observability stack for `/Users/jfb/Projects/tools/main/crates/asana-cli/src/lib.rs` and network-heavy Asana flows in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`.
- `directories` and `dirs` - XDG/home-directory path resolution in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`, `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/todoer/src/config.rs`, and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/config.rs`.

**Infrastructure:**
- `secrecy` - token wrapping in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`.
- `indicatif` - progress UX in `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`.
- `dialoguer` - interactive terminal flows in `/Users/jfb/Projects/tools/main/crates/asana-cli/Cargo.toml`.
- `sha2` - hashing for cache keys and project/session IDs in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs` and `/Users/jfb/Projects/tools/main/crates/silent-critic/Cargo.toml`.

## Workspace and Crate Layout

- Workspace members are declared centrally in `/Users/jfb/Projects/tools/main/Cargo.toml`: `cli-common`, `prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`, `gator`, and `bsky-comment-extractor`.
- Dependency versions are centralized in `[workspace.dependencies]` in `/Users/jfb/Projects/tools/main/Cargo.toml`; crate manifests such as `/Users/jfb/Projects/tools/main/crates/gator/Cargo.toml` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/Cargo.toml` opt into them with `workspace = true`.
- Workspace lints live in `[workspace.lints]` in `/Users/jfb/Projects/tools/main/Cargo.toml`; local exceptions are declared in `/Users/jfb/Projects/tools/main/crates/todoer/Cargo.toml` and `/Users/jfb/Projects/tools/main/crates/silent-critic/Cargo.toml`.

## Notable Crate-Specific Technology Choices

- `/Users/jfb/Projects/tools/main/crates/asana-cli/Cargo.toml` extends workspace `reqwest` with `multipart` and `stream`, and extends `tokio` with `fs`, `signal`, and `time` for API uploads, caching, and CLI runtime behavior.
- `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs` implements AT Protocol pagination, JWT refresh, and rate-limit backoff against BlueSky endpoints while `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/db.rs` stores extracted posts in SQLite.
- `/Users/jfb/Projects/tools/main/crates/gator/src/sandbox.rs` generates macOS `sandbox-exec` SBPL policy text, making `gator` platform-specific despite otherwise portable Rust code.
- `/Users/jfb/Projects/tools/main/crates/todoer/src/db.rs` keeps a compact SQLite schema for projects, tasks, and notes, while `/Users/jfb/Projects/tools/main/crates/todoer/src/main.rs` emits structured JSON responses for agent consumption.
- `/Users/jfb/Projects/tools/main/crates/silent-critic/src/db.rs` defines a larger SQLite schema for projects, criteria, sessions, contracts, evidence, decisions, and audit events.
- `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs` uses a TOML profile graph plus a markdown library directory rather than embedding prompt content in code.

## Configuration

**Environment:**
- Asana config and data roots are resolved in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs` using `ASANA_CLI_CONFIG_HOME` and `ASANA_CLI_DATA_HOME`.
- BlueSky extraction depends on `BSKY_APP_PASSWORD` in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/lib.rs` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`.
- Todoer and Silent Critic honor XDG-style paths in `/Users/jfb/Projects/tools/main/crates/todoer/src/config.rs` and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/config.rs`.

**Build:**
- Root build/tooling config files are `/Users/jfb/Projects/tools/main/justfile`, `/Users/jfb/Projects/tools/main/rustfmt.toml`, `/Users/jfb/Projects/tools/main/deny.toml`, `/Users/jfb/Projects/tools/main/release-please-config.json`, and `/Users/jfb/Projects/tools/main/.github/workflows/*.yml`.

## Platform Requirements

**Development:**
- Rust 1.94.0+, Cargo, and nightly rustfmt are required by `/Users/jfb/Projects/tools/main/README.md` and `/Users/jfb/Projects/tools/main/justfile`.
- `curl` and shell execution are assumed by the shared updater in `/Users/jfb/Projects/tools/main/crates/cli-common/src/update.rs`.

**Production:**
- Distribution targets are Linux `x86_64-unknown-linux-gnu`, Linux `aarch64-unknown-linux-gnu`, and macOS `aarch64-apple-darwin` in `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`.
- Library-only publishing is supported for `/Users/jfb/Projects/tools/main/crates/cli-common`, while binary crates publish prebuilt archives and crates.io releases via `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`.

---

*Stack analysis: 2026-03-23*
