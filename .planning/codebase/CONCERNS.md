# Codebase Concerns

**Analysis Date:** 2026-03-22

## Tech Debt

**Asana doctor command is a stub:**
- Issue: `tool_specific_checks()` returns an empty `Vec::new()` with a TODO comment
- Files: `crates/asana-cli/src/doctor.rs` (line 39)
- Impact: `asana-cli doctor` performs no tool-specific health checks despite the framework being present. Token validation, config file existence, and network connectivity are unchecked.
- Fix approach: Implement checks for PAT presence, config file validity, and reachability of `app.asana.com`.

**Todoer `status.parse().unwrap()` in production DB layer:**
- Issue: Row-to-model mapping in the SQLite query layer calls `.parse().unwrap()` on status strings read from the database
- Files: `crates/todoer/src/repo.rs` (lines 46, 67, 83)
- Impact: If the database contains an unrecognized status string (manual edit, schema migration, or future enum change), the process panics rather than returning an error to the caller.
- Fix approach: Use `.parse().map_err(|e| rusqlite::Error::FromSqlConversionFailure(...))` to propagate errors through the `rusqlite::Result` chain.

**Multiple independent tokio runtimes in asana-cli:**
- Issue: Each CLI subcommand creates its own `tokio::Runtime` via `RuntimeBuilder`
- Files: `crates/asana-cli/src/cli/task.rs`, `crates/asana-cli/src/cli/project.rs`, `crates/asana-cli/src/cli/tag.rs`, `crates/asana-cli/src/cli/mod.rs`
- Impact: No shared runtime context; resource overhead and inconsistent configuration per subcommand. Makes tracing/span propagation across subcommands impossible.
- Fix approach: Create a single runtime in `main.rs` and pass `async fn` handlers into `.block_on()`. All subcommands share one runtime.

**Asana CLI cache directory creation silently succeeds on failure:**
- Issue: `fs::create_dir_all(parent).await.ok()` in `write_cache()` swallows directory creation errors
- Files: `crates/asana-cli/src/api/client.rs` (line 848)
- Impact: If the cache directory cannot be created (permissions, disk full), the write silently fails and caching is permanently disabled without any user-visible signal.
- Fix approach: Log a warning when cache directory creation fails.

**Asana CLI operator token stored as plain `String` not `SecretString`:**
- Issue: `FileConfig.personal_access_token` is `Option<String>`, not `Option<SecretString>`
- Files: `crates/asana-cli/src/config.rs` (line 43)
- Impact: The PAT can appear in debug output, serialization logs, and crash reports. `secrecy` is already a dependency but not applied to the persisted field.
- Fix approach: Store as `Option<SecretString>` in `FileConfig`. The `Debug` impl already redacts the value but the type does not enforce it.

## Known Bugs

**Asana CSV output panics on in-memory CSV writer error:**
- Symptoms: If the in-memory `csv::Writer` encounters an encoding error, `wtr.into_inner().unwrap()` panics
- Files: `crates/asana-cli/src/cli/task.rs` (lines 2311-2314)
- Trigger: Rare; would require CSV writer internal error (UTF-8 violation in task name)
- Workaround: Only affects `--format csv` output path

## Security Considerations

**Asana PAT stored in plaintext config file:**
- Risk: Personal Access Token written to `~/.config/asana-cli/config.toml` as plain text
- Files: `crates/asana-cli/src/config.rs` (line 43, file write path)
- Current mitigation: `Debug` impl redacts the token with "REDACTED"; file permissions rely on system umask
- Recommendations:
  1. Verify config file is written with `0600` permissions (check `save()` implementation)
  2. Document the security trade-off
  3. Consider macOS Keychain / Linux Secret Service as an alternative

