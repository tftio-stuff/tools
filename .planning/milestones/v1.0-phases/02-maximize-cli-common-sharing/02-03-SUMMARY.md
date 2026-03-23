---
phase: 02-maximize-cli-common-sharing
plan: 03
subsystem: richer-clis
tags: [rust, prompter, unvenv, asana-cli]
requires: [02-01, 02-02]
provides:
  - `prompter` uses shared completion rendering and doctor report helpers
  - `unvenv` and `asana-cli` use shared workspace tool presets
key-files:
  modified: [crates/prompter/src/completions.rs, crates/prompter/src/doctor.rs, crates/prompter/src/main.rs, crates/unvenv/src/main.rs, crates/asana-cli/src/cli/mod.rs]
completed: 2026-03-22
---

# Phase 2 Plan 03 Summary

Migrated the richer CLI consumers onto the new shared rendering surface.

## Accomplishments
- Refactored `prompter` completions to start from shared buffered completion output and keep only its dynamic augmentation locally.
- Refactored `prompter` JSON doctor output to render through `DoctorReport` instead of a parallel local formatter.
- Replaced local workspace tool metadata constructors in `unvenv` and `asana-cli` with the shared preset.

## Verification
- `cargo test -p tftio-prompter -p tftio-unvenv -p tftio-asana-cli`
- `cargo clippy -p tftio-prompter -p tftio-unvenv -p tftio-asana-cli -- -D warnings`

## Notes
- No commits were created.
