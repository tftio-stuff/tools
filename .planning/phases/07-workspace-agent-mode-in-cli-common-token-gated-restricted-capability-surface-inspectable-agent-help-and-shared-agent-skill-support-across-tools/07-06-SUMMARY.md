---
phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
plan: 06
subsystem: cli
tags: [rust, clap, cli-common, agent-mode, prompter, rollout]
requires:
  - phase: 07-03
    provides: bce and gator agent-surface migrations
  - phase: 07-04
    provides: todoer and unvenv agent-surface migrations
  - phase: 07-05
    provides: asana-cli and silent-critic agent-surface migrations
provides:
  - shared agent-mode adoption for prompter
  - in-repo workspace CLI consistency smoke including agent-mode rollout coverage
  - final workspace rollout gate for Phase 7
affects: [workspace-agent-mode, cli-consistency, final-rollout-gate]
tech-stack:
  added: []
  patterns: [shared parse_with_agent_surface entrypoints, repo-level agent smoke script, shared test env lock for agent token mutation]
key-files:
  created: [.planning/phases/07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools/07-06-SUMMARY.md]
  modified: [crates/prompter/src/lib.rs, crates/prompter/src/main.rs, crates/prompter/tests/agent_surface.rs, justfile, tests/cli/lib.sh, tests/cli/06-agent-mode.sh, crates/cli-common/src/agent.rs, crates/cli-common/src/command.rs, crates/cli-common/src/lib.rs, crates/bsky-comment-extractor/src/main.rs, Cargo.lock]
key-decisions:
  - "Prompter uses the shared cli-common parse/help path and resolves AppMode after clap parsing instead of maintaining a separate parser surface."
  - "The final rollout gate treats agent-mode help redaction as a shared substrate concern and serializes env-token tests across cli-common modules with one shared mutex."
  - "The repo-level smoke script remains part of just cli-consistency and records ordinary help/help-subcommand redaction plus typo-suggestion suppression."
patterns-established:
  - "Pattern: CLIs with custom runtime dispatch can still adopt the shared agent surface by parsing into the clap model first and mapping into their internal mode enum afterward."
  - "Pattern: Env-mutating tests in parallel Rust modules should share one crate-level test lock when agent activation depends on process environment variables."
requirements-completed: [D-04, D-05, D-06, D-07, D-08, D-09, D-13]
duration: 12m 25s
completed: 2026-03-23
---

# Phase 07 Plan 06: Prompter and Final Rollout Summary

**Shared restricted agent mode for `prompter`, restored repo-wide CLI consistency, and final workspace rollout gating for Phase 7**

## Performance

- **Duration:** 12m 25s
- **Started:** 2026-03-23T14:10:06Z
- **Completed:** 2026-03-23T14:22:31Z
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments
- Migrated `prompter` onto the shared `cli-common` parser/help pipeline while preserving its `AppMode` runtime dispatch.
- Restored `just cli-consistency` to in-repo wiring and kept `tests/cli/06-agent-mode.sh` in the workspace smoke path.
- Closed the final rollout blocker by fixing shared test-env races in `cli-common`, rerunning repo smokes, and passing `cargo test --workspace --verbose`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Adapt `prompter`'s custom `AppMode` parser to the shared agent surface**
   - `e9a0a46` (`test`) — failing `prompter` agent surface tests
   - `15139b8` (`feat`) — shared parser adoption and prompter capability declarations
2. **Task 2: Restore workspace-level CLI consistency checks with a shared agent-mode smoke script**
   - `64fa873` (`test`) — restore repo agent-mode consistency smoke
3. **Task 3: Run the final workspace-wide rollout gate required by Phase 7 validation**
   - pending local commit — shared env-lock fix for `cli-common` test races, final BCE smoke stabilization, and workspace gate rerun

**Plan metadata:** pending final docs commit

