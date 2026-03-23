---
status: partial
phase: 07-add-shared-agent-help-and-agent-skill-support-across-cli-crates
source: [07-VERIFICATION.md]
started: 2026-03-23T01:35:51Z
updated: 2026-03-23T01:35:51Z
---

## Current Test

awaiting human testing

## Tests

### 1. Exhaustiveness review of canonical YAML output
expected: each binary's `--agent-help` output fully covers commands, flags, examples, env/config/defaults, output shapes, failure guidance, and operator mistakes
result: pending

### 2. Exhaustiveness review of Claude skill output
expected: each binary's `--agent-skill` output has YAML front matter plus markdown body matching the same source content and is complete enough for operator use
result: pending

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
