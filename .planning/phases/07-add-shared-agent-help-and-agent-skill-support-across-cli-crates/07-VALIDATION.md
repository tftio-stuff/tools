---
phase: 7
slug: add-shared-agent-help-and-agent-skill-support-across-cli-crates
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-22
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | `Cargo.toml`, `justfile` |
| **Quick run command** | `cargo test -p tftio-cli-common agent_docs --lib` |
| **Full suite command** | `cargo test --workspace --verbose` |
| **Estimated runtime** | not specified |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tftio-cli-common --lib` or the narrowest affected crate test command introduced by the task
- **After every plan wave:** Run `cargo test --workspace --verbose`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** not specified

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | ADOC-01, ADOC-03, ADOC-04 | unit | `cargo test -p tftio-cli-common agent_docs --lib` | `crates/cli-common/src/lib.rs` exists; `crates/cli-common/src/agent_docs.rs` = ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | ADOC-01, ADOC-03, ADOC-04 | unit | `cargo test -p tftio-cli-common agent_docs --lib` | `crates/cli-common/src/agent_docs.rs` = ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 2 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-bsky-comment-extractor agent_help && cargo test -p tftio-unvenv agent_help` | `crates/bsky-comment-extractor/tests/agent_help.rs` = ❌ W0; `crates/unvenv/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-02-02 | 02 | 2 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-gator agent_help` | `crates/gator/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 2 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-todoer agent_help` | `crates/todoer/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-03-02 | 03 | 2 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-silent-critic agent_help` | `crates/silent-critic/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-04-01 | 04 | 3 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-prompter agent_help` | `crates/prompter/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-04-02 | 04 | 3 | ADOC-02, ADOC-03, ADOC-04, ADOC-05 | integration | `cargo test -p tftio-asana-cli agent_help` | `crates/asana-cli/tests/agent_help.rs` = ❌ W0 | ⬜ pending |
| 07-04-03 | 04 | 3 | ADOC-05 | integration | `cargo test --workspace agent_help && cargo test --workspace --verbose` | All seven `tests/agent_help.rs` files = ❌ W0 until rollout plans land | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/agent_docs.rs` — shared validation surface and renderer tests
- [ ] `crates/cli-common/tests/agent_docs.rs` or equivalent unit coverage — canonical YAML and skill rendering assertions
- [ ] per-crate top-level invocation tests for all seven binaries — success on `--agent-help` and `--agent-skill`, hidden from standard help, reject subcommand placement

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Exhaustiveness and operator usefulness of generated docs | phase-contract | Automated tests can confirm structure and key strings but not whether the document is complete enough for real agent use | Run each binary with `--agent-help` and `--agent-skill`; inspect that commands, flags, examples, env/config, output shapes, and failure guidance are complete and tool-specific |

---

## Validation Sign-Off

- [x] All 9 planned tasks have matching automated verification entries or explicit Wave 0 dependencies
- [x] Sampling continuity: every task has an automated command, so there are no 3-task gaps
- [x] Wave 0 coverage lists the missing shared module and all seven crate-level `agent_help` test files
- [x] No watch-mode flags appear in any plan verification command
- [x] Feedback latency requirement remains intentionally unspecified
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** ready for execution
