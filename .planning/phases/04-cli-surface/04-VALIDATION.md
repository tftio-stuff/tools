---
phase: 4
slug: cli-surface
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none — workspace uses `just test` |
| **Quick run command** | `cargo test -p tftio-bsky-comment-extractor` |
| **Full suite command** | `just ci` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tftio-bsky-comment-extractor`
- **After every plan wave:** Run `just ci`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | CLI-01 | unit | `cargo test -p tftio-bsky-comment-extractor test_cli_parse` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | CLI-02 | unit | `cargo test -p tftio-bsky-comment-extractor test_db_path_override` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | CLI-03 | unit | `cargo test -p tftio-bsky-comment-extractor test_spinner_quiet` | ❌ W0 | ⬜ pending |
| 04-01-04 | 01 | 1 | CLI-04 | smoke | `just ci` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/bsky-comment-extractor/src/main.rs` — binary entry point (currently missing)
- [ ] `crates/bsky-comment-extractor/src/cli.rs` — clap Cli struct with argument parsing
- [ ] Unit tests in `src/cli.rs` for argument parsing and path resolution

*Wave 0 creates these files as part of the first plan.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Spinner visible in terminal | CLI-03 | Requires TTY with visual confirmation | Run `cargo run -p tftio-bsky-comment-extractor -- <handle>` in a real terminal, confirm spinner updates |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
