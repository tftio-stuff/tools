# Coding Conventions

**Analysis Date:** 2026-03-23

## Naming Patterns

**Files:**
- Module files use snake_case: `error.rs`, `output.rs`, `config.rs`, `models.rs`
- Test files may be integration tests in dedicated `tests/` directory or unit tests in `#[cfg(test)]` modules within source files
- Examples: `crates/todoer/tests/commands_new.rs`, `crates/asana-cli/tests/api_client.rs`

**Functions:**
- Use snake_case for all function names
- Builder pattern methods use `with_` prefix: `with_cache_dir()`, `with_timeout()`, `with_max_retries()`
- Getter methods use descriptive names: `as_str()`, `repo_info()`, `current_version()`
- Examples from codebase: `run_new()`, `open_db()`, `init_db()`, `ensure_project()`, `default_cache_dir()`, `resolve_project_key()`

**Variables:**
- Use snake_case for all variable names
- Private/internal fields in structs use lowercase: `profiles`, `post_prompt`, `expires_at`
- Public struct fields use snake_case: `base_url`, `user_agent`, `timeout`, `cache_ttl`
- Examples: `safehouse_extras`, `policy_denies`, `project_key`

**Types:**
- Use PascalCase for structs, enums, and traits
- Examples: `Config`, `ApiClient`, `Task`, `Status`, `ListResponse`, `Workspace`
- Type aliases use PascalCase: `type Result<T> = anyhow::Result<T>;`

## Code Style

**Formatting:**
- Tool: `rustfmt` configured in `rustfmt.toml`
- Edition: 2024
- Max line width: 100 characters
- Hard tabs: disabled
- Tab spaces: 4
- Force explicit ABI: enabled (`force_explicit_abi = true`)
- Use field init shorthand enabled (`use_field_init_shorthand = true`)
- Use try shorthand enabled (`use_try_shorthand = true`)

**Linting:**
- Tool: `clippy` with strict workspace-level configuration
- Workspace lints (from root `Cargo.toml`):
  - `rust.unsafe_code = "warn"`
  - `rust.missing_docs = "deny"` (must document public items)
  - `clippy.all = "deny"` with priority -1
  - `clippy.pedantic = "deny"` with priority -1
  - `clippy.nursery = "warn"` with priority -1
  - `clippy.cargo = "warn"` with priority -1
  - `enum_glob_use = "deny"`
  - `wildcard_imports = "deny"`
  - Pre-existing allowances: `missing_errors_doc`, `missing_panics_doc`, `too_many_arguments`, `too_many_lines`, `collapsible_if`, `use_self`, `branches_sharing_code`, `missing_const_for_fn`, `significant_drop_tightening`, `uninlined_format_args`, `bool_to_int_with_if`
- Per-crate overrides: `todoer` and `silent-critic` set `missing_docs = "allow"` for local lints

**Documentation:**
- All public items must have doc comments (enforced by lint)
- Use `///` for item documentation
- Use `//!` for module-level documentation
- Include examples in documentation when helpful
- Link to related items using markdown links in doc comments
- Example from `cli-common/src/output.rs`:
  ```rust
  /// Check if stdout is a TTY (terminal).
  ///
  /// Returns `true` if stdout is connected to a terminal, `false` if piped/redirected.
  /// This is used to determine whether to use colored output and fancy formatting.
  #[must_use]
  pub fn is_tty() -> bool {
  ```

## Import Organization

**Order:**
1. Internal crate imports (`use crate::...`)
2. External crate imports grouped by category (dependencies)
3. Standard library imports (std::*)
4. Type-level re-exports and use statements after groups

**Path Aliases:**
- No global path aliases configured
- Imports use fully qualified paths from crate root
- Examples:
  - `use crate::config::{Config, resolve_db_path};`
  - `use crate::db::open_db;`
  - `use crate::api::{ApiClient, ApiError, AuthToken, ListResponse};`

**Import Grouping (observed in asana-cli):**
```rust
// Internal crate modules
use crate::api::{auth::AuthToken, error::{ApiError, RateLimitInfo}, pagination::ListResponse};

// External dependencies
use async_stream::try_stream;
use base64::{Engine as _, engine::general_purpose};
use futures_core::Stream;
use reqwest::{Method, StatusCode, header::{...}};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::...};
use tokio::{fs, sync::RwLock, time::sleep};
use tracing::{debug, warn};
```

## Error Handling

**Patterns:**
- Use `anyhow::Result<T>` type alias as primary error type
- Example from `asana-cli/src/error.rs`:
  ```rust
  /// Result type alias leveraging `anyhow` for rich context.
  pub type Result<T> = anyhow::Result<T>;
  ```
