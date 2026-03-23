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
updated: 2026-03-23T14:38:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Human mode still exposes the normal CLI surface
expected: Run a migrated tool without agent env vars and confirm normal help remains fully available instead of being reduced to the restricted agent surface.
result: issue
reported: "> cargo run -q -p tftio-prompter -- --help printed a dead_code warning for render_standard_completion_for_command before showing the expected normal command list"
severity: minor

### 2. Agent help shows only the allowed capability surface
expected: With matching TFTIO_AGENT_TOKEN and TFTIO_AGENT_TOKEN_EXPECTED, a migrated tool such as prompter should print structured `--agent-help` output that lists only the allowlisted capabilities.
result: issue
reported: "> TFTIO_AGENT_TOKEN=phase7-test-token TFTIO_AGENT_TOKEN_EXPECTED=phase7-test-token cargo run -q -p tftio-prompter -- --agent-help printed the expected restricted capability list, but still prefixed it with the same dead_code warning"
severity: minor

### 3. Agent skill output is scoped to one capability
expected: With agent mode active, `--agent-skill <name>` should print only the contract for that single visible capability and should not reveal hidden commands or extra capabilities.
result: issue
reported: "> TFTIO_AGENT_TOKEN=phase7-test-token TFTIO_AGENT_TOKEN_EXPECTED=phase7-test-token cargo run -q -p tftio-prompter -- --agent-skill render-prompts printed the expected single-capability contract, but still prefixed it with the same dead_code warning"
severity: minor

### 4. Hidden commands stay redacted in help and parsing
expected: In agent mode, ordinary `--help`/`help` output should omit hidden commands, and trying a hidden command or typo path should fail as if the command does not exist without leaking suggestions.
result: issue
reported: "> agent-mode --help omitted hidden commands and agent-mode doctor failed as unrecognized, but both commands still printed the same dead_code warning before the user-facing output"
severity: minor

### 5. Workspace rollout smoke stays green across migrated tools
expected: The shared rollout checks, such as `bash tests/cli/06-agent-mode.sh` or `just cli-consistency`, should pass and demonstrate the restricted surface consistently across the migrated workspace CLIs.
result: pass

## Summary

total: 5
passed: 1
issues: 4
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "Human-mode help runs cleanly without emitting internal compiler warnings before the normal CLI surface"
  status: failed
  reason: "User reported: running prompter --help printed a dead_code warning for render_standard_completion_for_command before the expected help output"
  severity: minor
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Agent-facing help renders cleanly without internal compiler warnings before the restricted capability surface"
  status: failed
  reason: "User reported: running prompter --agent-help printed the expected restricted capability list, but still prefixed it with the same dead_code warning"
  severity: minor
  test: 2
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Agent skill output renders cleanly without internal compiler warnings before the single-capability contract"
  status: failed
  reason: "User reported: running prompter --agent-skill render-prompts printed the expected single-capability contract, but still prefixed it with the same dead_code warning"
  severity: minor
  test: 3
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Agent-mode help and hidden-command rejection render cleanly without internal compiler warnings before the redacted output"
  status: failed
  reason: "User reported: agent-mode help omitted hidden commands and agent-mode doctor failed as unrecognized, but both commands still printed the same dead_code warning before the user-facing output"
  severity: minor
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
