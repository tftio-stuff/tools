# Phase 2: YOLO Injection - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Inject agent-appropriate YOLO flags (autonomous mode) into the agent command by default. Claude gets `--dangerously-skip-permissions`, Codex gets `--full-auto`. Add `--no-yolo` CLI flag to opt out. Session mode skips YOLO injection entirely. Gemini has no known equivalent -- warn and skip.

</domain>

<decisions>
## Implementation Decisions

### --no-yolo interaction with --session
- `--no-yolo` and `--session` are mutually exclusive -- add to `Cli::validate()` conflict check (same pattern as `--share-worktrees` + `--session`)
- YOLO injection is skipped entirely in session mode -- session contract controls everything, agent uses its default permission mode
- This follows the Phase 1 pattern where session mode bypasses normal flow

### Gemini equivalent flag
- Gemini has no known YOLO flag -- skip injection and print stderr warning: `gator: no YOLO flag known for gemini, skipping`
- `--no-yolo` suppresses the Gemini warning (user explicitly doesn't want YOLO, so the missing flag is irrelevant)

### Claude's Discretion
- Where YOLO injection logic lives (in `build_command()` alongside prompt injection, or in `lib::run()` before calling it)
- Exact stderr warning format for Gemini
- Test strategy for verifying flag injection (unit tests on `build_command()` return value)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Agent command building
- `crates/gator/src/agent.rs` -- `build_command()` function with per-agent branching for prompt injection, `exec_command()`, existing tests for command construction
- `crates/gator/src/cli.rs` -- `Agent` enum (Claude, Codex, Gemini), `Cli` struct, `validate()` mutual exclusion checks

### Orchestration
- `crates/gator/src/lib.rs` -- `run()` function, session vs non-session branching, where `build_command()` is called (line 113)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Agent` enum in cli.rs with `Claude`, `Codex`, `Gemini` variants -- match arms for per-agent behavior
- `build_command()` in agent.rs already has per-agent branching (`match agent { Agent::Claude => ..., Agent::Codex => ..., Agent::Gemini => ... }`) for prompt injection
- `Cli::validate()` -- session conflict checker extended in Phase 1, same pattern for `--no-yolo`

### Established Patterns
- Per-agent differences handled in `build_command()` via `match agent`
- Session mode bypasses all auto-detection and injection (WorktreeInfo::default(), no sibling grants)
- `lib::run()` calls `build_command()` at line 113 with `&cli.agent`, `policy_path`, `prompt`, `&cli.agent_args`

### Integration Points
- `build_command()` signature or a new parameter for YOLO injection
- `Cli` struct needs `--no-yolo` field
- `lib::run()` session branch should skip YOLO (already skips worktree detection)
- `lib::run()` non-session branch needs to determine whether to inject YOLO before calling `build_command()`

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 02-yolo-injection*
*Context gathered: 2026-03-18*
