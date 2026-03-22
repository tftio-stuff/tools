---
phase: 3
slug: extraction-engine
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
| **Framework** | Rust built-in test harness |
| **Config file** | none |
| **Quick run command** | `cargo test -p tftio-bsky-comment-extractor` |
| **Full suite command** | `cargo test -p tftio-bsky-comment-extractor` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p tftio-bsky-comment-extractor`
- **After every plan wave:** Run `cargo test -p tftio-bsky-comment-extractor`
- **Before `/gsd:verify-work`:** Full suite must be green + `cargo clippy -p tftio-bsky-comment-extractor`
- **Max feedback latency:** 10 seconds

---

## Per-Requirement Verification Map

| Requirement | Test Type | Automated Command | File Exists | Status |
|-------------|-----------|-------------------|-------------|--------|
| AUTH-01 | unit | `cargo test -p tftio-bsky-comment-extractor -- auth` | ❌ W0 | ⬜ pending |
| AUTH-02 | unit (mock) | `cargo test -p tftio-bsky-comment-extractor -- create_session` | ❌ W0 | ⬜ pending |
| EXTR-01 | unit | `cargo test -p tftio-bsky-comment-extractor -- list_records` | ❌ W0 | ⬜ pending |
| EXTR-02 | unit | `cargo test -p tftio-bsky-comment-extractor -- pagination` | ❌ W0 | ⬜ pending |
| EXTR-03 | unit (mock) | `cargo test -p tftio-bsky-comment-extractor -- resolve_handle` | ❌ W0 | ⬜ pending |
| EXTR-04 | unit | `cargo test -p tftio-bsky-comment-extractor -- backoff` | ❌ W0 | ⬜ pending |
| STOR-01 | unit | `cargo test -p tftio-bsky-comment-extractor -- schema` | ❌ W0 | ⬜ pending |
| STOR-02 | unit | `cargo test -p tftio-bsky-comment-extractor -- upsert_idempotent` | ❌ W0 | ⬜ pending |
| STOR-03 | unit | `cargo test -p tftio-bsky-comment-extractor -- db_path` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/bsky-comment-extractor/src/lib.rs` — crate entry point
- [ ] `crates/bsky-comment-extractor/src/error.rs` — `ExtractorError` type
- [ ] `crates/bsky-comment-extractor/Cargo.toml` — crate manifest with workspace deps

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Rate limit backoff against live API | EXTR-04 | Cannot reliably trigger 429 in tests | Manual: run against live API with tight loop, observe backoff in tracing output |

---

## Validation Sign-Off

- [ ] All tasks have automated verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
