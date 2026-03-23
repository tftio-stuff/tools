---
status: complete
phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
source:
  - 07-01-SUMMARY.md
  - 07-02-SUMMARY.md
  - 07-03-SUMMARY.md
  - 07-04-SUMMARY.md
  - 07-05-SUMMARY.md
  - 07-06-SUMMARY.md
started: 2026-03-23T14:29:00Z
updated: 2026-03-23T15:08:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Human mode still exposes the normal CLI surface
expected: Run a migrated tool without agent env vars and confirm normal help remains fully available instead of being reduced to the restricted agent surface, without an internal warning prefix, and with user-facing help copy.
result: pass

### 2. Agent help shows only the allowed capability surface
expected: With matching TFTIO_AGENT_TOKEN and TFTIO_AGENT_TOKEN_EXPECTED, a migrated tool such as prompter should print structured `--agent-help` output that lists only the allowlisted capabilities, without an internal warning prefix.
result: pass

### 3. Agent skill output is scoped to one capability
expected: With agent mode active, `--agent-skill <name>` should print only the contract for that single visible capability and should not reveal hidden commands or extra capabilities, without an internal warning prefix.
result: pass

### 4. Hidden commands stay redacted in help and parsing
expected: In agent mode, ordinary `--help`/`help` output should omit hidden commands, and trying a hidden command or typo path should fail as if the command does not exist without leaking suggestions or internal warning output.
result: pass

### 5. Workspace rollout smoke stays green across migrated tools
expected: The shared rollout checks, such as `bash tests/cli/06-agent-mode.sh` or `just cli-consistency`, should pass and demonstrate the restricted surface consistently across the migrated workspace CLIs.
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None.
