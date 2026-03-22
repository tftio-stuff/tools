---
phase: 5
slug: query-subcommand
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-22
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | none |
| **Quick run command** | `cargo test -p tftio-bsky-comment-extractor --verbose` |
| **Full suite command** | `cargo test --workspace --verbose` |
| **Estimated runtime** | unknown — measure during execution |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tftio-bsky-comment-extractor --verbose`
- **After every plan wave:** Run `cargo test --workspace --verbose`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** unknown — measure during execution

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | QUERY-01 | integration | `cargo test -p tftio-bsky-comment-extractor query_outputs_jsonl --verbose` | ❌ W0 | ⬜ pending |
| 5-01-02 | 01 | 1 | QUERY-02 | integration | `cargo test -p tftio-bsky-comment-extractor query_limit_controls_page_size --verbose` | ❌ W0 | ⬜ pending |
| 5-01-03 | 01 | 1 | QUERY-03 | integration | `cargo test -p tftio-bsky-comment-extractor query_offset_skips_rows --verbose` | ❌ W0 | ⬜ pending |
| 5-01-04 | 01 | 1 | QUERY-04 | integration | `cargo test -p tftio-bsky-comment-extractor query_db_override_and_missing_db --verbose` | ❌ W0 | ⬜ pending |
| 5-01-05 | 01 | 1 | AGENT-02 | integration | `cargo test -p tftio-bsky-comment-extractor query_envelope_metadata --verbose` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/bsky-comment-extractor/tests/query_cli.rs` — CLI stdout/stderr contract for QUERY-01, QUERY-02, QUERY-03, QUERY-04, and AGENT-02
- [ ] `crates/bsky-comment-extractor/src/db.rs` — query-page unit tests for deterministic sort, limit, offset, and empty DB behavior
- [ ] `crates/bsky-comment-extractor/src/cli.rs` — parser tests for subcommand migration, top-level `--agent-help`, and defaults

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Concurrent query against a WAL-backed database while fetch has recently written | QUERY-04 | The phase research calls out WAL/`SQLITE_BUSY` behavior as an edge case that may require real-file reproduction beyond standard unit coverage | Populate a real database with `bce fetch`, then run `bce query --db <path>` during or immediately after another write-oriented session and confirm errors stay structured JSON on stderr |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency recorded during execution
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
