---
phase: 3
slug: extract-remaining-cli-glue-into-cli-common
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 3 — Validation Strategy

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
| 3-01-01 | 01 | 1 | CLI-SHARE-05 | unit | `cargo test -p tftio-cli-common && cargo clippy -p tftio-cli-common -- -D warnings` | ✅ | ⬜ pending |
| 3-02-01 | 02 | 2 | CLI-SHARE-05 | crate | `cargo test -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer && cargo clippy -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer -- -D warnings` | ✅ | ⬜ pending |
| 3-03-01 | 03 | 3 | CLI-SHARE-06 | crate | `cargo test -p tftio-silent-critic -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter && cargo clippy -p tftio-silent-critic -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter -- -D warnings` | ✅ | ⬜ pending |
| 3-04-01 | 04 | 4 | CLI-SHARE-04 | integration | `just cli-metadata-consistency && just cli-consistency && just test && just lint` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/cli-common/src/command.rs` and the new runner/output modules have focused tests before consumer migrations depend on them
- [ ] `crates/cli-common/src/doctor.rs` has focused tests for the new provider/build helpers before provider migrations land
- [ ] the repository shell suite contains checks for the newly deleted metadata-mapping and wrapper patterns before the enforcement plan lands
- [ ] no plan introduces manual-only verification for behavior that can be exercised by building and running the tools locally

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Help readability after runner extraction | CLI-SHARE-05 | Automated tests can prove behavior and presence, not the readability of help text | Spot-check `--help` for `gator`, `bce`, `todoer`, `silent-critic`, `unvenv`, `asana-cli`, and `prompter` after the final wave |
| Doctor text readability | CLI-SHARE-05 | JSON/text contracts can be automated, but line wrapping/readability is still visual | Inspect representative `doctor` text output after the final migration wave |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all shared-surface additions before consumer migrations
- [ ] No watch-mode flags
- [ ] Feedback latency documented without runtime estimates
- [ ] `nyquist_compliant: true` set in frontmatter after execution validation

**Approval:** pending
