# Technology Stack

**Analysis Date:** 2026-03-23

## Languages

**Primary:**
- Rust 1.94.0 (MSRV) - All production crates (`cli-common`, `prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`, `gator`)
- Bash/Shell - Setup, CI/CD, and test infrastructure

**Secondary:**
- SBPL (Sandbox Policy Language) - macOS sandbox policy rules in `crates/gator/src/sandbox.rs`
- TOML - Configuration files and profile composition

## Runtime

**Environment:**
- macOS primary (SBPL sandbox support via `sandbox-exec`)
- Unix/Linux compatible (vendored libgit2, cross-platform file paths)
- No WASI or WebAssembly targets

**Package Manager:**
- Cargo (workspace) - Centralized at root `Cargo.toml`
- Lockfile: `Cargo.lock` (committed for binary crates)

## Frameworks

**Core Tooling:**
- `clap` 4 - Command-line argument parsing with derive macros
- `tokio` 1 - Async runtime (multi-threaded with `rt-multi-thread` feature)

**HTTP/API:**
- `reqwest` 0.13 - Async HTTP client
  - `asana-cli` overrides with features: `multipart`, `stream` for Asana API
  - Default features: `json`, `query`, `rustls`

**Serialization:**
- `serde` 1 + `serde_json` 1 - Serialization/deserialization
- `serde_with` 3 - Custom serialization helpers
- `toml` 1 - TOML parsing for configuration

**Database:**
- `rusqlite` 0.38 - SQLite client (bundled SQLite)
  - Used by: `todoer` (`crates/todoer/src/db.rs`), `silent-critic` (`crates/silent-critic/src/db.rs`)

**Testing:**
- `mockito` 1 - HTTP mocking for API tests (`asana-cli`)
- `serial_test` 3 - Test isolation/serialization (`asana-cli`)
- Unit tests in source files (`#[cfg(test)]` modules)
- No dedicated test framework (uses standard Rust `#[test]`)

**Build/Dev:**
- `cargo-audit` - Dependency vulnerability scanning (CI)
- `cargo-deny` - Dependency compliance checking (licenses, sources) - `deny.toml`
- Just task runner - See `justfile` for dev commands
- Rustfmt (nightly) - Code formatting with strict settings (`rustfmt.toml`)
- Clippy - Linting with workspace-wide deny policies

**Async Utilities:**
- `async-stream` 0.3 - Async stream combinators
- `futures-core` 0.3, `futures-util` 0.3 - Futures utilities
- `tokio-util` 0.7 - Tokio extensions (codec features)

**Observability & Logging:**
- `tracing` 0.1 - Structured logging framework
- `tracing-subscriber` 0.3 - Tracing sink with `env-filter`
- Log level controlled via `RUST_LOG` environment variable (default: `info`)

**Security:**
- `secrecy` 0.10 - Redactable secrets (used in `asana-cli` for PAT tokens)
- `sha2` 0.10 - SHA-256 hashing (project identification in `silent-critic`)

**File/Directory Handling:**
- `directories` 6 + `dirs` 6 - XDG-compliant config/data directories
- `git2` 0.20 - Git operations (vendored libgit2)
  - Features: `vendored-libgit2` for standalone compilation
- `walkdir` 2 - Recursive directory traversal (`unvenv`)
- `tempfile` 3 - Temporary file handling (`gator`, tests)

**UI & Output:**
- `clap_complete` 4 - Shell completion generation
- `colored` 3 - Terminal color output
- `dialoguer` 0.12 - Interactive CLI dialogs (default-features false, features: `editor`, `fuzzy-select`) - Used in `silent-critic` for interactive contract composition
- `tabled` 0.20 - ASCII table rendering
- `indicatif` 0.18 - Progress bars (used in `prompter`)
- `is-terminal` 0.4 - Terminal detection

**Utilities:**
- `anyhow` 1 - Error context handling
- `thiserror` 2 - Error type derivation
- `chrono` 0.4 - Date/time (default-features false, context-specific features)
- `uuid` 1 - UUID generation (v4 feature)
- `regex` 1 - Regular expressions
- `csv` 1 - CSV parsing
- `dateparser` 0.2 - Date string parsing
- `base64` 0.22 - Base64 encoding/decoding
- `rpassword` 7 - Secure password input

## Key Dependencies

**Critical:**
- `git2` - Core to repository detection (`gator`, `silent-critic`)
- `tokio` - Async runtime for all I/O operations
- `rusqlite` - State persistence for `todoer` and `silent-critic`
- `reqwest` - Asana API integration (`asana-cli`)

**Infrastructure:**
- `clap` - CLI interface across all crates
- `serde` ecosystem - Serialization/deserialization throughout
- `tracing` - Observability across all crates

## Configuration

**Environment:**

Project-specific environment variables:
- `ASANA_PAT` - Asana Personal Access Token (`asana-cli`)
- `ASANA_BASE_URL` - Custom Asana API endpoint
- `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, `ASANA_PROJECT` - Asana defaults
- `ASANA_CLI_CONFIG_HOME`, `ASANA_CLI_DATA_HOME` - Custom config/data paths
- `SILENT_CRITIC_TOKEN` - Worker auth token for Silent Critic sessions
- `RUST_LOG` - Tracing level control (default: `info`)
- `XDG_CONFIG_HOME`, `XDG_DATA_HOME` - Standard XDG overrides
- `PATH` - Extended by `gator` with `.local/clankers/bin`

**Build:**
- `Cargo.toml` - Workspace manifest with centralized `[workspace.dependencies]`
- `Cargo.lock` - Committed for reproducible builds
- `rustfmt.toml` - Formatting: edition 2024, 100 char width, explicit ABI, field init shorthand
- `deny.toml` - License allowlist, vulnerability checks, wildcard bans
- No `.cargo/config.toml` overrides detected

## Platform Requirements

**Development:**
- Rust 1.94.0+ (MSRV declared in workspace)
- Nightly Rust (for `cargo fmt` only)
- Git 2.x (for `git2` vendored build)
- Standard build tools (C compiler for vendored libgit2)
- macOS preferred (for SBPL sandbox testing)

**Production:**
- macOS 10.12+ (sandbox-exec availability) for `gator`
- Linux/Unix compatible for other crates (`todoer`, `asana-cli`, `silent-critic`)
- SQLite 3.x support (bundled with `rusqlite`)

---

*Stack analysis: 2026-03-23*
