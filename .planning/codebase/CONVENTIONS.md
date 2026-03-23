# Coding Conventions

**Analysis Date:** 2026-03-23

## Naming Patterns

**Files:**
- `snake_case` for all `.rs` files: `error.rs`, `api_client.rs`, `custom_fields.rs`
- Module directories `snake_case` with `mod.rs` entry: `crates/asana-cli/src/api/mod.rs`
- Crate directories use `kebab-case`: `crates/cli-common/`, `crates/asana-cli/`, `crates/silent-critic/`

**Types (structs/enums):**
- `PascalCase` throughout: `ApiClient`, `SessionStatus`, `ContractSandbox`, `RateLimitInfo`
- Enum variants `PascalCase`: `SessionStatus::AwaitingAdjudication`, `EvaluatorType::HumanJudgment`
- Type aliases also `PascalCase`: `pub type Result<T> = anyhow::Result<T>;`

**Functions and methods:**
- `snake_case` for all functions: `open_db`, `init_db`, `run_new`, `resolve_db_path`, `build_command`
- Command entry points follow `run_<subcommand>` convention: `run_new`, `run_list`
- Builder methods: `with_` prefix for optional fields, no prefix for required: `.base_url()`, `.cache_dir()`, `.max_retries()`
- Boolean-returning predicates use `is_` or `can_` prefix: `is_tty()`, `can_transition_to()`
- Conversion helpers: `as_str()` for `&'static str`, `to_string()` via `Display`

**Constants:**
- `SCREAMING_SNAKE_CASE`: `ENV_CONFIG_HOME`, `ENV_DATA_HOME`, `ENV_TOKEN`, `VERSION`

**Variables:**
- `snake_case` throughout: `safehouse_extras`, `policy_denies`, `project_key`

## Code Style

**Formatter:** `rustfmt` (nightly required for formatting)

**Key settings** (`rustfmt.toml`):
- `edition = "2024"`
- `max_width = 100`
- `hard_tabs = false`, `tab_spaces = 4`
- `force_explicit_abi = true`
- `use_field_init_shorthand = true`
- `use_try_shorthand = true`

**Linter:** `clippy` with strict workspace deny configuration

**Active lint levels** (workspace `Cargo.toml`):
- `clippy::all = "deny"` (priority -1)
- `clippy::pedantic = "deny"` (priority -1)
- `clippy::nursery = "warn"` (priority -1)
- `clippy::enum_glob_use = "deny"` - enum variants must be fully qualified
- `clippy::wildcard_imports = "deny"` - no `use foo::*`
- `rust::missing_docs = "deny"` - all public items require doc comments
- `rust::unsafe_code = "warn"`

**Lint exceptions** (allowed workspace-wide due to pre-existing code, fix incrementally):
- `missing_errors_doc`, `missing_panics_doc` (doc completeness)
- `too_many_arguments`, `too_many_lines` (size limits)
- `uninlined_format_args`, `use_self`, `bool_to_int_with_if`, `collapsible_if`

**Per-crate overrides:**
- `todoer` and `silent-critic` disable `missing_docs` locally
- Test modules annotate `#[allow(unsafe_code)]` when manipulating env vars

## Import Organization

**Group order** (standard Rust convention enforced by rustfmt):
1. Internal crate: `use crate::api::...`, `use crate::models::...`
2. External crates: `use anyhow::...`, `use serde::...`, `use tokio::...`
3. Standard library: `use std::...`

**No wildcard imports** (`clippy::wildcard_imports = "deny"`). All symbols must be explicitly named.

**No enum glob use** (`clippy::enum_glob_use = "deny"`). Write `Status::New` not `use Status::*; New`.

**Example** (`crates/asana-cli/src/api/tasks.rs`):
```rust
use crate::{
    api::{ApiClient, ApiError},
    models::{Task, TaskCreateRequest, TaskListParams},
};
use futures_util::{StreamExt, pin_mut};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tracing::debug;
```

## Error Handling

Two distinct strategies - choose based on context:

**Library-facing code: `anyhow` with context**

Define `pub type Result<T> = anyhow::Result<T>;` in `error.rs`:
```rust
// crates/asana-cli/src/error.rs
pub type Result<T> = anyhow::Result<T>;
```

Use `anyhow::bail!` for early returns with a message:
```rust
anyhow::bail!("database not initialized");
```

Annotate errors with `.context()` or `.with_context()` at callsites:
```rust
// crates/silent-critic/src/db.rs
let conn = Connection::open(path)
    .with_context(|| format!("opening database: {}", path.display()))?;
```

**Typed API errors: `thiserror` enums**

