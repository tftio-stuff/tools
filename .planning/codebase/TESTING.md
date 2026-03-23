# Testing Patterns

**Analysis Date:** 2026-03-23

## Test Framework

**Runner:**
- The workspace standard is `cargo test`, wired through `just test` in `justfile` and through the `test` job in `.github/workflows/ci.yml`.
- Tests run on stable Rust in CI across `ubuntu-latest` and `macos-latest` from `.github/workflows/ci.yml`.
- Async tests use `#[tokio::test]`, visible in `crates/asana-cli/tests/api_client.rs` and `crates/asana-cli/src/api/client.rs`.

**Assertion Library:**
- The repository uses Rust’s built-in test harness plus standard assertions (`assert!`, `assert_eq!`, `matches!`, `panic!`).
- JSON assertions use `serde_json::Value`, for example in `crates/todoer/tests/json_output.rs` and `crates/bsky-comment-extractor/tests/query_cli.rs`.

**Run Commands:**
```bash
just test                 # `cargo test --workspace --verbose`
just test-crate todoer    # `cargo test -p todoer --verbose`
cargo test --workspace    # direct workspace run
cargo test -p tftio-gator # direct per-crate run
```

## Test File Organization

**Location:**
- The workspace mixes inline unit tests and `tests/` integration suites.
- Inline `#[cfg(test)]` modules are common in `crates/cli-common/src/lib.rs`, `crates/gator/src/cli.rs`, `crates/bsky-comment-extractor/src/cli.rs`, `crates/asana-cli/src/config.rs`, and `crates/silent-critic/src/commands/session.rs`.
- Integration tests live under `crates/*/tests/*.rs`, especially `crates/todoer/tests/`, `crates/prompter/tests/cli.rs`, `crates/asana-cli/tests/`, `crates/unvenv/tests/integration_test.rs`, and `crates/bsky-comment-extractor/tests/query_cli.rs`.

**Naming:**
- Use descriptive `snake_case` file names such as `commands_new.rs`, `config_resolution.rs`, `query_cli.rs`, and `api_client.rs`.
- Inline test function names are sentence-like `snake_case`, e.g. `parse_minimal` in `crates/gator/src/cli.rs` and `query_does_not_require_bsky_app_password` in `crates/bsky-comment-extractor/tests/query_cli.rs`.