**Silent-critic worker and operator tokens stored as plaintext in SQLite:**
- Risk: Session tokens stored unencrypted in `~/.local/share/silent-critic/<hash>/db.sqlite`
- Files: `crates/silent-critic/src/db.rs` (lines 51-52, 282-295)
- Current mitigation: None; file permissions not explicitly set to `0600`
- Recommendations:
  1. Set database file permissions to `0600` on creation
  2. Consider encrypting token columns or using SQLCipher
  3. Add token rotation documentation

**Gator SBPL policy: unescaped paths injected directly into S-expression:**
- Risk: Path strings from config are interpolated directly into SBPL policy via `path.display()` without escaping special characters (spaces, quotes, backslashes, unicode)
- Files: `crates/gator/src/sandbox.rs` (lines 47-70)
- Current mitigation: None; no sanitization of path strings before embedding in policy
- Recommendations:
  1. Validate that all path strings contain only characters safe for SBPL literal/subpath rules
  2. Reject or escape paths containing `"`, `\`, or non-ASCII characters
  3. Add regression test with a path containing spaces or special characters

**Unsafe `std::env::set_var()` in production path:**
- Risk: Environment mutation is unsafe in multi-threaded programs; Rust 2024 edition makes this a hard `unsafe` requirement
- Files: `crates/gator/src/lib.rs` (lines 27-29)
- Current mitigation: SAFETY comment states gator is single-threaded and sets PATH before spawning. This holds for current architecture.
- Recommendations:
  1. Pass the modified PATH explicitly to the spawned `Command` via `.env("PATH", ...)` instead of mutating the process environment
  2. Remove the `unsafe` block entirely

## Performance Bottlenecks

**Asana CLI subtask N+1 fetching:**
- Problem: For each task returned by a list query, a separate API request is made to fetch subtasks
- Files: `crates/asana-cli/src/api/tasks.rs`
- Cause: Asana API's `opt_expand=subtasks` is unreliable; subtask counts require per-task calls
- Improvement path:
  1. Add `--no-subtasks` flag to skip subtask fetching entirely
  2. Batch concurrent requests (async join) rather than sequential fetching
  3. Cache subtask data with same TTL as parent tasks

**In-memory API response cache without eviction:**
- Problem: `HashMap`-backed in-memory cache in `ApiClient` has no size cap or eviction policy
- Files: `crates/asana-cli/src/api/client.rs` (field `memory_cache: Arc<RwLock<HashMap<...>>>`)
- Cause: Cache grows unbounded for the lifetime of a process; CLI tools are short-lived so impact is low today
- Improvement path: Add LRU eviction or cap entry count if long-running process use cases emerge

## Fragile Areas

**Silent-critic: no database schema migration mechanism:**
- Files: `crates/silent-critic/src/db.rs` (lines 24-130)
- Why fragile: All tables created with `CREATE TABLE IF NOT EXISTS` in a single batch. Any schema change (new column, renamed table) requires manual migration or database deletion. No `PRAGMA user_version` tracking.
- Safe modification: Only additive changes (new nullable columns) are safe without a migration system
- Test coverage: Schema creation is tested; schema evolution is not

**Silent-critic: session state machine transitions not type-enforced:**
- Files: `crates/silent-critic/src/commands/session.rs`, `crates/silent-critic/src/db.rs`
- Why fragile: `SessionStatus` transitions (Discovering -> Composing -> Ready -> Executing -> AwaitingAdjudication -> Adjudicated) are enforced only at the DB query layer via `get_active_session_in_status()`. Invalid transitions are detected at runtime, not compile time.
- Safe modification: Always call `get_active_session_in_status()` before state-dependent operations
- Test coverage: Individual command transitions tested; invalid transition rejection paths are not

**Gator: macOS-only with no compile-time platform guard:**
- Files: `crates/gator/src/agent.rs` (line 5: `use std::os::unix::process::CommandExt`), `crates/gator/src/sandbox.rs`
- Why fragile: `sandbox-exec` is a macOS-only tool. The crate compiles on Linux (unix path) but will fail at runtime. No `#[cfg(target_os = "macos")]` guards prevent non-macOS builds from producing a non-functional binary.
- Safe modification: Add `#[cfg(target_os = "macos")]` to the `agent` module and CLI. CI only tests on macOS for this crate.
- Test coverage: Agent tests mock the command; sandbox tests skip if base profile file absent

