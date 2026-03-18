# Technology Stack

**Analysis Date:** 2026-03-17

## Languages

**Primary:**
- Rust (edition 2024, MSRV 1.94.0) - All 7 production crates
- Bash/Shell - `scripts/new-crate.sh` scaffolding, CI workflow steps

**Secondary:**
- TOML - Configuration files and prompter profile composition
- SBPL (Sandbox Policy Language) - macOS sandbox policy rules generated in `crates/gator/src/sandbox.rs`

## Runtime

**Environment:**
- Native compiled binary (no runtime VM or interpreter)
- Target platforms: Linux x86_64, Linux aarch64, macOS aarch64
- MSRV: 1.94.0 (declared in `Cargo.toml` `[workspace.package]` `rust-version` field)

**Package Manager:**
- Cargo (workspace resolver = "3")
- Lockfile: `Cargo.lock` committed (binary crates require reproducible builds)

## Frameworks

**CLI Parsing:**
- `clap` 4 (derive feature) - Argument parsing across all binary crates
- `clap_complete` 4 - Shell completion generation (`cli-common`, `prompter`, `unvenv`)

**Async Runtime:**
- `tokio` 1 (rt-multi-thread, macros; `asana-cli` adds fs, signal, time) - Used only in `asana-cli`
- `async-stream` 0.3 - Async stream macros (`asana-cli`)
- `futures-core` 0.3, `futures-util` 0.3 - Futures combinators (`asana-cli`)
- `tokio-util` 0.7 (codec) - Codec framing (`asana-cli`)

**HTTP:**
- `reqwest` 0.13 (json, query, rustls; `asana-cli` adds multipart, stream) - TLS via rustls, no system OpenSSL

**Serialization:**
- `serde` 1 (derive) - Core serialization framework used by all data-model crates
- `serde_json` 1 - JSON support
- `serde_with` 3 - Extended derive helpers (`asana-cli`)
- `toml` 1 - Config file parsing (`asana-cli`, `prompter`, `todoer`, `silent-critic`, `gator`)

**Database:**
- `rusqlite` 0.38 (bundled feature) - Embedded SQLite, no system library required
  - `crates/todoer/src/db.rs`
  - `crates/silent-critic/src/db.rs`

**Observability:**
- `tracing` 0.1 - Structured instrumentation (`asana-cli`)
- `tracing-subscriber` 0.3 (env-filter) - Tracing output (`asana-cli`); level via `RUST_LOG`

**Testing:**
- Standard Rust `#[test]` with `#[cfg(test)]` modules in source files
- `mockito` 1 - HTTP mock server (`asana-cli` dev-dependency)
- `serial_test` 3 - Serialized test execution to isolate env-var mutations (`asana-cli` dev-dependency)
- `tempfile` 3 - Temporary directories in tests (dev-dependency in most crates)

**Build/Dev:**
- `just` - Task runner (`justfile` at workspace root)
- `cargo-audit` - Dependency vulnerability scanning
- `cargo-deny` (`deny.toml`) - License and source compliance
- `cargo fmt` (nightly toolchain required) - Formatting (`rustfmt.toml`)
- `cargo clippy` - Lint with workspace-wide deny policies

## Key Dependencies

**Critical:**
- `clap` 4 - CLI interface for all binary crates; removal requires full argument-parsing rewrite
- `rusqlite` 0.38 (bundled) - Persistence for `todoer` and `silent-critic`; bundled = no system SQLite needed
- `reqwest` 0.13 - All Asana API HTTP communication; rustls = no system OpenSSL
- `git2` 0.20 (vendored-libgit2) - Repository operations in `unvenv`, `silent-critic`, `gator`; vendored = no system libgit2
- `serde` + `serde_json` + `toml` - Config and data interchange throughout workspace

**Infrastructure:**
- `anyhow` 1 - Error propagation with context in binary crates
- `thiserror` 2 - Typed error definitions in library crates (`asana-cli`, `gator`)
- `secrecy` 0.10 - Secret wrapping to prevent accidental logging of API tokens (`asana-cli`)
- `directories` 6 - XDG-compliant app data/config/cache paths (`asana-cli`, `silent-critic`)
- `dirs` 6 - Home directory resolution (`prompter`, `todoer`, `gator`)
- `sha2` 0.10 - SHA-256 project identity hashing (`silent-critic`, `asana-cli`)
- `uuid` 1 (v4) - UUID generation for entity IDs (`todoer`, `silent-critic`)
- `chrono` 0.4 - Date/time operations (default-features false, per-crate feature selection)
- `walkdir` 2 - Recursive directory traversal (`unvenv`, `silent-critic`)
- `colored` 3 - Terminal color output
- `is-terminal` 0.4 - TTY detection for output formatting
- `dialoguer` 0.12 (editor, fuzzy-select; default-features false) - Interactive prompts (`asana-cli`, `silent-critic`)
- `indicatif` 0.18 - Progress bars (`prompter`)
- `tabled` 0.20 - Terminal table rendering (`asana-cli`)
- `csv` 1 - CSV output (`asana-cli`)
- `regex` 1 - Pattern matching (`asana-cli`)
- `base64` 0.22 - Base64 encoding (`asana-cli`)
- `rpassword` 7 - Secure password input (`asana-cli`)
- `dateparser` 0.2 - Natural language date parsing (`asana-cli`)
- `tempfile` 3 - Temporary files (`gator` sandbox policy assembly, tests)

## Configuration

**Environment Variables:**
- `asana-cli`: `ASANA_PAT`, `ASANA_BASE_URL`, `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, `ASANA_PROJECT`, `ASANA_CLI_CONFIG_HOME`, `ASANA_CLI_DATA_HOME`
- `silent-critic`: `SILENT_CRITIC_TOKEN` (worker session auth)
- All crates: `RUST_LOG` (tracing level), `XDG_CONFIG_HOME`, `XDG_DATA_HOME`

**Config Files:**
- `asana-cli`: `~/.config/asana-cli/asana-cli.toml` (permissions enforced to 0o600 on Unix)
- `silent-critic`: `~/.config/silent-critic/config.toml`
- `todoer`: `.todoer.toml` (project-local)
- `gator`: reads prompter profile TOML files

**Build Files:**
- `Cargo.toml` - Workspace root; all dependency versions centralized under `[workspace.dependencies]`
- `Cargo.lock` - Committed for reproducibility
- `rustfmt.toml` - edition 2024, max_width 100, 4-space indent, `use_field_init_shorthand`, `use_try_shorthand`
- `deny.toml` - License allowlist (MIT, Apache-2.0, BSD-*, MPL-2.0, CC0-1.0, etc.); wildcards banned; crates.io only
- `justfile` - Task runner recipes
- `release-please-config.json` - Automated release PR configuration
- `prek.toml` - Pre-commit tool configuration

## Platform Requirements

**Development:**
- Rust stable toolchain (MSRV 1.94.0+)
- Rust nightly toolchain (required only for `cargo fmt`)
- C compiler (for vendored libgit2 build)
- `just` task runner
- `cargo-audit`, `cargo-deny` for quality gate commands
- macOS preferred for full `gator` SBPL sandbox testing

**Production:**
- No system OpenSSL, SQLite, or libgit2 required (all vendored or bundled)
- `gator` requires macOS (uses `sandbox-exec` which is macOS-only)
- All other crates are Linux/Unix compatible
- Release binaries published to GitHub Releases and crates.io

---

*Stack analysis: 2026-03-17*