**Structure:**
```text
crates/<crate>/
├── src/*.rs              # production code with `#[cfg(test)]` modules when unit-level access matters
└── tests/*.rs            # binary, integration, and contract-style tests
```

## Test Structure

**Suite Organization:**
```rust
#[test]
fn query_outputs_jsonl() {
    let (_tmp, db_path) = seeded_db();

    let output = Command::new(bin_path())
        .args(["query", "--db"])
        .arg(&db_path)
        .env_remove("BSKY_APP_PASSWORD")
        .output()
        .expect("run query");

    assert!(output.status.success());
}
```
- The pattern above is taken from `crates/bsky-comment-extractor/tests/query_cli.rs`.

**Patterns:**
- Setup is usually local helper-first: `seed_db()` in `crates/bsky-comment-extractor/tests/query_cli.rs`, `standard_env()` in `crates/asana-cli/tests/cli.rs`, `tmp_home()` in `crates/prompter/tests/cli.rs`, and `clear_git_env()` in `crates/unvenv/tests/integration_test.rs`.
- Teardown relies on RAII temp directories from `tempfile::TempDir` or `tempfile::tempdir`.
- Assertions usually validate both success/failure status and exact output shape, not just type-level success.

## Mocking

**Framework:** targeted, not global
- `mockito` is the primary HTTP mocking tool in `crates/asana-cli/tests/api_client.rs`, `crates/asana-cli/tests/cli.rs`, and `crates/asana-cli/src/api/client.rs`.
- `serial_test` is used only where process-wide env mutation must not race, specifically `crates/asana-cli/src/config.rs`.

**Patterns:**
```rust
let mut server = Server::new_async().await;
let _first = server
    .mock("GET", "/users/me")
    .with_status(429)
    .with_header("Retry-After", "0.05")
    .create();
```
- This is the prevailing API-test pattern in `crates/asana-cli/tests/api_client.rs`.
- Outside HTTP code, the repository prefers real temp files, real SQLite connections, and real subprocesses over trait-level mocks.

**What to Mock:**
- Mock remote HTTP boundaries in `crates/asana-cli`.
- Fake environment roots and config homes with temp dirs in `crates/prompter/tests/cli.rs`, `crates/asana-cli/tests/cli.rs`, and `crates/todoer/tests/config_resolution.rs`.

**What NOT to Mock:**
- SQLite access is usually exercised against real temp databases, as in `crates/todoer/tests/commands_new.rs` and `crates/bsky-comment-extractor/tests/query_cli.rs`.
- CLI parsing is exercised through actual `clap::Parser` or spawned binaries instead of custom stubs in `crates/gator/src/cli.rs`, `crates/todoer/tests/cli_parse.rs`, and `crates/prompter/tests/cli.rs`.

## Fixtures and Factories

**Test Data:**
```rust
let conn = open_db(path)?;
init_db(&conn)?;
upsert_post(&conn, uri, "did:plc:alice", text, created_at, &format!(r#"{{"uri":"{uri}"}}"#))?;
```
- `crates/bsky-comment-extractor/tests/query_cli.rs` uses this seed-helper style to build contract tests around actual database contents.

**Location:**
- Helpers are usually local to each test file instead of shared support modules. Examples: `seed_db()` in `crates/bsky-comment-extractor/tests/query_cli.rs`, `bin_path()` in `crates/prompter/tests/cli.rs`, and `run_command_with_env()` in `crates/asana-cli/tests/cli.rs`.

## Coverage

**Requirements:** None enforced
- No coverage command exists in `justfile`.
- No coverage upload or threshold config exists under `.github/`.
- CI in `.github/workflows/ci.yml` enforces format, lint, test, MSRV, audit, and deny, but not statement or branch coverage.

**Current Signals:**
- Approximate test attribute counts from the current workspace scan: `crates/asana-cli` 72, `crates/gator` 55, `crates/bsky-comment-extractor` 46, `crates/prompter` 42, `crates/unvenv` 29, `crates/silent-critic` 27, `crates/cli-common` 25, and `crates/todoer` 25.
- Inline test density is highest in `crates/asana-cli`, `crates/gator`, and `crates/cli-common`.
- `crates/todoer` is the opposite pattern: it has many `tests/*.rs` files and no inline `#[cfg(test)]` modules in `src/`.

**View Coverage:**
```bash
cargo test --workspace --verbose
just ci
```
- These commands show pass/fail confidence only. They do not emit coverage percentages in the checked-in tooling.

## Test Types

**Unit Tests:**
- Inline unit tests cover parsing, config resolution, and internal model logic in `crates/gator/src/cli.rs`, `crates/cli-common/src/lib.rs`, `crates/silent-critic/src/models.rs`, and `crates/bsky-comment-extractor/src/db.rs`.

**Integration Tests:**
- CLI integration is strong in `crates/prompter/tests/cli.rs`, `crates/asana-cli/tests/cli.rs`, `crates/unvenv/tests/integration_test.rs`, and `crates/bsky-comment-extractor/tests/query_cli.rs`.
- DB-backed command integration is strongest in `crates/todoer/tests/commands_*.rs`.

**E2E Tests:**
- No dedicated E2E framework such as `nextest`, `cargo-llvm-cov`, or browser-driven tooling is configured.
- The closest E2E-style tests are spawned binary tests in `crates/prompter/tests/cli.rs`, `crates/unvenv/tests/integration_test.rs`, and `crates/asana-cli/tests/cli.rs`.

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn rate_limit_recovers_after_retry() { /* ... */ }
```
- Use this style for async API flows in `crates/asana-cli/tests/api_client.rs`.

**Error Testing:**
```rust
let failure = Command::new(bin_path())
    .args(["query", "--db"])
    .arg(&missing_path)
    .output()
    .expect("run query");

assert!(!failure.status.success());
```
- This command-contract style is used in `crates/bsky-comment-extractor/tests/query_cli.rs`, `crates/prompter/tests/cli.rs`, and `crates/unvenv/tests/integration_test.rs`.

## Crate-Specific Notes

- `crates/asana-cli` has the broadest mix: inline unit tests, CLI integration tests, async API tests, retry-path tests, and an optional live smoke test in `crates/asana-cli/tests/api_client.rs` that runs only when `ASANA_CLI_TEST_TOKEN` is set.
- `crates/prompter` validates real binary behavior through `env!("CARGO_BIN_EXE_prompter")` and temp HOME directories in `crates/prompter/tests/cli.rs`.
- `crates/unvenv` uses subprocess-heavy integration tests in `crates/unvenv/tests/integration_test.rs`; those tests build the binary with `cargo build --bin unvenv` and hardcode `../../target/debug/unvenv`.
- `crates/todoer` leans on real SQLite integration tests in `crates/todoer/tests/` and has only a light JSON shape test for `crates/todoer/src/output.rs`.
- `crates/gator`, `crates/cli-common`, and `crates/silent-critic` currently rely on inline tests only; there is no `tests/` directory for those crates.

## Coverage Gaps and Risks

- There is no repository-level coverage measurement or threshold in `justfile` or `.github/workflows/ci.yml`.
- CI does not test Windows; the matrix in `.github/workflows/ci.yml` covers Linux and macOS only.
- `crates/gator` and `crates/silent-critic` expose user-facing binaries but have no spawned-binary integration suite under `crates/*/tests/`.
- `crates/todoer/tests/config_resolution.rs` mutates process env with `unsafe { std::env::set_var(...) }` but does not use `serial_test`; that file assumes isolated execution.
- `crates/asana-cli/tests/api_client.rs` contains an optional live smoke test branch that is skipped unless external env vars are present, so CI exercises only the mocked path by default.

---

*Testing analysis: 2026-03-23*
