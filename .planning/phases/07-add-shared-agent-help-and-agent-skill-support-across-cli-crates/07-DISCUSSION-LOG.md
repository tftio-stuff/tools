# Phase 7: Add shared --agent-help and --agent-skill support across CLI crates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-22
**Phase:** 07-add-shared-agent-help-and-agent-skill-support-across-cli-crates
**Areas discussed:** Agent doc contract, Skill doc behavior, Crate coverage, CLI behavior,
Agent help content depth, Agent skill shape, Output relationship, Per-tool specificity

---

## Agent doc contract

| Option | Description | Selected |
|--------|-------------|----------|
| Canonical YAML | One canonical YAML document shape across all binaries | ✓ |
| Custom sections | Each crate chooses its own top-level sections | |

**User's choice:** Canonical YAML
**Notes:** The user confirmed `--agent-help` should use one canonical YAML shape across all
binaries.

---

## Skill doc behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Whole-tool skill doc | Print one skill-style document describing the whole tool | ✓ |
| Topic-specific skill lookup | Require an extra topic/skill name argument | |
| Other behavior | Different behavior outside the two options above | |

**User's choice:** Whole-tool skill doc
**Notes:** `--agent-skill` should describe the entire tool, not require a topic selector.

---

## Crate coverage

| Option | Description | Selected |
|--------|-------------|----------|
| All seven binaries | Implement both flags in every binary crate in this workspace | ✓ |
| Partial rollout | Implement only a subset of binaries now | |

**User's choice:** All seven binaries
**Notes:** Coverage includes `prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`,
`gator`, and `bsky-comment-extractor`.

---

## CLI behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Hidden global top-level flags | Hidden, top-level only, valid without subcommand, stdout, exit 0 | ✓ |
| Normal visible help flags | Visible in standard help text | |
| Subcommand-scoped flags | Resolved within subcommands instead of at top level | |

**User's choice:** Hidden global top-level flags
**Notes:** The user confirmed all of these behaviors should be true: hidden global flags,
top-level only, valid with no subcommand, stdout on success with exit code `0`, and hidden from
normal help output.

---

## Agent help content depth

| Option | Description | Selected |
|--------|-------------|----------|
| Exhaustive standalone reference | Cover all commands, flags, args, examples, outputs, env/config, and error modes | ✓ |
| Minimal summary | Short overview only | |

**User's choice:** Exhaustive standalone reference
**Notes:** The user explicitly wants full command coverage, common and edge-case examples, output
shapes, environment variables, config files and default paths, failure modes, and likely operator
mistakes.

---

## Agent skill shape

| Option | Description | Selected |
|--------|-------------|----------|
| Claude-style skill doc | YAML front matter followed by markdown instructions | ✓ |
| YAML only | Structured metadata without markdown body | |
| Markdown only | Instructions without skill front matter | |

**User's choice:** Claude-style skill doc
**Notes:** `--agent-skill` should be ready to save as a Claude skill document.

---

## Output relationship

| Option | Description | Selected |
|--------|-------------|----------|
| Same underlying information | Two renderings over one shared knowledge model | ✓ |
| Different emphasis | `--agent-help` and `--agent-skill` diverge materially in content | |

**User's choice:** Same underlying information
**Notes:** The user said "Same, I think?" indicating the two outputs should share the same core
information.

---

## Per-tool specificity

| Option | Description | Selected |
|--------|-------------|----------|
| Shared inherited sections + per-crate specifics | Reuse inherited behavior from `cli-common`, customize the rest | ✓ |
| Fully generic template | Same content structure and details for every tool | |
| Fully bespoke docs | No shared inherited sections at all | |

**User's choice:** Shared inherited sections + per-crate specifics
**Notes:** Shared behavior from `cli-common` can stay common, but tool-specific flags, commands,
configuration, and behavior need per-crate treatment.

---

## the agent's Discretion

- Exact implementation shape inside `cli-common`
- Internal representation shared by `--agent-help` and `--agent-skill`
- Test strategy and helper factoring across crates

## Deferred Ideas

None.
