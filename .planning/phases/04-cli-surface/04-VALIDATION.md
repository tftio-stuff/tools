---
phase: 4
slug: cli-surface
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 4 -- Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none -- workspace uses `just test` |
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
| 04-02-01 | 02 | 2 | CLI-01 | unit | `cargo test -p tftio-bsky-comment-extractor test_cli_parse` | W0 (Task 1 creates) | pending |
| 04-02-02 | 02 | 2 | CLI-02 | unit | `cargo test -p tftio-bsky-comment-extractor test_db_path_default` | W0 (Task 2 creates) | pending |
| 04-02-03 | 02 | 2 | CLI-03 | unit | `cargo test -p tftio-bsky-comment-extractor test_make_spinner_quiet` | W0 (Task 2 creates) | pending |
| 04-02-04 | 02 | 2 | CLI-04 | smoke | `just ci` | N/A | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `crates/bsky-comment-extractor/src/main.rs` -- binary entry point (currently missing)
- [ ] `crates/bsky-comment-extractor/src/cli.rs` -- clap Cli struct with argument parsing
- [ ] Unit tests in `src/cli.rs` for argument parsing (test_cli_parse_*)
- [ ] Unit tests in `src/main.rs` for path resolution (test_db_path_default) and spinner suppression (test_make_spinner_quiet)

*Wave 0 creates these files as part of Plan 02, Tasks 1 and 2.*

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