## Files Created/Modified
- `crates/prompter/src/lib.rs` - factors `resolve_app_mode(cli: Cli)` out of argument parsing.
- `crates/prompter/src/main.rs` - routes startup through `parse_with_agent_surface` and shared agent help rendering.
- `crates/prompter/tests/agent_surface.rs` - verifies visible skills and hidden-command rejection.
- `justfile` - runs in-repo CLI metadata checks and includes `tests/cli/06-agent-mode.sh` in `cli-consistency`.
- `tests/cli/lib.sh` - adds `run_agent()` with shared token env vars.
- `tests/cli/06-agent-mode.sh` - verifies all migrated tools plus ordinary help/help-subcommand redaction and typo suppression.
- `crates/cli-common/src/agent.rs` - rewrites trailing `help` to `--help` in agent mode and keeps grouped command subtrees visible only when declared.
- `crates/cli-common/src/command.rs` - uses the shared test env lock for agent-token mutations.
- `crates/cli-common/src/lib.rs` - defines the shared test env lock used by parallel env-mutating unit tests.
- `crates/bsky-comment-extractor/src/main.rs` - stabilizes the query-command smoke test with a guaranteed missing temporary database path.
- `Cargo.lock` - records the current workspace package versions used by the final gate.

## Decisions Made
- Recorded the manual inspectability review as performed during execution instead of leaving it for operator follow-up.
- Fixed the `help`/`--help` redaction path in the shared substrate instead of adding tool-local workarounds.
- Treated the `cli-common` env race as rollout fallout because it surfaced only under the required workspace-wide final gate.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Serialized env-token tests across `cli-common` modules**
- **Found during:** Task 3 (final workspace-wide rollout gate)
- **Issue:** `cargo test -p tftio-cli-common --lib --verbose` was flaky because `agent.rs` and `command.rs` each used their own test-only mutex while both mutated `TFTIO_AGENT_TOKEN*` process env vars in parallel.
- **Fix:** Added one crate-level `test_support::env_lock()` and reused it in both test modules.
- **Files modified:** `crates/cli-common/src/lib.rs`, `crates/cli-common/src/agent.rs`, `crates/cli-common/src/command.rs`
- **Verification:** `cargo test -p tftio-cli-common --lib --verbose`
- **Committed in:** pending local Task 3 commit

**2. [Rule 1 - Bug] Stabilized BCE query smoke expectations for the final workspace gate**
- **Found during:** Task 3 (final workspace-wide rollout gate)
- **Issue:** BCE's query-command smoke depended on the caller environment's default database state.
- **Fix:** Pointed the smoke to a generated missing temporary database path and asserted the expected failure path deterministically.
- **Files modified:** `crates/bsky-comment-extractor/src/main.rs`
- **Verification:** `cargo test --workspace --verbose`
- **Committed in:** pending local Task 3 commit

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** No scope expansion. Both fixes were required to make the planned final gate deterministic.

## Issues Encountered
- `cargo test -p tftio-cli-common --lib --verbose` initially failed only in the full parallel test run because env-token tests in separate modules raced on process-global environment variables.
- The shared `cli-common` library emits an existing `dead_code` warning for `render_standard_completion_for_command`; it did not block the rollout gate.

## Manual Inspectability Review

Performed during execution.

Commands run:
```bash
TFTIO_AGENT_TOKEN=phase7-test-token TFTIO_AGENT_TOKEN_EXPECTED=phase7-test-token cargo run -q -p tftio-prompter -- --agent-help
TFTIO_AGENT_TOKEN=phase7-test-token TFTIO_AGENT_TOKEN_EXPECTED=phase7-test-token cargo run -q -p tftio-prompter -- --agent-skill render-prompts
```

Observed result:
- `--agent-help` listed only `render-prompts`, `list-profiles`, `tree-profiles`, and `validate-profiles`.
- `--agent-skill render-prompts` rendered only the `run` contract and did not reveal hidden commands.

## Verification

- `cargo test -p tftio-prompter agent_surface`
- `just cli-consistency`
- `bash tests/cli/06-agent-mode.sh`
- `cargo test -p tftio-cli-common --lib --verbose`
- `cargo test --workspace --verbose`

## User Setup Required

None.

## Next Phase Readiness
- Phase 7 rollout requirements are green at package, repo-smoke, and full-workspace levels.
- The shared restricted surface is now wired across all current `cli-common` consumers, including `prompter`.

## Self-Check: PASSED

- Verified `cargo test --workspace --verbose` exits 0.
- Verified `just cli-consistency` and `bash tests/cli/06-agent-mode.sh` exit 0.
- Verified the manual inspectability review commands ran and produced redacted agent-facing output.

---
*Phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools*
*Completed: 2026-03-23*
