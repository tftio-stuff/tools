---
phase: 07
slug: workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
status: planned
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-23
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo workspace unit + integration tests |
| **Config file** | none — Cargo defaults plus `justfile` recipes |
| **Quick run command** | `cargo test -p tftio-cli-common --lib` |
| **Full suite command** | `cargo test --workspace --verbose` |
| **Estimated runtime** | ~90 seconds |

---

## Sampling Rate

- **After every task commit:** Run the task-local automated verify from the map below
- **After every plan wave:** Run `cargo test --workspace --verbose`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds for task-level feedback, ~90 seconds for end-of-wave / final gating

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | D-01..D-03 | unit | `cargo test -p tftio-cli-common agent_mode_activation --lib` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | D-10..D-12 | unit | `cargo test -p tftio-cli-common capability_policy --lib` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 2 | D-04..D-06 | unit | `cargo test -p tftio-cli-common agent_surface_redaction --lib` | ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 2 | D-07..D-09 | unit | `cargo test -p tftio-cli-common agent_help_render --lib` | ❌ W0 | ⬜ pending |
| 07-02-03 | 02 | 2 | D-04..D-06 | unit | `cargo test -p tftio-cli-common agent_surface_redaction --lib` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 3 | D-04, D-07, D-08, D-09, D-14 | package smoke | `cargo test -p tftio-bsky-comment-extractor agent_surface` | ❌ W0 | ⬜ pending |
| 07-03-02 | 03 | 3 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-gator agent_surface` | ❌ W0 | ⬜ pending |
| 07-03-03 | 03 | 3 | D-13..D-14 | package smoke | `cargo test -p tftio-bsky-comment-extractor agent_surface && cargo test -p tftio-gator agent_surface` | ✅ | ⬜ pending |
| 07-04-01 | 04 | 3 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-todoer agent_surface` | ❌ W0 | ⬜ pending |
| 07-04-02 | 04 | 3 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-unvenv agent_surface` | ❌ W0 | ⬜ pending |
| 07-04-03 | 04 | 3 | D-13 | package smoke | `cargo test -p tftio-todoer agent_surface && cargo test -p tftio-unvenv agent_surface` | ✅ | ⬜ pending |
| 07-05-01 | 05 | 3 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-asana-cli agent_surface` | ❌ W0 | ⬜ pending |
| 07-05-02 | 05 | 3 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-silent-critic agent_surface` | ❌ W0 | ⬜ pending |
| 07-05-03 | 05 | 3 | D-13 | package smoke | `cargo test -p tftio-asana-cli agent_surface && cargo test -p tftio-silent-critic agent_surface` | ✅ | ⬜ pending |
| 07-06-01 | 06 | 4 | D-04..D-09, D-13 | package smoke | `cargo test -p tftio-prompter agent_surface` | ❌ W0 | ⬜ pending |
| 07-06-02 | 06 | 4 | D-04..D-09, D-13 | repo smoke | `just cli-consistency` | ❌ W0 | ⬜ pending |
| 07-06-03 | 06 | 4 | D-13 | repo smoke + final handoff | `bash tests/cli/06-agent-mode.sh` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave Layout

| Wave | Plans | Primary validation |
|------|-------|--------------------|
| 1 | 07-01 | `cargo test -p tftio-cli-common agent_mode_activation --lib`, `cargo test -p tftio-cli-common capability_policy --lib` |
| 2 | 07-02 | `cargo test -p tftio-cli-common agent_surface_redaction --lib`, `cargo test -p tftio-cli-common agent_help_render --lib` |
| 3 | 07-03, 07-04, 07-05 | package-level `agent_surface` suites for each migrated consumer pair |
| 4 | 07-06 | `cargo test -p tftio-prompter agent_surface`, `just cli-consistency`, `bash tests/cli/06-agent-mode.sh`, then final gate `cargo test --workspace --verbose` |

### Final Gate

- `cargo test --workspace --verbose` remains the end-of-wave / end-of-phase rollout blocker after all task-level and repo-smoke verifies are green.

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/agent.rs` — shared agent-mode substrate and matching unit tests
- [ ] `crates/cli-common/src/completions.rs` — filtered-command completion tests
- [ ] `crates/*/tests/` or crate-local test modules — one smoke test per consumer for `--agent-help`, `--agent-skill`, hidden-command rejection, and redaction cases where required
- [ ] `justfile` + `tests/cli/06-agent-mode.sh` — `cli-consistency` executes the repo smoke script in-repo
- [ ] `crates/bsky-comment-extractor/src/main.rs` executable entrypoint/smoke viability on this branch

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent help text is actually inspectable and useful to an autonomous agent | D-07..D-09 | Structured text quality and workflow readability are not fully captured by unit assertions | Run each migrated tool with agent mode active, inspect `--agent-help` and at least one `--agent-skill <name>` output, and confirm the visible contract is complete without revealing hidden commands |
| Final rollout summary records manual inspectability handoff | D-13 | Execution needs an explicit operator handoff even if the spot check is deferred | In Plan 07-06 summary, record the exact commands used for spot-check review and whether the review was performed or remains pending operator sign-off |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [x] Fast task-level feedback stays at or under ~30s where practical; slower workspace suite reserved for final gating
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
