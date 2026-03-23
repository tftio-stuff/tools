# Phase 7: Workspace agent mode in cli-common: token-gated restricted capability surface, inspectable agent help, and shared --agent-skill support across tools - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-22
**Phase:** 07-workspace-agent-mode-in-cli-common-token-gated-restricted-capability-surface-inspectable-agent-help-and-shared-agent-skill-support-across-tools
**Areas discussed:** Activation boundary, Visible agent surface, Agent self-description, Policy model, Rollout scope

---

## Activation boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Fail closed on valid token only | Restricted agent mode activates only when a specific env var is present and a valid token match succeeds; otherwise the CLI behaves normally for humans | ✓ |
| Presence-only switch | Any non-empty env var activates restricted mode | |
| Always restricted in orchestrated contexts | Separate env or mode flag toggles restricted mode without token verification | |

**User's choice:** Fail closed on valid token only
**Notes:** Restricted mode is activated by exact env-to-env token match. The token source of truth is a shared presented-token env var plus a shared expected-token env var. Workspace-generic env var names are preferred over tool-local ones.

## Visible agent surface

| Option | Description | Selected |
|--------|-------------|----------|
| Strong hiding everywhere | Disallowed functionality disappears from help, completions, suggestions, and agent-facing docs; parse attempts fail as if nonexistent | ✓ |
| Help-hidden only | Hidden in docs/help, but still executable if guessed | |
| Visible but denied | Visible but execution returns a restriction error | |

**User's choice:** Strong hiding everywhere
**Notes:** Hidden commands or flags must parse-fail indistinguishably from unknown input, without revealing that a larger surface exists.

## Agent self-description

| Option | Description | Selected |
|--------|-------------|----------|
| Structured text help | Shared `--agent-help` emits stable structured text optimized for LLM reading | ✓ |
| JSON only | Strict machine-readable contract only | |
| Dual format from day one | Text plus JSON in the same phase | |

**User's choice:** Structured text help
**Notes:** `--agent-skill <name>` is a capability-specific reference surface for one visible capability/workflow, not a runtime authority switch.

## Policy model

| Option | Description | Selected |
|--------|-------------|----------|
| Declarative policy in `cli-common` | Tools register visibility metadata in shared types | ✓ |
| Tool-local callbacks | Each tool decides policy procedurally | |
| Hybrid | Shared declarations with tool-local escape hatches | |

**User's choice:** Declarative policy in `cli-common`
**Notes:** Allowlisting unit is capability-level groups rather than raw commands/flags. This is intended to support `--agent-skill` directly and stay inspectable.

## Rollout scope

| Option | Description | Selected |
|--------|-------------|----------|
| `cli-common` plus all workspace CLIs | Shared substrate lands and every `cli-common` consumer adopts it in this phase | ✓ |
| `cli-common` plus proving tools first | Shared substrate plus a limited initial rollout | |
| `cli-common` only | Framework now, migrations later | |

**User's choice:** `cli-common` plus all workspace CLIs
**Notes:** This phase is a complete superset of the original Phase 6 `bce --agent-help` scope and includes shared `--agent-skill`.

## the agent's Discretion

- Exact env var names
- Final structured-text schema for agent-facing help output
- Rust type and helper naming inside `cli-common`
- Internal rollout order across workspace CLIs

## Deferred Ideas

None.
