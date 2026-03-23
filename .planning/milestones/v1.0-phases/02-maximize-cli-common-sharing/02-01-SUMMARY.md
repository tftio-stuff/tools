---
phase: 02-maximize-cli-common-sharing
plan: 01
subsystem: cli-common
tags: [rust, clap, cli-common, doctor, completions]
requires: []
provides:
  - shared workspace tool preset in `cli-common`
  - shared doctorless adapter and no-doctor dispatcher
  - buffer-first completion rendering
  - structured doctor report rendering and shared response rendering
key-files:
  modified: [crates/cli-common/src/app.rs, crates/cli-common/src/command.rs, crates/cli-common/src/completions.rs, crates/cli-common/src/doctor.rs, crates/cli-common/src/json.rs, crates/cli-common/src/lib.rs]
completed: 2026-03-22
---

# Phase 2 Plan 01 Summary

Expanded `tftio-cli-common` so later migrations could delete more entrypoint boilerplate instead of wrapping it.

## Accomplishments
- Added `workspace_tool` and `ToolSpec::workspace` for the shared tools workspace preset.
- Added the shared `NoDoctor` adapter and `run_standard_command_no_doctor`.
- Added `CompletionOutput`, `render_completion`, `render_completion_instructions`, and `write_completion` for buffer-first completion generation.
- Added `DoctorReport` plus shared plain-text and JSON doctor rendering helpers.
- Added `render_response` for the shared JSON-vs-text success contract.

## Verification
- `cargo test -p tftio-cli-common`
- `cargo clippy -p tftio-cli-common -- -D warnings`

## Notes
- No commits were created.
