# Phase 7: Workspace agent mode in cli-common: token-gated restricted capability surface, inspectable agent help, and shared --agent-skill support across tools - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Move the Silent Critic-style worker visibility boundary into `tftio-cli-common` so workspace CLIs can expose an inspectable but restricted agent-facing surface. In this phase, agent mode is token-gated by environment variables, fail-closed, and available across all workspace tools that use `cli-common`. This phase supersets the original `bce --agent-help` scope by delivering shared `--agent-help`, shared `--agent-skill <name>`, declarative capability policy, and tool adoption across the workspace.

</domain>

<decisions>
## Implementation Decisions

### Activation boundary
- **D-01:** Restricted agent mode activates only when a presented token env var is set and exactly matches an expected token env var at process start. If the token is absent or invalid, the CLI stays in normal human mode.
- **D-02:** The token contract is workspace-shared and generic: every tool uses the same presented-token env var name and the same expected-token env var name.
- **D-03:** The boundary is fail-closed for agent mode activation. Presence of the mode env var alone is insufficient.

### Visible agent surface
- **D-04:** Agent mode uses strong hiding everywhere. Disallowed functionality is absent from `--agent-help`, normal clap help, completions, suggestion paths, and `--agent-skill` output.
- **D-05:** If an agent invokes a hidden command or flag, parsing must fail indistinguishably from nonexistence. Error output must not reveal that a larger surface exists.
- **D-06:** Hidden commands and higher-role operations must not be discoverable through normal interaction in agent mode.

### Agent self-description
- **D-07:** Shared `--agent-help` output is structured text optimized for LLM reading on stdout. JSON is not required in this phase.
- **D-08:** Shared `--agent-skill <name>` outputs the inspectable contract for one allowed capability/workflow only, filtered by current agent-mode visibility.
- **D-09:** `--agent-help` provides the top-level manifest of visible capabilities, arguments, output semantics, and constraints for the current agent-mode surface.

### Policy model
- **D-10:** `cli-common` owns a declarative policy model. Tools register named capabilities/skills plus the commands, subcommands, and flags belonging to each capability.
- **D-11:** Allowlisting is capability-level, not raw parser-fragment-level. The visible surface is built from named capability groups so `--agent-skill` can describe them directly.
- **D-12:** Tool-local procedural policy logic is not the default design. Uniform policy declaration belongs in `cli-common`.

### Rollout scope
- **D-13:** This phase delivers both the shared `cli-common` substrate and migration of all workspace CLIs that already depend on `cli-common`.
- **D-14:** The Phase 6 `bce --agent-help` work is absorbed into this phase rather than implemented as a separate tool-local one-off.

### the agent's Discretion
- Exact env var names
- Exact structured-text formatting and section titles for `--agent-help` and `--agent-skill`
- Internal Rust type names for capability declarations and filtering helpers
- Migration sequencing across workspace crates, as long as all `cli-common` consumers are covered in this phase

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Silent Critic visibility model
- `docs/the_silent_critic_tooling_design.md` — role-scoped manifest design, worker-visible surface rules, and the requirement that higher-role capabilities stay undisclosed during normal worker interaction
- `docs/the-silent-critic-system-spec.md` — execution-time visibility invariants, non-escalation requirements, and the rule that workers must not reveal hidden criteria or higher authorization surfaces through ordinary tool usage
- `docs/the_silent_critic_formal_appendix.md` — formal visibility-boundary property `M(worker) ⊂ M(auditor) ⊂ M(operator)` and non-escalation constraints for role-scoped surfaces

### Existing workspace phase/requirement context
- `.planning/ROADMAP.md` — Phase 6 `Agent Help` scope that this phase supersets, plus the new Phase 7 roadmap entry
- `.planning/PROJECT.md` — current `bce` milestone intent and existing `--agent-help` target
- `.planning/REQUIREMENTS.md` — existing `AGENT-01` requirement that is absorbed into the shared workspace agent-mode design

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/cli-common/src/app.rs`: `ToolSpec` and workspace tool metadata already centralize shared CLI capability description and are the natural place to extend with agent-surface metadata
- `crates/cli-common/src/command.rs`: shared standard-command mapping and execution already centralize metadata/help command behavior, making it the right place to insert shared agent-mode filtering and shared agent-facing commands
- `crates/cli-common/src/lib.rs`: current re-export layer can expose the shared agent-mode types and helpers to all tools
- `crates/silent-critic/src/commands/session.rs`: existing worker-token model and `run_manifest` behavior provide the closest in-repo reference for role-scoped visibility and runtime manifest generation

### Established Patterns
- Workspace CLIs already route common UX through `ToolSpec`, `StandardCommand`, and shared runner/error helpers in `cli-common`
- `bce` already has a top-level hidden `--agent-help` flag, showing there is an existing but tool-local agent surface stub to replace with a shared implementation
- `silent-critic` already enforces a token-gated worker boundary using `SILENT_CRITIC_TOKEN`, and the docs explicitly require the worker-facing surface to hide higher-role capabilities

### Integration Points
- `crates/cli-common/src/app.rs`
- `crates/cli-common/src/command.rs`
- `crates/cli-common/src/lib.rs`
- Every CLI crate that depends on `tftio-cli-common`, including `bsky-comment-extractor`, `todoer`, `silent-critic`, `prompter`, `asana-cli`, `unvenv`, and `gator`

</code_context>

<specifics>
## Specific Ideas

- The tool should be fully inspectable by an agent, but the agent must see only a subset of functionality.
- The orchestrator injects the token into the agent environment so the agent can operate autonomously inside the restricted surface.
- `--agent-skill` is part of the phase deliverable, not an optional later extension.
- The new workspace agent mode is intended as a complete superset of the original `bce --agent-help` Phase 6 idea.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools*
*Context gathered: 2026-03-22*
