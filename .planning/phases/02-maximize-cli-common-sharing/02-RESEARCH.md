# Phase 2 Research — Maximize cli-common sharing

**Gathered:** 2026-03-22
**Status:** Complete
**Scope:** `crates/cli-common` plus all workspace CLI crates

## Question

After Phase 01 established the shared base UX, what duplicated or broadly useful CLI code still remains in the tools, and how should the remaining extraction work be split so `tftio-cli-common` becomes the maximal shared CLI surface without swallowing domain behavior?

## Current State Inventory

### What `cli-common` already owns

`cli-common` now provides:
- `ToolSpec`
- `StandardCommand`
- shared version/license/completion/doctor/update dispatch
- shared JSON envelopes and top-level JSON/text error rendering
- shared spinner creation and TTY helpers

This removed the most obvious duplicated base UX, but it did **not** eliminate the remaining boilerplate around those primitives.

### Repeated patterns still present in tool crates

| Area | Current duplication |
|------|---------------------|
| Tool metadata scaffolding | `tool_spec()` exists in `gator`, `todoer`, `silent-critic`, `bce`, `unvenv`, `asana-cli`, and `prompter` |
| Doctorless adapters | local `NoDoctor` structs still exist in `gator`, `todoer`, and `silent-critic` |
| Meta-command adapters | local `run_meta_command()` mapping functions still exist in `gator`, `todoer`, `silent-critic`, and `bce` |
| Doctor report shaping | `prompter` maintains a parallel plain-text + JSON doctor renderer around the same state model |
| Completion rendering | `prompter` still reimplements completion instruction rendering and raw script buffering because `cli-common` only writes directly to stdout |
| Top-level error styling | `unvenv`, `prompter`, and some domain paths still format their own plain-text fatal output instead of sharing one presentation helper |
| Command response shaping | `todoer` and `silent-critic` still manually match JSON-vs-text success rendering in many command arms |

## Key Findings

### 1. The biggest remaining wins are adapter extraction, not brand-new UX concepts

Phase 01 moved the contract into `cli-common`, but many tools still own the same tiny adapter layer around it. The next milestone should delete those per-tool adapters by making `cli-common` capable of:
- constructing common `ToolSpec` variants
- supplying a shared doctorless implementation
- mapping crate-local metadata enums to `StandardCommand`
- rendering completions to a buffer before printing

### 2. `prompter` is the best signal for missing shared primitives

`prompter` still owns:
- completion instruction rendering
- completion buffering before augmentation
- a custom doctor JSON renderer separate from the shared plain-text renderer

Those are strong indicators that `cli-common` still lacks reusable interfaces for “render, then post-process” workflows and structured doctor reporting.

### 3. `todoer` and `silent-critic` still expose reusable command-output patterns

Those tools repeatedly do:
- command dispatch
- JSON/text branching
- shared top-level `ok_response`/`print_error` usage
- plain text table/list rendering

Not all of that belongs in `cli-common`, but a generic command-response helper for JSON-vs-text success/failure paths likely does.

### 4. `unvenv` and `asana-cli` still reveal output-layer gaps

`unvenv` retains custom colored fatal printing and TTY-aware scan output. `asana-cli` still keeps doctor composition local. Some of that is domain-specific, but shared fatal-output, banner, and doctor-report helpers would remove more per-tool code.

### 5. “Maximal shared surface” still has a boundary

The following should remain local:
- gator sandbox / session logic
- todoer project resolution, schema, and task tables
- silent-critic supervision domain commands and text summaries
- `bce` extraction/runtime logic and XDG DB path semantics
- `prompter` profile graph logic and dynamic completion data source
- `asana-cli` API/domain command tree and manpage generation

The target is **shared CLI infrastructure**, not domain behavior.

## Recommended `cli-common` Expansion Areas

### `app`
Add constructors and presets that remove repeated `tool_spec()` functions:
- workspace-default repo preset
- no-update / no-doctor variants
- machine-tool vs user-tool helpers where semantics differ

### `command`
Expand beyond `run_standard_command()`:
- helper for meta-command dispatch from crate-local enums
- helper for “standard command or domain action” entrypoints
- shared no-doctor adapter to delete local `NoDoctor` structs

### `completions`
Add render-first APIs:
- `render_completion_script<T>(shell) -> String`
- `render_completion_instructions(shell, bin_name) -> String`
- `write_completion_script(...)`

This would let `prompter` keep augmentation logic while deleting its duplicate instruction wrapper and direct `clap_complete` calls.

### `doctor`
Add structured report primitives:
- report/state type shared by plain-text and JSON rendering
- reusable JSON doctor renderer built from tool checks + extra state
- small helper constructors for path/config checks

### `output` / `error`
Add reusable fatal/info presentation helpers:
- consistent plain-text fatal prefix rendering
- banner/header helpers on stdout vs stderr
- maybe a small “report to stdout / report error to stderr” helper for domain tools

### `json` / command response helpers
Add a helper that reduces repeated success branching:
- shared JSON-vs-text response adapter for command handlers
- response wrapper for command name + serializable payload + text fallback

## Proposed Execution Shape

### Wave 1 — extend `cli-common` abstractions
Build the missing reusable primitives first so consumer migrations delete code instead of just moving it around.

### Wave 2 — migrate the low-risk adapter-heavy tools
Move `gator`, `todoer`, `silent-critic`, and `bce` onto the new helpers first. These crates have obvious local adapter functions to delete.

### Wave 3 — migrate richer consumers
Then refactor `unvenv`, `asana-cli`, and `prompter` onto the expanded surface, especially the completion and doctor-report interfaces.

### Wave 4 — dependency cleanup and enforcement
Once extraction is done, centralize the remaining shared dependencies behind `cli-common` where practical and add grep/shell/cargo checks proving the local boilerplate stayed deleted.

## Risks

### Over-generalizing domain output
A shared command-response helper can reduce repetition, but shared table rendering or domain summaries would be a mistake.

### Clap reuse friction
Some metadata enums have different shapes (`todoer` version takes `--json`, `gator` uses a global `--json`, `bce` has a flat command enum). Shared helpers must remove boilerplate without forcing one parser shape everywhere.

### Hidden dependency expansion
Maximal extraction will likely move more direct dependency usage (`clap_complete`, `colored`, `toml`, maybe some serde helpers) behind `cli-common`. That is desirable only when behavior becomes genuinely shared and test-covered.

## Validation Architecture

### Automated verification
- per shared-surface task: `cargo test -p tftio-cli-common && cargo clippy -p tftio-cli-common -- -D warnings`
- per migration wave: targeted crate tests for the touched tools
- final contract check: `just cli-metadata-consistency && just cli-consistency && just test && just lint`

### Contract enforcement opportunities
- grep-based assertions that local `tool_spec()` / `NoDoctor` / `run_meta_command()` boilerplate is removed where shared helpers replace it
- shell suite assertions for doctor/completion/error/json contracts
- focused tests in `cli-common` for completion rendering buffers and doctor JSON report rendering

## Planning Recommendation

Split the implementation into four plans:
1. expand `cli-common` with reusable app/command/completion/doctor/output primitives
2. migrate the adapter-heavy command-line tools off local metadata/doctorless/meta boilerplate
3. migrate the richer tools (`unvenv`, `asana-cli`, `prompter`) and extract the remaining generally useful helpers
4. finish with dependency cleanup, repository-level enforcement, and documentation of the intentional boundary