- Propagate errors with `?` operator in functions returning `Result`
- Use `anyhow::anyhow!()` macro to construct contextual errors
- Example from `asana-cli/src/lib.rs`:
  ```rust
  fmt()
      .with_env_filter(filter)
      .with_target(false)
      .try_init()
      .map_err(|err| anyhow!(err))?;
  ```

**Error Messages in CLI:**
- Main entry points convert errors to exit codes
- Example from `gator/src/main.rs`:
  ```rust
  if let Err(e) = gator::run(&cli) {
      if json {
          eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
      } else {
          eprintln!("gator: {e}");
      }
      std::process::exit(1);
  }
  ```
- Support JSON output for errors when `--json` flag is present

## Logging

**Framework:** `tracing` crate for structured logging

**Initialization:**
- Initialize tracing subscriber in main entry points
- Example from `asana-cli/src/lib.rs`:
  ```rust
  pub fn init_tracing() -> Result<()> {
      let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
      fmt()
          .with_env_filter(filter)
          .with_target(false)
          .try_init()
          .map_err(|err| anyhow!(err))?;
      Ok(())
  }
  ```

**Patterns:**
- Use `tracing::{debug, warn, info, error}` macros for logging
- Include contextual information in log messages
- Example usage in `api/client.rs`: `debug!(...)`, `warn!(...)`

## Comments

**When to Comment:**
- Public API documentation via doc comments is mandatory (enforced by lint)
- Explain WHY not WHAT - code should be clear enough to explain what it does
- Use comments for non-obvious algorithmic choices or workarounds
- Example from `gator/src/lib.rs`:
  ```rust
  // SAFETY: gator is single-threaded and sets PATH before spawning anything.
  unsafe { std::env::set_var("PATH", ...); }
  ```

**JSDoc/TSDoc:**
- Use Rust doc comments (`///` and `//!`)
- Include parameter and return value documentation
- Use code examples in doc comments when appropriate
- Example from `cli-common/src/lib.rs`:
  ```rust
  //! Common functionality for Workhelix Rust CLI tools.
  //!
  //! This library provides shared functionality for CLI tools including:
  //! - Shell completion generation
  //! - Health check framework
  //! - License display
  //! - Terminal output utilities
  ```

## Function Design

**Size:** No specific upper bound enforced (too_many_lines is allowed), but keep functions focused on single responsibility

**Parameters:**
- Use strong types rather than primitives for clarity
- Builder pattern for complex configuration: `ApiClient::builder(token).base_url(url).cache_dir(dir).build()`
- Example from `api/client.rs`:
  ```rust
  pub fn builder(token: AuthToken) -> ApiClientBuilder {
      ApiClientBuilder::new(token)
  }
  ```

**Return Values:**
- Use `Result<T>` for fallible operations
- Use `Option<T>` for nullable values
- Avoid bare tuples; prefer named structs for multiple return values
- Mark pure functions with `#[must_use]` attribute
- Example from `output.rs`:
  ```rust
  #[must_use]
  pub fn success(msg: &str) -> String { ... }

  #[must_use]
  pub fn is_tty() -> bool { ... }
  ```

## Module Design

**Exports:**
- Re-export commonly used types from lib.rs
- Example from `cli-common/src/lib.rs`:
  ```rust
  pub use doctor::DoctorChecks;
  pub use license::LicenseType;
  pub use types::{DoctorCheck, RepoInfo};
  pub use completions::generate_completions;
  pub use doctor::run_doctor;
  pub use license::display_license;
  ```

**Barrel Files:**
- Use `mod.rs` files for module organization in subdirectories
- Re-export key types at the barrel level
- Example in `crates/commands/mod.rs` and submodule files

**Module Structure:**
- Organize by domain: `cli`, `commands`, `config`, `db`, `models`, `output`, etc.
- Keep related functionality together
- One concept per file when possible
- Examples:
  - `todoer` modules: `cli.rs`, `commands/`, `config.rs`, `db.rs`, `models.rs`, `output.rs`, `project.rs`, `repo.rs`
  - `asana-cli` modules: `api/`, `cli.rs`, `config.rs`, `doctor.rs`, `error.rs`, `filters.rs`, `models.rs`, `output.rs`, `templates.rs`

**UNSAFE Code:**
- Minimize unsafe code; only use when necessary
- Always add `SAFETY:` comment explaining the invariant
- Example from `gator/src/lib.rs`:
  ```rust
  // SAFETY: gator is single-threaded and sets PATH before spawning anything.
  unsafe { std::env::set_var("PATH", ...); }
  ```

## Builder Pattern

Used extensively for complex initialization (particularly in `asana-cli`):
- Return `Self` from builder methods for method chaining
- Mark builder methods with `#[must_use]`
- Example from `api/client.rs`:
  ```rust
  #[must_use]
  pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self {
      self.cache_dir = cache_dir;
      self
  }
  ```

---

*Convention analysis: 2026-03-23*
