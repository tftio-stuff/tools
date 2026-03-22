# Phase 7: Add shared --agent-help and --agent-skill support across CLI crates - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Add shared support in `cli-common` for hidden top-level `--agent-help` and `--agent-skill`
flags, then wire that support into every binary crate in this workspace so each tool can emit
complete agent-facing documentation without requiring source inspection. `cli-common` remains a
library; milestone completion is when all seven binary crates expose the flags and return their
tool-specific content.

</domain>

<decisions>
## Implementation Decisions

### Agent help contract
- **D-01:** `--agent-help` must emit one canonical YAML document shape across all binaries.
- **D-02:** The YAML must be exhaustive and standalone for agents: every command and subcommand,
  every flag and positional argument, common and edge-case examples, output shapes, environment
  variables, config files and default paths, failure modes, and likely operator mistakes.
- **D-03:** The content may share inherited sections from `cli-common` where behavior is truly
  shared, but tool-specific commands, configuration, environment, and errors must remain per-crate.

### Agent skill contract
- **D-04:** `--agent-skill` must output the same underlying information as `--agent-help`, not a
  different capability or separate topic selector.
- **D-05:** `--agent-skill` must be rendered as a ready-to-save Claude-style skill document:
  YAML front matter first, then a markdown body with full tool instructions, examples, constraints,
  and error handling.

### CLI behavior
- **D-06:** Both `--agent-help` and `--agent-skill` are hidden global flags.
- **D-07:** Both flags are top-level only, valid with no subcommand, print to stdout on success,
  and exit with code `0`.
- **D-08:** Both flags must stay hidden from normal `--help` output.

### Workspace rollout
- **D-09:** This phase covers all seven binary crates in the workspace:
  `prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`, `gator`, and
  `bsky-comment-extractor`.
- **D-10:** Shared flag parsing and inherited behavior belong in `cli-common`; each binary still
  owns its final agent-facing content and any crate-specific wiring needed to expose the flags.

### the agent's Discretion
- Exact Rust API shape in `cli-common` for representing shared agent-help and agent-skill payloads
- How to minimize duplication between YAML reference output and skill-document rendering
- How to integrate shared flag handling into crates with different clap entrypoint structures
- Test layout and helper abstractions, as long as every binary crate ends with the required flags
  and the documented behavior above

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and requirements
- `.planning/ROADMAP.md` — Phase 7 placeholder and milestone sequencing in the active planning tree
- `.planning/REQUIREMENTS.md` — Existing `AGENT-01` requirement and current milestone requirement
  vocabulary
- `.planning/PROJECT.md` — Current project framing in the active planning tree; confirms current
  milestone context and the existing `--agent-help` expectation

### Workspace conventions
- `CLAUDE.md` — Workspace structure, crate list, command conventions, and quality gates relevant to
  a cross-crate rollout

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/cli-common/src/lib.rs` — current shared CLI utility crate and likely home for new shared
  agent-doc abstractions
- `crates/cli-common/src/output.rs` — existing shared output helpers; possible place for common
  emission helpers or formatting support
- `crates/bsky-comment-extractor/src/cli.rs` — existing hidden global `--agent-help` clap pattern
- `crates/bsky-comment-extractor/src/main.rs` — existing top-level `agent_help` dispatch and
  placeholder YAML emission

### Established Patterns
- Binary crates use `clap` derive parsers with crate-specific top-level entrypoints in either
  `src/main.rs`, `src/cli.rs`, or `src/cli/mod.rs`
- Shared CLI behavior is usually factored through `cli-common`, but each binary retains its own
  clap types and dispatch logic
- Several tools already distinguish human-facing output from machine-facing output, so agent-doc
  emission should fit the existing stdout-oriented CLI style

### Integration Points
- `prompter`: `crates/prompter/src/main.rs`
- `unvenv`: `crates/unvenv/src/main.rs`
- `asana-cli`: `crates/asana-cli/src/cli/mod.rs`
- `todoer`: `crates/todoer/src/cli.rs`
- `silent-critic`: `crates/silent-critic/src/cli.rs`
- `gator`: `crates/gator/src/cli.rs`
- `bsky-comment-extractor`: `crates/bsky-comment-extractor/src/cli.rs` and
  `crates/bsky-comment-extractor/src/main.rs`

</code_context>

<specifics>
## Specific Ideas

- The output should be "complete" and "exhaustive", not a short summary.
- `--agent-help` and `--agent-skill` should describe the same tool knowledge, rendered in two
  different output formats.
- Shared inherited behavior may stay identical across tools, but everything else should be
  per-crate.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-add-shared-agent-help-and-agent-skill-support-across-cli-crates*
*Context gathered: 2026-03-22*