**Gator: silent-critic integration via shell-out without version pinning:**
- Files: `crates/gator/src/session.rs` (lines 46-48)
- Why fragile: `gator --session <id>` shells out to `silent-critic` binary found on PATH. If silent-critic is not installed, is the wrong version, or changes its JSON output schema, gator silently fails or panics during JSON parsing.
- Safe modification: Check silent-critic availability during startup. Pin expected API version.
- Test coverage: JSON parsing tested with hardcoded strings; actual binary invocation not tested

**Prompter: progress bar template `.unwrap()` in production init:**
- Files: `crates/prompter/src/lib.rs` (line 955)
- Why fragile: `ProgressStyle::template()` returns `Result` but is called with `.unwrap()` in `init_scaffold()`. The template is hardcoded so this should never fail, but a future template change could introduce a panic in the init path.
- Safe modification: Use `.expect("hardcoded progress bar template is invalid")` with a clear message, or extract the style to a lazy static with a validated template.
- Test coverage: Not covered; init_scaffold is not unit tested

## Scaling Limits

**SQLite write serialization in silent-critic:**
- Current capacity: One connection per process; WAL mode enabled (concurrent reads OK, serialized writes)
- Limit: Multiple simultaneous `silent-critic` CLI invocations (e.g., parallel CI jobs on same machine sharing state) will block on writes
- Scaling path: Use connection pooling (r2d2 or sqlx) if concurrent access patterns emerge; or document that one session per machine is the intended use

## Dependencies at Risk

**tokio feature surface fragmentation:**
- Risk: `asana-cli` overrides tokio with `fs`, `signal`, `time` features; workspace default only specifies `rt`, `macros`, `rt-multi-thread`. Other crates sharing tokio get only the base feature set.
- Files: `crates/asana-cli/Cargo.toml`
- Impact: Code added to other crates that uses `tokio::fs` or `tokio::signal` will compile only if linked with asana-cli; standalone those crates silently lack features.
- Migration plan: Audit and align tokio features at workspace level.

**reqwest with rustls-tls (no native-tls fallback):**
- Risk: Internal certificate authorities or enterprise proxies using custom root CAs are not trusted by rustls's bundled roots
- Files: Workspace `Cargo.toml` (reqwest features)
- Impact: Users behind corporate TLS inspection proxies may get TLS errors with no clear path to add custom CAs
- Migration plan: Document how to set `REQUESTS_CA_BUNDLE` equivalent. Provide `native-tls` feature flag if needed.

## Test Coverage Gaps

**Gator: no tests for non-session (implicit) policy resolution path:**
- What's not tested: The code path in `crates/gator/src/lib.rs` (lines 41-60) that resolves workdir, loads safehouse config, applies named policies
- Files: `crates/gator/src/lib.rs`, `crates/gator/src/config.rs`
- Risk: Changes to config loading, policy merging, or worktree detection could break the default operating mode unnoticed
- Priority: High

**Silent-critic: operator token printed in plaintext to stdout:**
- What's not tested: That operator token exposure is appropriate and intentional in the `session new` output
- Files: `crates/silent-critic/src/main.rs` (lines 180, 187)
- Risk: Token leaks in CI logs or shell history when running `silent-critic session new`
- Priority: Medium

**Asana CLI: offline mode not integration tested:**
- What's not tested: `--offline` flag behavior; network timeout handling; error messages for network failures
- Files: `crates/asana-cli/tests/cli.rs`
- Risk: Offline mode regressions go undetected
- Priority: Low

---

*Concerns audit: 2026-03-22*
