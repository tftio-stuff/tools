---
status: passed
phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding
source:
  - .planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-01-SUMMARY.md
  - .planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-02-SUMMARY.md
  - .planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-03-SUMMARY.md
  - .planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-04-SUMMARY.md
started: 2026-03-22T23:41:00Z
updated: 2026-03-22T23:46:00Z
mode: automated
---

## Current Test

number: complete
name: Automated verification complete
expected: |
  Phase 03 verification is satisfied by repository-level automated evidence rather than manual interaction.
awaiting: none

## Tests

### 1. Shared metadata mapping and fatal runner helpers preserve thin-tool command behavior
expected: `gator`, `bce`, and `todoer` should still expose their metadata and primary command flows correctly after moving entrypoint glue onto the shared `cli-common` helper layer. Representative metadata commands, fatal-path handling, and thin-tool command parsing should continue to work without local boilerplate regressions.
result: pass
evidence:
  - `cargo test -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer`
  - `cargo clippy -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer -- -D warnings`
  - `just cli-metadata-consistency`

### 2. Shared JSON and text response emission preserve machine-readable contracts
expected: Tools that now rely on the richer shared response emitters should still return the same observable JSON envelope and text output behavior. Commands exercised by the shell suite and crate tests should continue to expose shared `ok` and `command` fields for JSON mode while keeping tool-specific text summaries intact.
result: pass
evidence:
  - `cargo test -p tftio-cli-common`
  - `cargo clippy -p tftio-cli-common -- -D warnings`
  - `cargo test -p tftio-silent-critic`
  - `cargo clippy -p tftio-silent-critic -- -D warnings`
  - `just cli-consistency`

### 3. Shared doctor scaffolding and fatal display handlers preserve richer-tool behavior
expected: `prompter doctor --json`, `bce doctor`, `unvenv doctor`, and `asana-cli` top-level fatal handling should still behave correctly after moving doctor-provider scaffolding and fatal-runner code into `cli-common`. Tool-specific state collection and tracing should remain intact while shared report/exit semantics stay uniform.
result: pass
evidence:
  - `cargo test -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter`
  - `cargo clippy -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter -- -D warnings`
  - `just cli-metadata-consistency`
  - `just cli-consistency`

### 4. Repository enforcement covers the final cli-common boundary
expected: The repository should now automatically enforce the thinner final boundary: deleted metadata/runner/response/doctor glue patterns stay deleted, the documented `cli-common` boundary matches the implementation, and the full test/lint suite remains green.
result: pass
evidence:
  - `just cli-metadata-consistency`
  - `just cli-consistency`
  - `just test`
  - `just lint`

## Summary

total: 4
passed: 4
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None.
