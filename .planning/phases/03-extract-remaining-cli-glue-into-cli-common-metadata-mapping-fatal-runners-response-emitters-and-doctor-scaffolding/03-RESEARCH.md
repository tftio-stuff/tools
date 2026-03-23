# Phase 3 Research — Extract remaining CLI glue into `cli-common`

**Gathered:** 2026-03-22
**Status:** Complete
**Scope:** `crates/cli-common` plus the remaining per-tool entrypoint, response, and doctor scaffolding

## Question

After Phase 02 extracted the major shared CLI adapters, what reusable glue still lives in individual tools, and how should the remaining work be split so `tftio-cli-common` owns the rest of the broadly useful metadata, runner, response, and doctor infrastructure?

## Current State Inventory

### What `cli-common` already owns

`cli-common` now provides:
- shared workspace `ToolSpec` presets via `workspace_tool(...)`
- shared standard-command dispatch, including the doctorless adapter
- shared version/license/completions/doctor/update entrypoints
- shared JSON envelopes and shared `render_response(...)`
- shared completion rendering buffers
- shared structured doctor-report rendering
- shared top-level error rendering and spinner / TTY helpers

This deleted the biggest common substrate, but several thin glue layers still remain in tool crates.

### Remaining repeated patterns in tool crates

| Area | Current duplication |
|------|---------------------|
| Metadata enum mapping | `gator` and `bce` still translate local clap enums to `StandardCommand` with local `match` blocks |
| Top-level fatal runners | `gator`, `bce`, `unvenv`, and `asana-cli` still hand-roll parse/run/error/exit plumbing |
| JSON-vs-text success branching | `silent-critic` still has multiple `if json { ... } else { ... }` success paths; `todoer` still has a few command-specific branches |
| Doctor provider scaffolding | `bce`, `unvenv`, and `prompter` still repeat repo/version/tool-check wiring around the same shared doctor renderer |
| Zero-value wrappers | `todoer` still defines a local `print_error(...)` wrapper around `cli-common` |

## File-Level Findings

### 1. Remaining metadata mapping glue is narrow but still duplicated

Observed in:
- `crates/gator/src/main.rs`
- `crates/bsky-comment-extractor/src/main.rs`

Both crates still do a local enum-to-`StandardCommand` conversion before calling the shared dispatcher. That pattern is a good fit for a tiny adapter helper in `cli-common` so the entrypoints only describe the tool-specific metadata enum once.

### 2. Fatal runner logic is still inconsistent across tools

Observed in:
- `crates/asana-cli/src/main.rs`
- `crates/unvenv/src/main.rs`
- `crates/bsky-comment-extractor/src/main.rs`
- `crates/gator/src/main.rs`

These crates still repeat combinations of:
- parse CLI
- run validation / dispatch
- print plain-text fatal errors with `print_error(...)` or custom `eprintln!`
- choose exit code

A shared runner helper would finish the base UX unification and reduce drift in top-level failure behavior.

### 3. `render_response(...)` helped, but it did not finish success-path extraction

Most obvious in:
- `crates/silent-critic/src/main.rs`
- `crates/todoer/src/main.rs`
- selected paths in `crates/prompter/src/lib.rs`

These crates still build JSON payloads and text strings separately, then branch on `json` in each command arm. That is still shared CLI plumbing rather than domain behavior. A richer output type or emitter in `cli-common` would reduce a large amount of repeated success-path code.

### 4. Doctor rendering moved, but provider construction did not

Observed in:
- `crates/bsky-comment-extractor/src/main.rs`
- `crates/unvenv/src/main.rs`
- `crates/prompter/src/doctor.rs`

The shared renderer exists, but each tool still repeats the same provider skeleton:
- repo info
- current version
- optional tool checks
- extra details / warnings / errors

That points to a missing builder/helper layer in `cli-common`, not more missing renderer logic.

### 5. The remaining extraction boundary is still infrastructure, not domain behavior