Define structured enums in a dedicated `error.rs` (see `crates/asana-cli/src/api/error.rs`):
```rust
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("HTTP {status}: {message}")]
    Http { status: StatusCode, message: String, details: Option<Value> },
    #[error("rate limited after {retry_after:?}: {body}")]
    RateLimited { retry_after: Duration, body: String },
}
```
- Use `#[from]` for automatic conversion from standard error types
- Convenience constructors annotated `#[must_use]` and `pub const fn`

**Main binary entry points:**
```rust
// crates/asana-cli/src/main.rs
match cli::run() {
    Ok(code) => std::process::exit(code),
    Err(err) => {
        tracing::error!(error = %err, "command execution failed");
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
```

**`prompter` uses `Result<T, String>`** - pre-existing pattern. Do not propagate to new crates.

## Enums as Typed Strings

All domain enums follow a strict four-trait pattern. Implement all four when adding a new enum:

```rust
impl MyEnum {
    pub fn as_str(&self) -> &'static str { match self { ... } }
}
impl FromStr for MyEnum {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "value" => Ok(Self::Variant),
            _ => Err(format!("invalid myenum: {s}")),
        }
    }
}
impl Serialize for MyEnum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for MyEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
// Add fmt::Display if the type appears in user-facing output:
impl fmt::Display for MyEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}
```

Enums serialize as lowercase snake_case strings: `"awaiting_adjudication"`, `"tool_authored"`. Never as integers or capitalized strings.

## Documentation Comments

**Required for all public items** (`missing_docs = "deny"` enforced except in `todoer` and `silent-critic`):

- Module-level: `//!` at file top
- Functions: single-line summary, then `/// # Errors` section if fallible
- Struct fields: brief description on each field
- Mark pure value-returning functions `#[must_use]`

```rust
/// Check if stdout is a TTY (terminal).
///
/// Returns `true` if stdout is connected to a terminal, `false` if piped/redirected.
#[must_use]
pub fn is_tty() -> bool { ... }
```

**SAFETY comments:** Always add `// SAFETY:` comment before `unsafe` blocks explaining the invariant.

## Structs and Data Models

**Standard derives:**
- Data transfer/storage structs: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Enums as flags/state: `#[derive(Debug, Clone, PartialEq, Eq)]`
- Clap output format enums: `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]`

**Serde attributes:**
- `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields
- `#[serde(default)]` on `Vec` fields to allow missing keys in deserialization
- No renaming attributes - Rust `snake_case` field names match serialized form

## Configuration Pattern

All crates resolve config via XDG with `dirs` fallback:
```rust
pub fn resolve_config_path() -> anyhow::Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join("crate-name/config.toml"))
    } else {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Ok(home.join(".config/crate-name/config.toml"))
    }
}
```

Config structs derive `Deserialize` and `Default`. `load_config()` returns `Config::default()` when the file is absent (graceful degradation).

## Logging

**Framework:** `tracing` crate only (no `println!` for diagnostics in library code)

**Initialization** (once per binary in `main.rs` or `lib.rs`):
```rust
pub fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).try_init()
        .map_err(|err| anyhow!(err))?;
    Ok(())
}
```

**Usage:** `tracing::debug!(...)`, `tracing::warn!(...)`, `tracing::error!(error = %err, "message")`

## Output Rendering

**TTY detection:** Use `is_terminal::IsTerminal` via `cli-common`'s `output::is_tty()`. Return colored output for TTY, plain `[OK]`/`[ERROR]`/`[WARNING]`/`[INFO]` prefixed text when piped.

**Structured output formats:** Commands support `table`, `json`, `csv`, `markdown` via `--format` flag backed by `clap::ValueEnum` enums defined in `output/mod.rs`.

**Agent-facing JSON envelope** (e.g., `todoer`): `{"ok": true, "data": ...}` wrapper via `output::ok_response(command, payload)`.

## Module Design

**Exports:** `lib.rs` explicitly declares public modules with doc comments. Avoid re-exporting except in `cli-common` where shared types are surfaced at crate root.

**No wildcard re-exports.** Consumers import from specific sub-paths.

**Commands module pattern:**
- `commands/mod.rs` declares submodules
- Each subcommand in its own file: `new.rs`, `list.rs`, `task.rs`, `init.rs`
- Each file exports one `run_<name>` function returning a typed result struct (e.g., `NewResult { task: Task }`)

**Builder pattern** (complex config structs like `ApiClient`):
```rust
ApiClient::builder(token)
    .base_url(url)
    .cache_dir(dir)
    .max_retries(3)
    .build()
    .expect("client initialises")
```

---

*Convention analysis: 2026-03-23*
