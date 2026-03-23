---
status: passed
phase: 02-maximize-cli-common-sharing
source:
  - .planning/phases/02-maximize-cli-common-sharing/02-01-SUMMARY.md
  - .planning/phases/02-maximize-cli-common-sharing/02-02-SUMMARY.md
  - .planning/phases/02-maximize-cli-common-sharing/02-03-SUMMARY.md
  - .planning/phases/02-maximize-cli-common-sharing/02-04-SUMMARY.md
started: 2026-03-22T23:05:00Z
updated: 2026-03-22T23:12:00Z
mode: automated
---

## Current Test

number: complete
name: Automated verification complete
expected: |
  Phase 02 verification is satisfied by repository-level automated evidence rather than manual interaction.
awaiting: none

## Tests

### 1. Shared metadata adapters replaced local boilerplate
expected: Representative tools now expose their shared metadata commands through the cli-common adapter layer rather than local `tool_spec()` / `NoDoctor` glue. In practice, commands like `gator --json meta version`, `todoer meta version --json`, `silent-critic --json meta version`, and `bce doctor` still behave correctly, and the repository shell suite's new `05-shared-boilerplate.sh` check passes because those local boilerplate patterns stay deleted.
result: pass
evidence:
  - `just cli-metadata-consistency`
  - `sh tests/cli/05-shared-boilerplate.sh`

### 2. Shared JSON envelope and response rendering still hold across machine-oriented tools
expected: Commands such as `todoer list --all --json`, `gator --json ...`, and `silent-critic --json ...` still return the shared JSON envelope with consistent top-level fields, even though more of the success-path response rendering now goes through cli-common helpers.
result: pass
evidence:
  - `cargo test -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor`
  - `cargo clippy -p tftio-gator -p tftio-todoer -p tftio-silent-critic -p tftio-bsky-comment-extractor -- -D warnings`
  - `just cli-consistency`

### 3. Richer tools kept their special behavior while moving generic rendering into cli-common
expected: `prompter doctor --json` still reports structured doctor state, `prompter completions bash` still includes dynamic profile helpers, and `unvenv` / `asana-cli` still expose their normal command trees and base metadata commands after switching to shared workspace tool presets.
result: pass
evidence:
  - `cargo test -p tftio-prompter -p tftio-unvenv -p tftio-asana-cli`
  - `cargo clippy -p tftio-prompter -p tftio-unvenv -p tftio-asana-cli -- -D warnings`
  - `just cli-metadata-consistency`

### 4. Repository enforcement covers the maximal-sharing boundary
expected: The repository now documents that shared CLI UX belongs in cli-common, and the shell suite plus `just cli-consistency` enforce that extracted boilerplate does not reappear.
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

## Gaps

None.
