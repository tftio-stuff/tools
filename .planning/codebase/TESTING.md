# Testing Patterns

**Analysis Date:** 2026-03-23

## Test Framework

**Runner:**
- `cargo test` (built-in Rust test harness)
- No external test runner config file; workspace root `Cargo.toml` drives all test runs

**Assertion Library:**
- Rust built-in `assert!`, `assert_eq!`, `assert_ne!`, `panic!`
- No third-party assertion libraries

**Async Test Runtime:**
- `tokio` with `#[tokio::test]` attribute for async tests (`asana-cli` only)

**Test Isolation:**
- `serial_test` crate (`v3`, workspace dep) for tests that mutate environment variables

**Run Commands:**
```bash
cargo test                              # All tests, all crates
cargo test -p tftio-todoer              # Single crate
cargo test -p tftio-asana-cli          # Single crate
just test                               # All tests via justfile
just test-crate <crate-name>           # Single crate via justfile
```

## Test File Organization

**Unit tests:** Co-located in `src/` files, inside `#[cfg(test)]` modules at the bottom of the file.

**Integration tests:** In `tests/` directory at the crate root, one file per logical concern.

**Naming:**
- Unit test modules are always named `tests` (`mod tests { ... }`)
- Integration test files named by concern: `db_schema.rs`, `cli_parse.rs`, `commands_new.rs`, `api_client.rs`
- Test function names use snake_case, prefixed with `test_` only in subprocess/integration tests

**Per-crate structure:**
```
crates/todoer/
    src/              # unit tests co-located in #[cfg(test)]
    tests/
        cli_parse.rs
        commands_new.rs
        commands_task.rs
        commands_list.rs
        commands_init.rs
        db_schema.rs
        config_resolution.rs
        json_output.rs
        models.rs
        output_table.rs
        input.rs
        repo.rs
        project_discovery.rs
        project_resolution.rs

crates/asana-cli/
    src/              # unit tests in #[cfg(test)], serial_test for env mutation
    tests/
        api_client.rs   # async tests with mockito
        cli.rs          # subprocess binary tests

crates/prompter/
    src/              # unit tests
    tests/
        cli.rs          # subprocess binary tests via CARGO_BIN_EXE_prompter

crates/unvenv/
    tests/
        integration_test.rs   # subprocess binary tests using real git init

crates/gator/
    src/              # unit tests only (no tests/ directory)

crates/silent-critic/
    src/              # unit tests only, primary coverage in db.rs (~140 lines)
```

## Test Structure

**Unit test module pattern:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptive_behavior_name() {
        // arrange
        // act
        // assert
    }
}
```

**Example from `crates/gator/src/prompt.rs`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_profile_list_prepends_base() {
        let user = vec!["rust.full".to_owned()];
        let all = build_profile_list(&user);
        assert_eq!(all[0], "core.baseline");
        assert_eq!(all[1], "core.agent");
        assert_eq!(all[2], "core.git");
        assert_eq!(all[3], "rust.full");
    }

    #[test]
    fn build_profile_list_empty_user() {
        let all = build_profile_list(&[]);
        assert_eq!(all.len(), 3);
    }
}
```

**In-memory SQLite factory for DB unit tests (`crates/silent-critic/src/db.rs`):**
```rust
fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    init_db(&conn).unwrap();
    conn
}

#[test]
fn project_roundtrip() {
    let conn = test_db();
    let p = Project { id: uuid::Uuid::new_v4().to_string(), ... };
    insert_project(&conn, &p).unwrap();
    let found = get_project_by_repo_hash(&conn, "abc123").unwrap().unwrap();
    assert_eq!(found.name, "test-project");
}
```

**Filesystem-backed SQLite for integration tests (`crates/todoer/tests/`):**
```rust
use tempfile::tempdir;

#[test]
fn new_creates_task() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();
    let result = run_new(&config, &project, "do thing").unwrap();
    assert_eq!(result.task.description, "do thing");
}
```

## Mocking

**Framework:** `mockito` crate (`v1`, workspace dep). Only used in `crates/asana-cli`.