The following should remain local after this phase:
- `gator` sandbox/session behavior
- `todoer` task/project resolution and task rendering
- `silent-critic` criterion/session/project domain summaries
- `bce` extraction runtime, XDG DB-path behavior, and extraction summary text
- `prompter` config/profile graph logic and dynamic completion augmentation
- `unvenv` venv scanning/report contents
- `asana-cli` Asana API command tree and manpage generation

The goal is to remove the last reusable CLI glue, not to flatten domain output into `cli-common`.

## Recommended `cli-common` Expansion Areas

### `command`
Add the final metadata adapter helpers:
- local metadata enum -> `StandardCommand` conversion helpers
- helper entrypoint for “run a standard command or continue to domain dispatch”

### `runner`
Add a shared top-level runner for:
- parse / validate / run / print fatal error / return exit code
- consistent JSON/text fatal behavior where supported

This likely belongs in a new module rather than overloading `error.rs`.

### `json` / `output`
Add a richer command-output emitter:
- shared success envelope for JSON
- text fallback supplied by the caller
- helper that avoids every tool re-implementing `if json { ... } else { ... }`

### `doctor`
Add provider/build scaffolding:
- a small builder or helper for repo/version/check wiring
- reusable support for extra details, warnings, and errors
- keep state collection local to each tool

### `error`
Collapse the remaining wrapper noise by exposing the final helper surface directly so tools like `todoer` do not need a pass-through local wrapper.

## Proposed Execution Shape

### Wave 1 — expand `cli-common` with the final helper layer
Build the shared runner, richer response emitter, metadata mapping helper, and doctor-provider scaffolding in `cli-common` first, with tests.

### Wave 2 — migrate the thin-entrypoint tools
Refactor `gator`, `bce`, and `todoer` first. They have low-risk remaining boilerplate and give fast feedback on the new helpers.

### Wave 3 — migrate the richer remaining consumers
Use the final shared helpers in `silent-critic`, `unvenv`, `asana-cli`, and `prompter`, where the remaining duplication is mostly success-path branching, fatal runners, and doctor provider scaffolding.

### Wave 4 — enforce and document the thinner boundary
Add repository checks for the newly deleted glue patterns, run the full suite, and update the documented `cli-common` boundary.

## Risks

### Overfitting the runner to one clap shape
Some tools have global `--json`; others keep JSON on specific subcommands. Shared runner helpers must improve consistency without forcing one parser layout everywhere.

### Turning domain output into framework code
Shared response emitters should remove JSON/text branching, but they should not absorb task tables, extraction summaries, or domain-specific multi-line reports.

### Doctor builders becoming macro-heavy
A small builder/helper is sufficient. If the abstraction becomes more complex than the repeated code it replaces, the phase should stop at a lighter shared helper.

## Validation Architecture

### Automated verification
- shared helper changes: `cargo test -p tftio-cli-common && cargo clippy -p tftio-cli-common -- -D warnings`
- migration wave 1: `cargo test -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer && cargo clippy -p tftio-gator -p tftio-bsky-comment-extractor -p tftio-todoer -- -D warnings`
- migration wave 2: `cargo test -p tftio-silent-critic -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter && cargo clippy -p tftio-silent-critic -p tftio-unvenv -p tftio-asana-cli -p tftio-prompter -- -D warnings`
- full contract check: `just cli-metadata-consistency && just cli-consistency && just test && just lint`

### Contract enforcement opportunities
- grep/shell assertions that remaining local metadata-mapping helpers stay deleted where shared helpers replace them
- repository shell checks for shared error/response/doctor behavior
- focused `cli-common` tests for the new runner and output-emitter APIs

## Planning Recommendation

Split the work into four plans:
1. add the final shared helper layer to `cli-common`
2. migrate the thin-entrypoint tools (`gator`, `bce`, `todoer`)
3. migrate the richer remaining consumers (`silent-critic`, `unvenv`, `asana-cli`, `prompter`)
4. finish with enforcement, documentation, and full verification
