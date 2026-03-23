---
phase: 07-add-shared-agent-help-and-agent-skill-support-across-cli-crates
plan: 03
subsystem: cli
tags: [rust, clap, agent-help, agent-skill, sqlite]
requires:
  - phase: 07-01
    provides: shared agent-doc rendering, raw-argv detection, and clap coverage helpers
provides:
  - exhaustive todoer agent-help and agent-skill output
  - exhaustive silent-critic agent-help and agent-skill output
  - subprocess regressions for hidden top-level-only agent-doc behavior
affects: [todoer, silent-critic, phase-07-validation]
tech-stack:
  added: [tftio-cli-common]
  patterns: [top-level raw-argv agent-doc interception, clap coverage validation, subprocess help regressions]
key-files:
  created:
    - crates/todoer/tests/agent_help.rs
    - crates/silent-critic/tests/agent_help.rs
  modified:
    - crates/todoer/Cargo.toml
    - crates/todoer/src/cli.rs
    - crates/todoer/src/main.rs
    - crates/silent-critic/src/cli.rs
    - crates/silent-critic/src/main.rs
    - Cargo.lock
key-decisions:
  - "todoer now authors one AgentDoc that explains project resolution, JSON envelopes, stdin-driven note input, and SQLite-backed task flows."
  - "silent-critic now authors one AgentDoc that explains the session state machine, worker token requirements, config/db paths, and export-format exceptions."
  - "todoer gained a direct workspace dependency on tftio-cli-common so both stateful crates use the shared raw-argv detector and renderers."
patterns-established:
  - "Stateful crates can preserve required subcommand parsing by intercepting only exact top-level `--agent-help` and `--agent-skill` argv shapes before clap."
  - "Exhaustive authored docs stay aligned with nested clap trees by pairing integration assertions with `assert_command_coverage` and `assert_argument_coverage` unit tests."
requirements-completed: [ADOC-02, ADOC-03, ADOC-04, ADOC-05]
duration: "6m 40s"
completed: "2026-03-23T01:07:45Z"
---

# Phase 07 Plan 03: Stateful crate agent docs summary

Shared agent-doc plumbing now covers the two SQLite-backed crates with exhaustive per-tool docs, top-level-only dispatch, and regression tests for nested command trees.

## Summary

- Added a `todoer`-specific `AgentDoc` covering `.todoer.toml`, project discovery order, JSON envelope shapes, stdin-driven `new` and `task note` input, and the full `task update status` flow.
- Added a `silent-critic`-specific `AgentDoc` covering project/criterion/session/contract/decide/log commands, repo-hash database resolution, `SILENT_CRITIC_TOKEN`, the session state machine, and JSON/Markdown export behavior.
- Short-circuited both binaries on exact top-level `--agent-help` and `--agent-skill` requests before clap parsing so required subcommands still behave normally.
- Added subprocess regressions that prove top-level success, hidden help behavior, and rejection of subcommand-placed agent-doc flags.

## Task Outcomes

### Task 1: Add exhaustive agent-doc output to todoer and cover JSON/project behavior

- **RED:** `cbf9379` — added failing subprocess coverage for top-level `todoer --agent-help` / `--agent-skill`, hidden help, and subcommand placement rejection.
- **GREEN:** `35d9661` — implemented authored `todoer` docs and top-level raw-argv interception; added clap coverage tests for nested command paths and arguments.
- **Supporting commit:** `153a13e` — recorded the lockfile change from adding the workspace `tftio-cli-common` dependency to `todoer`.

### Task 2: Add agent-doc support to silent-critic for the full session/criterion command tree

- **RED:** `d6eb29f` — added failing subprocess coverage for top-level `silent-critic --agent-help` / `--agent-skill`, hidden help, and subcommand placement rejection.
- **GREEN:** `73f7c15` — implemented authored `silent-critic` docs and top-level raw-argv interception; added clap coverage tests for the nested project, criterion, session, contract, decide, and log command tree.

## Verification

- `cargo test -p tftio-todoer agent_help`
- `cargo test -p tftio-todoer agent_doc_covers_todoer`
- `cargo test -p tftio-silent-critic agent_help`
- `cargo test -p tftio-silent-critic agent_doc_covers_silent_critic`
- `cargo test -p tftio-todoer agent_help && cargo test -p tftio-silent-critic agent_help`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the shared cli-common dependency to todoer**
- **Found during:** Task 1
- **Issue:** `todoer` did not depend on `tftio-cli-common`, so the shared raw-argv detector and agent-doc renderers were unavailable from the plan's listed files alone.
- **Fix:** Added `tftio-cli-common.workspace = true` to `crates/todoer/Cargo.toml` and updated `Cargo.lock`.
- **Files modified:** `crates/todoer/Cargo.toml`, `Cargo.lock`
- **Verification:** `cargo test -p tftio-todoer agent_help` and `cargo test -p tftio-todoer agent_doc_covers_todoer`
- **Commit:** `35d9661`, `153a13e`

## Authentication Gates

None.

## Issues Encountered

None.

## Self-Check: PASSED

- Verified `.planning/phases/07-add-shared-agent-help-and-agent-skill-support-across-cli-crates/07-03-SUMMARY.md` exists.
- Verified task commits `cbf9379`, `35d9661`, `d6eb29f`, `73f7c15`, and `153a13e` exist in git history.