**Async mock server pattern (`crates/asana-cli/tests/api_client.rs`):**
```rust
use mockito::{Matcher, Server};

#[tokio::test]
async fn paginate_workspaces_streams_all_pages() {
    {
        let mut server = Server::new_async().await;
        let _first_page = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_body(r#"{ "data": [...], "next_page": { "offset": "after-first" } }"#)
            .create();
        let _second_page = server
            .mock("GET", "/workspaces")
            .match_query(Matcher::UrlEncoded("offset".into(), "after-first".into()))
            .with_status(200)
            .with_body(r#"{ "data": [...] }"#)
            .create();

        let client = ApiClient::builder(token)
            .base_url(server.url())   // point client at mock server
            .cache_dir(cache.path().join("cache"))
            .build()
            .expect("client initialises");

        // exercise and assert ...
        drop(server);  // always drop at end of inner scope
    }
}
```

**HTTP header testing pattern:**
```rust
server
    .mock("GET", "/users/me")
    .with_status(429)
    .with_header("Retry-After", "0.05")
    .with_body(r#"{ "errors": [ { "message": "Too many requests" } ] }"#)
    .create();
```

**What to mock:** External HTTP API calls; rate limiting and retry behavior.

**What NOT to mock:** SQLite databases (use in-memory or `tempfile` real DBs), filesystem (use `TempDir`), git repositories (use real `git init` in `TempDir`).

## Subprocess / Binary Testing

Used in `prompter`, `unvenv`, `asana-cli`. These spawn the actual compiled binary as a subprocess.

**Binary path via `CARGO_BIN_EXE_*` env var (`crates/prompter/tests/cli.rs`):**
```rust
const fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_prompter")
}
```

**Binary path via manifest dir fallback (`crates/unvenv/tests/integration_test.rs`):**
```rust
fn get_binary_path() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir).join("../../target/debug/unvenv")
}
```

**Subprocess test pattern:**
```rust
let out = Command::new(bin_path())
    .env("HOME", &home)   // isolate $HOME for config paths
    .arg("init")
    .output()
    .unwrap();
assert!(out.status.success(), "init failed: {}", String::from_utf8_lossy(&out.stderr));
let stdout = String::from_utf8_lossy(&out.stdout);
assert!(stdout.contains("expected substring"));
```

**Exit code assertions:**
```rust
assert_eq!(output.status.code(), Some(2));  // specific non-zero exit (policy violation)
assert!(output.status.success());           // zero exit
```

**Git environment isolation for subprocess tests (`crates/unvenv/tests/integration_test.rs`):**
```rust
fn clear_git_env(mut cmd: Command) -> Command {
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_COMMON_DIR");
    cmd
}
// Usage:
let output = clear_git_env(Command::new(binary_path))
    .current_dir(temp_dir.path())
    .output()
    .expect("Failed to execute binary");
```
This guard is required when tests run inside a git hook context where `GIT_DIR` is already set.

## Environment Variable Tests

Tests that call `std::env::set_var`/`remove_var` must be serialized to prevent races in parallel test execution.

**`serial_test` pattern (`crates/asana-cli/src/config.rs`):**
```rust
use serial_test::serial;

#[test]
#[serial]
fn environment_overrides_take_precedence() {
    // set env vars, run code, assert, clean up
}
```

**`with_temp_env` helper pattern (wraps setup/teardown):**
```rust
fn with_temp_env<F>(config_home: &TempDir, data_home: &TempDir, f: F)
where
    F: FnOnce(),
{
    set_env(ENV_CONFIG_HOME, config_home.path());
    set_env(ENV_DATA_HOME, data_home.path());
    f();
    remove_env(ENV_CONFIG_HOME);
    remove_env(ENV_DATA_HOME);
    remove_env(ENV_TOKEN);
    // ... remaining env vars ...
}
```

**Manual `unsafe` env mutation (`crates/todoer/tests/config_resolution.rs`):**
```rust
#![allow(unsafe_code)]

#[test]
fn config_path_respects_xdg_config_home() {
    // SAFETY: test runs in isolation (serial_test or single-threaded)
    unsafe { std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdgconfig") };
    let path = resolve_config_path().unwrap();
    assert_eq!(path, PathBuf::from("/tmp/xdgconfig/todoer/config.toml"));
}
```

