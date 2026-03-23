---
phase: 2
slug: maximize-cli-common-sharing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test`, `cargo clippy`, `just`, repository shell CLI suite |
| **Config file** | `Cargo.toml`, `justfile`, `tests/cli/*.sh` |
| **Quick run command** | `cargo test -p tftio-cli-common && cargo clippy -p tftio-cli-common -- -D warnings` |
| **Full suite command** | `just cli-metadata-consistency && just cli-consistency && just test && just lint` |
| **Estimated runtime** | not estimated |

---

## Sampling Rate

- **After every task commit:** Run the task-specific focused `cargo test -p <crate>` or shell command listed in the plan
- **After every plan wave:** Run the full suite command listed above
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** not estimated

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | CLI-SHARE-01 | unit | `cargo test -p tftio-cli-common && cargo clippy -p tftio-cli-common -- -D warnings` | ✅ | ⬜ pending |
| 2-02-01 | 02 | 2 | CLI-SHARE-03 | crate | `cargo test -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor && cargo clippy -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor -- -D warnings` | ✅ | ⬜ pending |
| 2-03-01 | 03 | 3 | CLI-SHARE-02 | crate | `cargo test -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter && cargo clippy -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter -- -D warnings` | ✅ | ⬜ pending |
| 2-04-01 | 04 | 4 | CLI-SHARE-04 | integration | `just cli-metadata-consistency && just cli-consistency && just test && just lint` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/completions.rs` — buffer-oriented completion rendering tests exist before `prompter` migration
- [ ] `crates/cli-common/src/doctor.rs` — structured doctor report / JSON rendering tests exist before doctor helper migrations
- [ ] `crates/cli-common/src/app.rs` or `crates/cli-common/src/command.rs` — helper tests exist for shared tool-spec/meta-command boilerplate extraction
- [ ] `tests/cli/*.sh` — shell suite covers the newly shared command/doctor/completion contracts before repository cleanup lands

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Help/readability after boilerplate extraction | CLI-SHARE-03 | Shell tests catch presence, not human readability | Spot-check `--help` for `gator`, `todoer`, `unvenv`, `asana-cli`, and `prompter` after the final migration wave |
| Completion augmentation readability | CLI-SHARE-02 | Script content can be asserted automatically, but human install guidance quality is still visual | Inspect the first lines of `prompter completions bash` and `asana-cli completions bash` after extraction |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all shared-surface additions before consumer migrations
- [ ] No watch-mode flags
- [ ] Feedback latency documented without runtime estimates
- [ ] `nyquist_compliant: true` set in frontmatter after execution validation

**Approval:** pending
