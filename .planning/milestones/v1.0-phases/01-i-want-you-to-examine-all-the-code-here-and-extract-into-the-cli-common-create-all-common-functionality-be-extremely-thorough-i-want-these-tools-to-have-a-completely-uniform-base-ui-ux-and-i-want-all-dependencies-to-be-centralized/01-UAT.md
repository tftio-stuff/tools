---
status: passed
phase: 01-cli-common-unification
source:
  - 01-01-SUMMARY.md
  - 01-02-SUMMARY.md
  - 01-03-SUMMARY.md
  - 01-04-SUMMARY.md
started: 2026-03-22T20:30:00Z
updated: 2026-03-22T21:05:00Z
---

## Current Test

number: 5
name: Workspace CLI consistency enforcement catches drift
expected: |
  Running `just cli-consistency` should pass and verify that the workspace binaries
  share the agreed base help and metadata behavior instead of silently drifting apart again.
awaiting: complete

## Tests

### 1. Shared metadata commands are uniform across tools
expected: Running the base metadata commands on the migrated tools should now behave consistently. For example, `todoer meta version --json`, `gator meta version --json`, and `silent-critic meta version --json` should all succeed and return JSON instead of ad-hoc text. User-facing tools should also expose their shared metadata commands without breaking their primary invocation paths.
result: pass

### 2. Machine-readable CLIs share one JSON envelope and error contract
expected: Commands such as `todoer list --all --json`, `gator claude --session abc --no-yolo --json`, and `silent-critic --json ...` should emit the same top-level JSON shape for success and failure, including shared `ok` and `command` fields where applicable.
result: pass

### 3. User-facing CLIs keep their original workflows while sharing the base UX
expected: `bce alice.bsky.social` should still parse as extraction mode, `unvenv` should still default to its scan flow, and `asana-cli` should still keep its domain-specific command tree while now sharing the common version/license/completions/doctor/update behavior.
result: pass

### 4. Prompter still preserves its custom behavior on top of the shared base
expected: `prompter --help`, `prompter version --json`, `prompter doctor --json`, and `prompter completions bash` should all work. JSON version/doctor output should remain valid, and completions should still include prompter-specific augmentation.
result: pass

### 5. Workspace CLI consistency enforcement catches drift
expected: Running `just cli-consistency` should pass and verify that the workspace binaries share the agreed base help and metadata behavior instead of silently drifting apart again.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

None yet.