## Live / Optional Tests

`asana-cli` has one live smoke test that is skipped unless an env var is provided:

```rust
#[tokio::test]
async fn optional_live_smoke_test() {
    let token = match std::env::var("ASANA_CLI_TEST_TOKEN") {
        Ok(value) if !value.is_empty() => SecretString::new(value.into()),
        _ => return,   // skip silently when token absent
    };
    // real API call against https://app.asana.com/api/1.0
}
```

This pattern keeps CI clean while enabling manual live validation with `ASANA_CLI_TEST_TOKEN` set.

## Fixtures and Test Data

No shared fixture files. All test data is constructed inline or via local helper functions.

**SQLite test helper factory (preferred pattern for DB tests):**
```rust
fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    init_db(&conn).unwrap();
    conn
}
```

**Struct literal construction:**
```rust
let project = ResolvedProject {
    name: "Test".to_string(),
    key: "test".to_string(),
};
```

**Temp directory:**
```rust
let dir = tempfile::tempdir().unwrap();        // auto-cleaned on drop
let dir = tempfile::TempDir::new().unwrap();   // same, explicit handle
```

**Unique temp home directory (subprocess tests):**
```rust
fn tmp_home(prefix: &str) -> PathBuf {
    let mut p = env::temp_dir();
    let unique = format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    p.push(unique);
    p
}
```
- Use this style for async API flows in `crates/asana-cli/tests/api_client.rs`.

## Coverage

**Requirements:** None enforced. No coverage thresholds in CI.

**Tooling:** Not configured. No `cargo-tarpaulin` or `cargo-llvm-cov` in workspace deps or `justfile`.

**Manual coverage:**
```bash
cargo install cargo-tarpaulin
cargo tarpaulin -p tftio-todoer
```

## Test Types by Crate

| Crate | Unit tests | Integration tests | Async tests | Mock HTTP | Subprocess |
|-------|------------|-------------------|-------------|-----------|------------|
| `cli-common` | Yes (`src/`) | No | No | No | No |
| `prompter` | Yes (`src/`) | Yes (`tests/cli.rs`) | No | No | Yes |
| `unvenv` | Yes (`src/`) | Yes (`tests/`) | No | No | Yes |
| `asana-cli` | Yes (`src/`) | Yes (`tests/`) | Yes | Yes (mockito) | Yes |
| `todoer` | Yes (`src/`) | Yes (`tests/`, 14 files) | No | No | No |
| `silent-critic` | Yes (`src/db.rs`) | No | No | No | No |
| `gator` | Yes (`src/`) | No | No | No | No |

**Approximate counts:**
- Sync unit/integration tests: ~247 functions across workspace
- Async tests: 13 (all in `crates/asana-cli`)

## Common Patterns Summary

**Error variant testing:**
```rust
let err = client.get_current_user().await.expect_err("should rate limit");
match err {
    ApiError::RateLimited { retry_after, body } => {
        assert!(retry_after >= Duration::from_millis(10));
        assert!(body.contains("rate limited"));
    }
    other => panic!("expected rate limited error, got {other:?}"),
}
```

**Stdout content assertions:**
```rust
let stdout = String::from_utf8_lossy(&out.stdout);
assert!(stdout.contains("expected substring"));
assert!(stdout.starts_with("expected prefix"));
assert!(stdout.ends_with("expected suffix"));
```

**State machine / roundtrip testing:**
```rust
insert_project(&conn, &p).unwrap();
let found = get_project_by_repo_hash(&conn, "abc123").unwrap().unwrap();
assert_eq!(found.name, "test-project");
assert_eq!(found.id, p.id);
```

**Deduplication / ordering assertions:**
```rust
let common_count = stdout.matches("COMMON").count();
assert_eq!(common_count, 1, "shared file should appear exactly once, found {common_count}");
let common_pos = stdout.find("COMMON").unwrap();
let a_pos = stdout.find("A_ONLY").unwrap();
assert!(common_pos < a_pos, "files should appear in order");
```

---

*Testing analysis: 2026-03-23*
