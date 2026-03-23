---
phase: 02-maximize-cli-common-sharing
plan: 02
subsystem: workspace-clis
tags: [rust, gator, todoer, silent-critic, bce]
requires: [02-01]
provides:
  - low-risk tools use shared workspace tool metadata
  - local doctorless boilerplate removed
  - shared response rendering adopted where useful
key-files:
  modified: [crates/gator/src/main.rs, crates/todoer/src/main.rs, crates/silent-critic/src/main.rs, crates/bsky-comment-extractor/src/main.rs]
completed: 2026-03-22
---

# Phase 2 Plan 02 Summary

Migrated the lower-risk CLI entrypoints onto the richer `cli-common` surface.

## Accomplishments
- Removed local `NoDoctor` and `tool_spec()` boilerplate from `gator`, `todoer`, `silent-critic`, and `bce`.
- Replaced those with shared workspace tool presets and the shared doctorless command adapter.
- Moved part of the repeated JSON-vs-text success handling in `todoer` and `silent-critic` onto `render_response`.

## Verification
- `cargo test -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor`
- `cargo clippy -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor -- -D warnings`
- `just cli-metadata-consistency`

## Notes
- No commits were created.
