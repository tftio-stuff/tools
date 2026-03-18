# Codebase Concerns

**Analysis Date:** 2026-03-22

## Unsafe Code Usage

**Gator environment variable mutation:**
- Issue: Unsafe `std::env::set_var()` used to prepend clankers to PATH
- Files: `crates/gator/src/lib.rs` (lines 27-29)
- Impact: Sets environment variables in a single-threaded context before spawning agents. While documented as safe with a SAFETY comment, environment mutation is inherently fragile if code organization changes.
- Fix approach: Consider using a thread-local or passing PATH modifications through command-line arguments instead of environment mutation. This would make the contract explicit and eliminate the unsafe block.

**Sandbox policy file path handling:**
- Issue: `.expect()` calls in sandbox.rs when reading base profile
- Files: `crates/gator/src/sandbox.rs` (line 18)
- Impact: If `dirs::home_dir()` returns None, the application panics. macOS-only tool, but no home directory is a critical failure state.
- Fix approach: Return a proper error instead of panicking. Provide helpful diagnostics about missing home directory.

## Error Handling Gaps

**Silent-critic subtask fetching with silent failures:**
- Issue: Subtask fetching errors are silently ignored during task list operations
- Files: `crates/asana-cli/src/api/tasks.rs` (lines 41-57)
- Impact: If subtask fetch fails, tasks are returned without their subtasks but no warning is surfaced to user. Users may think data is complete when it's partial.
- Fix approach: Add verbose logging or a warning flag. Consider collecting errors and reporting summary at end.

**Asana CLI runtime creation:**
- Issue: Multiple CLI modules create tokio runtimes independently with `RuntimeBuilder`
- Files: `crates/asana-cli/src/cli/task.rs`, `src/cli/mod.rs`, `src/cli/project.rs`, `src/cli/tag.rs`, `src/cli/workspace.rs`, `src/cli/user.rs`, `src/cli/custom_field.rs`, `src/cli/section.rs`
- Impact: Each subcommand creates its own tokio runtime. No centralized runtime management means potential resource waste and inconsistent configuration.
- Fix approach: Move runtime creation to a single point in `main.rs` or CLI dispatcher. Pass runtime reference to command handlers.

**Token persistence without explicit validation:**
- Issue: Asana PAT stored on disk with minimal validation
- Files: `crates/asana-cli/src/config.rs` (lines 42-43, 87-94)
- Impact: Invalid or expired tokens are only detected at API call time. Config system doesn't validate token format on load.
- Fix approach: Add token validation on load (basic format check, expiration awareness if available).

## Fragile Areas

**Silent-critic database schema migration:**
- Issue: No versioning of database schema; all tables created at once with `CREATE TABLE IF NOT EXISTS`
- Files: `crates/silent-critic/src/db.rs` (lines 24-130+)
- Impact: Cannot support future schema changes without breaking existing databases. No migration path defined.
- Fix approach: Implement a schema_version table and migration functions. Design migrations to be additive-only.

**Gator sandbox policy assembly:**
- Issue: SBPL policy strings built with string formatting and `writeln!().unwrap()`
- Files: `crates/gator/src/sandbox.rs` (lines 38, 45-50, 55-60, 65-70)
- Impact: String formatting errors would panic. Complex S-expression generation is hard to validate. No syntax checking of generated SBPL.
- Fix approach: Use a typed SBPL builder or formatter that prevents invalid policy generation. Parse generated policy to validate syntax before execution.

**Gator session mode vs non-session mode branching:**
- Issue: Two distinct code paths (session mode with external contracts vs non-session mode with implicit policy resolution) with minimal shared logic
- Files: `crates/gator/src/lib.rs` (lines 34-61)
- Impact: Bugs introduced in one path won't be caught by tests of the other. Configuration loading is different between paths.
- Fix approach: Extract common policy assembly logic. Unify configuration loading regardless of session mode. Add integration tests covering both paths.

**Asana API client cache with RwLock contention:**
- Issue: Single RwLock protecting memory cache shared across all API requests
- Files: `crates/asana-cli/src/api/client.rs` (lines 241, 244)
- Impact: High concurrent request load could cause lock contention. Cache is per-process only; not shared across CLI invocations.
- Fix approach: Consider sharded locking for cache if concurrency becomes bottleneck. Explore persistent caching strategy across invocations.

## Test Coverage Gaps

**Asana CLI mocked testing without offline fallback coverage:**
- Issue: CLI tests use mockito for API mocking, but no coverage of actual offline behavior
- Files: `crates/asana-cli/tests/cli.rs`, related CLI command tests
- Impact: Offline mode (`--offline` flag) is not integration tested. Network errors during real API calls are not well-exercised.
- Fix approach: Add tests that exercise actual network timeouts and offline mode. Test error messages for network failures.

**Silent-critic session state machine:**
- Issue: Session status transitions (Discovering → Composing → Ready → Executing → etc.) not comprehensively tested
- Files: `crates/silent-critic/src/commands/session.rs` (lines 54, 72-100+)
- Impact: Invalid state transitions could occur undetected. Contract composition logic is complex and untested.
- Fix approach: Add unit tests for state machine transitions with invalid/valid state combinations.

**Gator policy generation edge cases:**
- Issue: No tests for sandbox policy assembly with complex path structures, symlinks, or special characters
- Files: `crates/gator/src/sandbox.rs`, `crates/gator/src/lib.rs`
- Impact: Edge cases in path handling (symlinks, spaces, unicode) could cause policy syntax errors at runtime.
- Fix approach: Add unit tests for `emit_ancestors()`, `emit_rw_grant()`, etc. with various path types. Test generated policy syntax validation.

