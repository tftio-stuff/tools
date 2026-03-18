# Testing Patterns

**Analysis Date:** 2026-03-17

## Test Framework

**Runner:**
- Native Rust `#[test]` and `#[cfg(test)]` modules for unit tests
- `#[tokio::test]` macro for async tests (requires `tokio` with `macros` feature)
- Integration tests in dedicated `tests/` directory at crate root
- Config: No explicit test configuration file; uses Cargo's built-in test runner

**Assertion Library:**
- Standard Rust `assert!()`, `assert_eq!()`, `assert_ne!()` macros
- No external assertion library (keep it simple)

**Test Dependencies:**
- `mockito = "1"` - HTTP mocking for API tests
- `serial_test = "3"` - Test isolation and serial execution
- `tempfile = "3"` - Temporary file/directory creation for test fixtures

**Run Commands:**
```bash
cargo test -p <crate-name>           # Run all tests for specific crate
cargo test                             # Run all tests in workspace
cargo test --lib                       # Run unit tests only
cargo test --test '*'                  # Run integration tests
```

## Test File Organization

**Location:**
- **Unit tests:** Co-located with implementation in `#[cfg(test)]` modules within source files
- **Integration tests:** Separate `.rs` files in `tests/` directory at crate root
- Observed in crates: `prompter`, `todoer`, `asana-cli`, `unvenv` have integration tests

**Naming:**
- Integration test files are descriptive: `commands_new.rs`, `cli_parse.rs`, `db_schema.rs`, `api_client.rs`
- Tests use snake_case: `test_init_list_validate_run()`, `new_creates_task()`
- Grouped by domain: todoer has tests for db, cli, commands, config, models, output, project, repo, input

**Structure:**
```
crates/CRATE_NAME/
├── src/
│   ├── lib.rs          # includes mod re-exports
│   ├── module.rs       # contains unit tests in #[cfg(test)]
│   └── ...
└── tests/
    ├── integration_1.rs
    ├── integration_2.rs
    └── ...
```

Example from crates with tests:
- `crates/prompter/tests/cli.rs`
- `crates/todoer/tests/commands_new.rs`, `tests/models.rs`, `tests/db_schema.rs`, etc.
- `crates/asana-cli/tests/api_client.rs`, `tests/cli.rs`

## Test Structure

**Unit Test Suite Pattern (from cli-common/src/output.rs):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tty_returns_bool() {
        let _result = is_tty();
        // Assertion here
    }

    #[test]
    fn test_success_format() {
        let msg = success("test message");
        assert!(msg.contains("test message"));
        assert!(msg.contains("✅") || msg.contains("[OK]"));
    }
}
```

**Integration Test Pattern (from prompter/tests/cli.rs):**
```rust
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

#[test]
fn test_init_list_validate_run() {
    let home = tmp_home("prompter_it_home");
    fs::create_dir_all(&home).unwrap();

    // Test commands via Command::new() spawning actual binary
    let out = Command::new(bin_path())
        .env("HOME", &home)
        .arg("init")
        .output()
        .unwrap();
    assert!(out.status.success(), "init failed: {}", String::from_utf8_lossy(&out.stderr));

    // Further assertions
}
```

**Patterns:**
- Use `super::*` to import items under test
- Separate test setup in dedicated helper functions
- Use temporary directories and files for isolated test state
- Spawn actual binaries in integration tests to verify CLI behavior

## Mocking

**Framework:** `mockito` crate version 1.x for HTTP mocking

**Patterns (from asana-cli/tests/api_client.rs):**
```rust
use mockito::{Matcher, Server};

