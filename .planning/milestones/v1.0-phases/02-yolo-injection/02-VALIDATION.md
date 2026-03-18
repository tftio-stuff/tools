---
phase: 2
slug: yolo-injection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none -- inline `#[cfg(test)]` modules |
| **Quick run command** | `cargo test -p tftio-gator` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tftio-gator`
- **After every plan wave:** Run `just test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | PERM-01 | unit | `cargo test -p tftio-gator build_command_claude_yolo` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | PERM-01 | unit | `cargo test -p tftio-gator build_command_codex_yolo` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | PERM-01 | unit | `cargo test -p tftio-gator build_command_gemini_yolo_warn` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | PERM-01 | unit | `cargo test -p tftio-gator yolo_skipped_in_session` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | PERM-02 | unit | `cargo test -p tftio-gator parse_no_yolo` | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | PERM-02 | unit | `cargo test -p tftio-gator validate_no_yolo_with_session` | ❌ W0 | ⬜ pending |
| 02-01-07 | 01 | 1 | PERM-02 | unit | `cargo test -p tftio-gator build_command_no_yolo_skips_injection` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `--no-yolo` with Gemini suppresses warning | PERM-02 | Stderr capture in unit tests is complex; behavior follows from `inject_yolo=false` gate | Run `cargo run -p tftio-gator -- --agent gemini --no-yolo` and verify no warning printed |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
