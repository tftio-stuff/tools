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
updated: 2026-03-23T14:58:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Human mode still exposes the normal CLI surface
expected: Run a migrated tool without agent env vars and confirm normal help remains fully available instead of being reduced to the restricted agent surface, without an internal warning prefix.
result: issue
reported: "> cargo run -q -p tftio-prompter -- --help no longer prints the warning, but the help starts with 'Command-line interface structure for the prompter tool.' and 'This structure defines the main CLI interface using clap's derive API.', which reads like an internal doc comment instead of user-facing help"
severity: minor

### 2. Agent help shows only the allowed capability surface
expected: With matching TFTIO_AGENT_TOKEN and TFTIO_AGENT_TOKEN_EXPECTED, a migrated tool such as prompter should print structured `--agent-help` output that lists only the allowlisted capabilities, without an internal warning prefix.
result: pass

### 3. Agent skill output is scoped to one capability
expected: With agent mode active, `--agent-skill <name>` should print only the contract for that single visible capability and should not reveal hidden commands or extra capabilities, without an internal warning prefix.
result: pass

### 4. Hidden commands stay redacted in help and parsing
expected: In agent mode, ordinary `--help`/`help` output should omit hidden commands, and trying a hidden command or typo path should fail as if the command does not exist without leaking suggestions or internal warning output.
result: issue
reported: "> agent-mode --help correctly omitted hidden commands, and agent-mode doctor failed cleanly as unrecognized with no suggestion leak, but the help output still starts with 'Command-line interface structure for the prompter tool.' and 'This structure defines the main CLI interface using clap's derive API.', which reads like internal implementation prose instead of user-facing help"
severity: minor

### 5. Workspace rollout smoke stays green across migrated tools
expected: The shared rollout checks, such as `bash tests/cli/06-agent-mode.sh` or `just cli-consistency`, should pass and demonstrate the restricted surface consistently across the migrated workspace CLIs.
result: pass

## Summary

total: 5
passed: 3
issues: 2
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "Human-mode help renders as user-facing CLI help rather than internal implementation prose"
  status: failed
  reason: "User reported: prompter --help starts with 'Command-line interface structure for the prompter tool.' and 'This structure defines the main CLI interface using clap's derive API.', which reads like an internal doc comment instead of user-facing help"
  severity: minor
  test: 1
  root_cause: "The top-level `Cli` derive in `crates/prompter/src/main.rs` still uses Rust doc comments as clap about/long_about text, so both human `--help` and agent-mode `--help` render implementation prose copied from source comments."
  artifacts:
    - path: "crates/prompter/src/main.rs"
      issue: "Top-level CLI doc comments are surfaced directly as clap help text"
  missing:
    - "Replace the internal struct doc comments with user-facing clap `about`/`long_about` text or remove the internal prose from help rendering"
  debug_session: ""

- truth: "Agent-mode help renders as user-facing CLI help rather than internal implementation prose"
  status: failed
  reason: "User reported: agent-mode --help correctly omitted hidden commands and doctor failed cleanly as unrecognized, but the help still starts with 'Command-line interface structure for the prompter tool.' and 'This structure defines the main CLI interface using clap's derive API.', which reads like internal implementation prose instead of user-facing help"
  severity: minor
  test: 4
  root_cause: "Agent-mode `--help` reuses the same clap metadata as human help, so the internal doc-comment prose on the top-level `Cli` derive leaks into the redacted help output too."
  artifacts:
    - path: "crates/prompter/src/main.rs"
      issue: "Shared clap help metadata carries internal doc-comment prose into agent-mode help"
  missing:
    - "Set user-facing clap `about`/`long_about` text for the top-level CLI so both human and agent help render clean copy"
  debug_session: ""