**Todoer database with minimal integration tests:**
- Issue: Limited test coverage for database operations; most tests are unit tests
- Files: `crates/todoer/tests/` (18 test files but mostly isolated units)
- Impact: Schema changes, query bugs, or concurrent access issues could exist in production.
- Fix approach: Add integration tests with real SQLite databases. Test concurrent access patterns.

## Security Considerations

**Asana token storage in plaintext config:**
- Risk: Personal Access Tokens stored in `~/.config/asana-cli/config.toml`
- Files: `crates/asana-cli/src/config.rs` (lines 87-94, file I/O)
- Current mitigation: Unix file permissions not explicitly set; relies on system umask
- Recommendations:
  1. Explicitly set `0600` permissions on config file after creation (already done in some Unix configs, needs verification)
  2. Document security implications clearly
  3. Consider using system keychain (macOS Keychain, Linux Secret Service) instead of file storage

**Silent-critic worker token in plaintext:**
- Risk: Worker and operator tokens stored in SQLite database without encryption
- Files: `crates/silent-critic/src/db.rs` (lines 51-52)
- Current mitigation: Database file stored in `~/.local/share/silent-critic/` with no explicit permissions
- Recommendations:
  1. Encrypt sensitive columns (worker_token, operator_token)
  2. Set database file permissions to `0600`
  3. Document token rotation procedures

**Gator sandbox escape risk:**
- Risk: Sandbox policy assembly depends on correct path handling and SBPL syntax
- Files: `crates/gator/src/sandbox.rs`
- Current mitigation: Static base profile at `~/.config/sandbox-exec/agent.sb`; dynamic rules added for working directory
- Recommendations:
  1. Validate all path inputs before embedding in SBPL
  2. Sanitize special characters in paths
  3. Add integration tests that verify sandbox denials are enforced

## Performance Bottlenecks

**Asana CLI subtask enumeration during list:**
- Problem: For each task with subtasks, a separate API call is made
- Files: `crates/asana-cli/src/api/tasks.rs` (lines 41-61)
- Cause: Asana API's `opt_expand=subtasks` is deprecated and incomplete; `num_subtasks` unreliable
- Improvement path:
  1. Batch subtask requests if API supports it
  2. Add pagination/limit for subtask fetching (don't fetch all subtasks for all tasks)
  3. Cache subtasks locally in workspace-specific cache
  4. Provide `--no-subtasks` flag to skip this entirely

**Silent-critic evidence collection without batching:**
- Problem: Each criterion check runs as separate command invocation
- Files: `crates/silent-critic/src/commands/session.rs` (contract execution)
- Cause: Evidence model assumes serial command execution for notary pattern
- Improvement path:
  1. Group related checks and run in parallel where safe
  2. Implement check command batching
  3. Cache results across multiple runs of same session

## Dependencies at Risk

**tokio feature surface:**
- Risk: Different crates require different tokio features (`fs`, `signal`, `time`); some features not enabled globally
- Files: `crates/asana-cli/Cargo.toml` (line 47)
- Impact: If asana-cli adds code using `tokio::signal::ctrl_c()`, it works because features are enabled. Other crates using tokio might not have all features available.
- Migration plan: Audit all tokio usage across workspace. Enable required features at workspace level if safe.

**reqwest with rustls configuration:**
- Risk: reqwest built with rustls instead of native TLS; may have compatibility issues with internal CAs
- Files: `Cargo.toml` workspace (reqwest feature config)
- Impact: Users with internal certificate authority will need to configure rustls trust roots
- Migration plan: Document TLS configuration options. Consider adding environment variable for custom CA bundle paths.

**Old mockito version:**
- Risk: mockito 1.x is stable but asana-cli tests heavily dependent on its mock server behavior
- Files: `crates/asana-cli/tests/cli.rs`
- Impact: Security updates to mockito may require test refactoring
- Migration plan: Monitor mockito releases. Consider migrating to wiremock or httptest if needed.

## Scaling Limits

**SQLite database contention in silent-critic:**
- Current capacity: Tested with basic schema; no load testing
- Limit: SQLite writer locks could become bottleneck if multiple sessions write simultaneously
- Scaling path:
  1. Add connection pooling
  2. Implement write batching for audit events
  3. Consider migration to PostgreSQL for multi-process scenarios

**In-memory cache without eviction in asana-cli:**
- Current capacity: Unbounded HashMap for API response cache
- Limit: Long-running sessions with many requests could consume unbounded memory
- Scaling path:
  1. Implement LRU eviction policy for memory cache
  2. Cap memory cache size
  3. Use disk cache more aggressively

## Missing Critical Features

**Error recovery in gator agent execution:**
- Problem: If sandboxed agent crashes or times out, error handling is minimal
- Blocks: Cannot implement robust agent supervision or retry logic
- Fix approach: Add timeout handling, capture stderr/stdout properly, implement retry strategy

**Silent-critic evidence audit trail without timestamps:**
- Problem: Evidence records stored but command execution order/timing not captured
- Blocks: Cannot reconstruct exact execution sequence for debugging
- Fix approach: Add timestamps to each evidence record, track execution order explicitly

**Asana CLI batch operations without rollback:**
- Problem: CreateBatch/UpdateBatch operations don't have transaction semantics
- Blocks: Cannot guarantee all-or-nothing updates
- Fix approach: Implement dry-run mode for batch ops. Add transaction-like behavior with rollback.

---

*Concerns audit: 2026-03-22*