#[tokio::test]
async fn paginate_workspaces_streams_all_pages() {
    {
        let mut server = Server::new_async().await;
        let _first_page = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_body(r#"{ "data": [...], "next_page": {...} }"#)
            .create();
        let _second_page = server
            .mock("GET", "/workspaces")
            .match_query(Matcher::UrlEncoded("offset".into(), "after-first".into()))
            .with_status(200)
            .with_body(r#"{ "data": [...] }"#)
            .create();

        let client = ApiClient::builder(token)
            .base_url(server.url())
            .cache_dir(cache.path().join("cache"))
            .build()
            .expect("client initialises");

        // Test code using mocked HTTP responses
        drop(server); // Clean up server scope
    }
}
```

**Key Patterns:**
- Create mock server with `Server::new_async().await`
- Set up mocks with `.mock("METHOD", "path").with_status(...).with_body(...).create()`
- Use `Matcher::UrlEncoded()` to match query parameters
- Scope server lifetime with block `{ ... drop(server); }`
- Base URL from mock: `server.url()`

**What to Mock:**
- External HTTP APIs (required for unit/integration testing without network)
- File system operations when testing configuration loading
- Database operations with temporary test databases

**What NOT to Mock:**
- Core business logic (test the real implementation)
- In-memory data structures
- Standard library functions

## Fixtures and Factories

**Test Data (from todoer/tests/commands_new.rs):**
```rust
use tempfile::tempdir;

#[test]
fn new_creates_task() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config {
        db_path: Some(db.to_string_lossy().to_string()),
    };
    let project = ResolvedProject {
        name: "Test".to_string(),
        key: "test".to_string(),
    };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();

    let result = run_new(&config, &project, "do thing").unwrap();
    assert_eq!(result.task.description, "do thing");
}
```

**Location:**
- Test data constructed inline in test functions
- Temporary directories created with `tempfile::tempdir()`
- Helper functions like `tmp_home()` for common setup patterns
- Database initialization: `init_db()`, `ensure_project()` functions used in tests

## Coverage

**Requirements:** Not enforced; no coverage tooling configured

**View Coverage:**
- Not configured; would use `cargo tarpaulin` or `cargo llvm-cov` if needed

## Test Types

**Unit Tests:**
- Scope: Individual functions and modules
- Approach: Co-located with source code in `#[cfg(test)]` modules
- Examples: `output.rs` tests formatting functions, `models.rs` tests serialization
- Minimal dependencies; avoid file I/O when possible

**Integration Tests:**
- Scope: CLI interactions, database operations, API client behavior
- Approach: Standalone `.rs` files in `tests/` directory
- Examples:
  - `prompter/tests/cli.rs` - Full CLI workflows (init, list, validate, run)
  - `todoer/tests/` - Commands (new, list, task operations), database schema, config resolution
  - `asana-cli/tests/api_client.rs` - Async pagination, rate limiting, retries with mocked API
- May spawn actual binaries or test library functions with real fixtures

**E2E Tests:**
- Framework: Not used as dedicated E2E harness
- Closest equivalent: Integration tests that spawn CLI binaries
- Examples: `prompter/tests/cli.rs` runs actual `prompter` binary with various environments

## Async Testing

**Pattern (from asana-cli):**
```rust
#[tokio::test]
async fn rate_limit_recovers_after_retry() {
    let mut server = Server::new_async().await;
    let _first = server
        .mock("GET", "/users/me")
        .with_status(429)
        .with_header("Retry-After", "0.05")
        .create();
    let _second = server
        .mock("GET", "/users/me")
        .with_status(200)
        .create();

    let token = AuthToken::new(SecretString::new("rate-limit-token".into()));
    let client = ApiClient::builder(token)
        .base_url(server.url())
        .build()
        .expect("client initialises");

    // Test async code directly
    let result = client.request::<ResponseType>("GET", "/users/me").await;
    assert!(result.is_ok());
}
```

**Key Points:**
- Use `#[tokio::test]` macro to enable async test execution
- Mock servers support async with `Server::new_async().await`
- Test async streams with `tokio::pin!()` and `.next().await`
- Temporary directories work with async tests via `tokio::fs`

## Common Patterns

**Error Testing:**
- Test both success and failure paths
- Example from `todoer/tests/commands_new.rs`:
  ```rust
  #[test]
  fn new_creates_task() {
      // successful path
      let result = run_new(&config, &project, "do thing").unwrap();
      assert_eq!(result.task.description, "do thing");
  }
  ```

**Serialization Testing:**
```rust
#[test]
fn status_serializes_as_string() {
    let s = Status::InProgress;
    let v = serde_json::to_string(&s).unwrap();
    assert_eq!(v, "\"IN-PROGRESS\"");
}
```

**Test Isolation:**
- Use `#[serial]` attribute from `serial_test` crate for tests that must not run in parallel
- Observed in `asana-cli/tests/` for tests modifying global state
- Example usage:
  ```rust
  #[tokio::test]
  #[serial]
  async fn test_that_needs_isolation() {
      // test code
  }
  ```

**Temporary File Handling:**
```rust
use tempfile::tempdir;

let dir = tempdir().unwrap();
let db_path = dir.path().join("test.db");
// Test with temporary database
// Automatically cleaned up when dir is dropped
```

## Testing Characteristics

- **Granular:** Each crate has focused integration tests
- **Hermetic:** Tests use temporary files and mocked HTTP, no external dependencies
- **Fast:** Unit tests in process, integration tests with mocked I/O
- **Debuggable:** Real binary execution in integration tests (not stubbed)
- **Documented:** Test names describe what they test, minimal comments needed

---

*Testing analysis: 2026-03-17*
