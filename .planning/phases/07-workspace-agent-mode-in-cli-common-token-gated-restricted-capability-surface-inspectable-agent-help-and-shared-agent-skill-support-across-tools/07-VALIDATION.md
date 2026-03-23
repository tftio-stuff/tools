---
phase: 07
slug: workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
status: draft
nyquist_compliant: false
wave_0_complete: false
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

- **After every task commit:** Run `cargo test -p tftio-cli-common --lib`
- **After every plan wave:** Run `cargo test --workspace --verbose`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | D-01..D-03 | unit | `cargo test -p tftio-cli-common agent_mode_activation --lib` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | D-04..D-06 | unit + integration | `cargo test -p tftio-cli-common agent_surface_redaction --lib` | ❌ W0 | ⬜ pending |
| 07-01-03 | 01 | 1 | D-07..D-09 | unit | `cargo test -p tftio-cli-common agent_help_render --lib` | ❌ W0 | ⬜ pending |
| 07-01-04 | 01 | 1 | D-10..D-12 | unit | `cargo test -p tftio-cli-common capability_policy --lib` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 2 | D-13..D-14 | integration / smoke | `cargo test --workspace --verbose` | ✅ | ⬜ pending |
| 07-02-02 | 02 | 2 | D-13..D-14 | integration / smoke | `just cli-consistency` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/agent.rs` — shared agent-mode substrate and matching unit tests
- [ ] `crates/cli-common/src/completions.rs` — filtered-command completion tests
- [ ] `crates/*/tests/` or crate-local test modules — one smoke test per consumer for `--agent-help`, `--agent-skill`, and hidden-command rejection
- [ ] `scripts/test-cli-metadata-consistency.sh` or equivalent replacement — restore `just cli-consistency`
- [ ] `crates/bsky-comment-extractor/src/main.rs` executable entrypoint/smoke viability on this branch

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent help text is actually inspectable and useful to an autonomous agent | D-07..D-09 | Structured text quality and workflow readability are not fully captured by unit assertions | Run each migrated tool with agent mode active, inspect `--agent-help` and at least one `--agent-skill <name>` output, and confirm the visible contract is complete without revealing hidden commands |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
