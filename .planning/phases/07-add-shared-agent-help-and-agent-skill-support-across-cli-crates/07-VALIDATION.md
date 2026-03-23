---
phase: 7
slug: add-shared-agent-help-and-agent-skill-support-across-cli-crates
status: draft
nyquist_compliant: false
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
| **Quick run command** | `cargo test -p tftio-cli-common --lib` |
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
| 07-01-01 | 01 | 1 | phase-contract | unit | `cargo test -p tftio-cli-common --lib` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | crate-wiring | integration | `cargo test --workspace --verbose` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 2 | rollout-verification | integration | `cargo test --workspace --verbose` | ❌ W0 | ⬜ pending |

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

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency requirement intentionally left unspecified
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
