---
phase: 1
slug: cli-common-unification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test`, `cargo clippy`, `just` |
| **Config file** | `Cargo.toml`, `justfile`, workspace crate test suites |
| **Quick run command** | `cargo test -p tftio-cli-common && cargo test -p tftio-gator && cargo test -p tftio-todoer` |
| **Full suite command** | `just test && just lint` |
| **Estimated runtime** | not estimated |

---

## Sampling Rate

- **After every task commit:** Run the task-specific focused `cargo test -p <crate>` command listed in the plan.
- **After every plan wave:** Run `just test && just lint`.
- **Before `$gsd-verify-work`:** Run `just ci`.
- **Max feedback latency:** not estimated

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 1 | CLI-UNIFY-01 | unit | `cargo test -p tftio-cli-common` | ✅ | ⬜ pending |
| 1-02-01 | 02 | 2 | CLI-UNIFY-02 | crate | `cargo test -p tftio-gator && cargo test -p tftio-todoer && cargo test -p tftio-silent-critic` | ✅ | ⬜ pending |
| 1-03-01 | 03 | 2 | CLI-UNIFY-03 | crate | `cargo test -p tftio-unvenv && cargo test -p tftio-bsky-comment-extractor && cargo test -p tftio-asana-cli` | ✅ | ⬜ pending |
| 1-04-01 | 04 | 3 | CLI-UNIFY-04 | integration | `cargo test -p tftio-prompter && just test && just lint` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/json.rs` — shared JSON envelope tests exist before consumer migrations land
- [ ] `crates/cli-common/src/progress.rs` — spinner contract tests exist before `bsky-comment-extractor` / `unvenv` migration
- [ ] `crates/cli-common/src/command.rs` — metadata-command tests exist before consumer rewiring

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Help output parity across binaries | CLI-UNIFY-04 | Clap help layout is user-facing and hard to assert comprehensively with grep alone | Run each binary with `--help`; confirm shared metadata-command vocabulary and no obviously divergent headings |
| Completion instruction wrappers still make sense | CLI-UNIFY-04 | `prompter` and `asana-cli` have custom completion behavior | Run `prompter completions bash`, `asana-cli completions bash`, and `unvenv completions bash`; confirm instruction header and script output are both present |
| JSON envelope compatibility | CLI-UNIFY-02 | Machine consumers depend on shape, not just success code | Exercise one failing and one successful JSON path in `gator`, `todoer`, and `silent-critic`; confirm exact `ok/command/data` and `ok/error` shape |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all shared-surface references
- [ ] No watch-mode flags
- [ ] Feedback latency documented without runtime estimate claims
- [ ] `nyquist_compliant: true` set in frontmatter after execution validation

**Approval:** pending
