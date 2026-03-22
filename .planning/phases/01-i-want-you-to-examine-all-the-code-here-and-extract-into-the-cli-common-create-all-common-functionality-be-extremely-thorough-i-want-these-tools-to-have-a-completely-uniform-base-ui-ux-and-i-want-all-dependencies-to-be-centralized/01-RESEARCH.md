# Phase 1 Research — CLI Common Unification

**Gathered:** 2026-03-22
**Status:** Complete
**Scope:** Entire Cargo workspace under `crates/`

## Question

What has to move into `cli-common`, what should stay crate-specific, and how should the work be split so every CLI in this workspace converges on one consistent base UI/UX?

## Current State Inventory

### `cli-common` today

`cli-common` already owns a small shared layer:
- shell completions generation
- doctor trait + runner
- license text rendering
- tty-aware output helpers
- self-update launcher
- shared `RepoInfo` and `DoctorCheck` types

It does **not** yet own the base application contract for these tools. There is no shared layer for:
- standard metadata commands and wiring
- consistent error rendering
- consistent JSON response envelopes
- progress spinner construction
- stdout/stderr style policy
- shared top-level app metadata / repo metadata contract
- shared testable conventions for help/version/license/completions/doctor/update

### Repeated patterns by crate

| Crate | Repeated / divergent CLI behavior |
|------|------------------------------------|
| `unvenv` | Inline clap enum for `Version`, `License`, `Completions`, `Doctor`, `Update`; inline tty/color behavior in `main.rs` |
| `bsky-comment-extractor` | Inline spinner construction, text error rendering, completion summary formatting |
| `gator` | Inline plain-text vs JSON error printing; no shared response envelope |
| `todoer` | Duplicated JSON envelope helpers in `src/output.rs`; command-specific text output in `main.rs` |
| `silent-critic` | Same duplicated JSON envelope helpers as `todoer`; separate error formatting path |
| `prompter` | Custom `AppMode` dispatcher, custom doctor implementation, custom completion instruction wrapper, inline tty formatting |
| `asana-cli` | Closest to a standard meta-command surface, but still wires doctor/update/license/completions manually and prints debug-style errors from `main.rs` |

### Dependency duplication that should collapse behind `cli-common`

The root `Cargo.toml` already centralizes versions. The remaining problem is **behavioral centralization**. Multiple crates still need direct access to crates that could be hidden behind `cli-common` APIs:
- `clap_complete`
- `colored`
- `is-terminal`
- `indicatif`
- `serde_json`
- parts of `thiserror`-style presentation logic

This does **not** mean every dependency disappears from every crate. It means shared UX behavior stops being re-implemented independently.

## Key Findings

### 1. Uniformity gap is mostly entrypoint and rendering code

Most divergence sits in:
- `src/main.rs`
- `src/cli.rs` / `src/cli/mod.rs`
- per-crate `output.rs` helpers
- custom doctor/completions glue

Core domain code is mostly unrelated and should remain crate-specific.

### 2. Two crates duplicate the exact same JSON envelope shape

`todoer/src/output.rs` and `silent-critic/src/output.rs` both expose:
- `ok_response(command, data)`
- `err_response(command, code, message, details)`

That belongs in `cli-common` unchanged so these crates can delete the duplicate modules.

### 3. `prompter` and `asana-cli` need adapter-style migration, not brute-force replacement

Those two crates have richer CLI behavior:
- `prompter` has dynamic completion augmentation and an `AppMode` abstraction
- `asana-cli` has a broad multi-command tree, manpage generation, tracing initialization, and domain-specific command handlers

They still need the shared base contract, but the migration should preserve crate-specific command trees.

### 4. `bsky-comment-extractor` and `unvenv` are ideal proving grounds for shared text UX

They are smaller surfaces with obvious repeated behavior:
- tty detection
- progress display / summary formatting
- license / completions / doctor / update wiring
- consistent error prefixing and exit codes

### 5. `gator`, `todoer`, and `silent-critic` are the proving grounds for machine-readable UX

These tools already expose JSON-oriented behavior or agent-facing output. They need:
- one shared response envelope
- one shared error printer that can render either plain text or JSON
- one consistent metadata-command surface

## Recommended `cli-common` Target Surface

`cli-common` should grow into a small CLI platform crate with these modules and responsibilities:

### `app`
Shared tool metadata contract.

Proposed types:
- `ToolSpec` with exact fields for `bin_name`, `display_name`, `version`, `license`, `repo_owner`, `repo_name`, `supports_json`, `supports_update`, `supports_doctor`
- helper constructors for standard binary metadata

### `command`
Shared metadata command plumbing.

Proposed types/functions:
- `StandardCommand`
- `run_standard_command::<Cli, D>(...)`
- helpers for version, license, completions, doctor, update

### `json`
Shared machine-readable envelope.

Required exact payload shape:
```json
{"ok":true,"command":"...","data":{...}}
```
```json
{"ok":false,"command":"...","error":{"code":"...","message":"...","details":{}}}
```

### `error`
Shared error presentation.

Required functions:
- plain text printer with consistent prefix rules
- JSON printer backed by `json::err_response`
- shared exit-code helper for top-level `main.rs` / `run()` wrappers

### `progress`
Shared spinner builder and tty-safe progress behavior.

Required behavior:
- return `None` when disabled or non-TTY
- single consistent spinner template for workspace CLIs
- stderr draw target by default

### `output`
Retain the existing tty-aware helpers, but expand to include:
- stdout/stderr tty checks
- headline/banner helpers without hard-coded emojis in non-TTY mode
- consistent success/warn/info formatting primitives used by all tools

## What Should Stay Crate-Specific

These do **not** belong in `cli-common`:
- Asana domain commands and renderers
- BlueSky extraction logic and DB path rules
- Prompter’s profile graph resolution and dynamic completion data source
- Gator sandbox/session logic
- Todoer project resolution and SQLite schema
- Silent Critic’s supervision domain and contract/session models

The rule is:
- **Base UX contract in `cli-common`**
- **Domain behavior in the crate**

## Migration Strategy

### Wave 1 — build the shared contract first

Expand `cli-common` before touching consumers:
1. add metadata / standard-command layer
2. add shared JSON envelope and error printer
3. add shared spinner / progress helpers
4. add tests that lock the contract down

### Wave 2 — migrate low-risk consumers in parallel

Parallel branch A:
- `gator`
- `todoer`
- `silent-critic`

Parallel branch B:
- `unvenv`
- `bsky-comment-extractor`

Reason: different files, different UX classes, minimal conflict.

### Wave 3 — migrate adapter-heavy consumers

Move `prompter` and `asana-cli` once the shared primitives are stable.

### Wave 4 — enforce consistency workspace-wide

Add verification that every binary crate now exposes the same base behavior where applicable:
- version
- license
- completions
- doctor
- update (opt-in only if supported)
- plain text errors
- JSON envelope for machine-readable tools

## Risks

### Breaking CLI compatibility

Risk: forcing identical parser structures across all tools could break existing command syntax.

Mitigation:
- unify the base commands and renderers
- preserve each crate’s domain-specific command tree
- prefer adapters over parser rewrites where syntax stability matters

### Over-centralizing domain output

Risk: moving table rendering, API-specific formatting, or domain summaries into `cli-common` would make the shared crate bloated.

Mitigation:
- only centralize primitives and envelopes
- leave domain tables and rich renderers local

### Hidden dependency churn

Risk: moving behavior to `cli-common` changes crate-level dependency needs and test imports.

Mitigation:
- migrate one crate class at a time
- remove duplicate local modules only after replacement compiles
- verify per-crate tests after each migration slice

## Testing Implications

The phase should not rely on one final workspace test run only. It needs staged verification:
- `cli-common` unit tests first
- per-crate CLI / output tests after each migration
- workspace `just test` + `just lint` after the last migration wave

Manual checks are still required for:
- `--help` output readability
- completion scripts emitting the expected header/instructions
- plain text vs JSON error mode behavior

## Validation Architecture

### Automated sampling
- quick contract checks after each task: focused `cargo test -p <crate>` on the crate being migrated
- full contract checks after each wave: `just test` then `just lint`
- final contract check: `just ci`

### Manual contract checks
- run `--help` for each binary crate and compare heading / metadata-command presence
- run JSON mode for `gator`, `todoer`, and `silent-critic` to verify the shared envelope shape
- run completion generation for `prompter`, `asana-cli`, and one simple tool to confirm wrapper behavior still holds

## Planning Recommendation

Split the implementation into four plans:
1. build the shared `cli-common` platform surface
2. migrate machine-oriented CLIs (`gator`, `todoer`, `silent-critic`)
3. migrate user-oriented CLIs (`unvenv`, `bsky-comment-extractor`, `asana-cli`)
4. finish `prompter`, remove remaining duplication, and add workspace-wide consistency verification

That split keeps the critical shared write-set in one plan, lets consumer migrations proceed with low overlap, and reserves the most custom CLI (`prompter`) for the end when the shared contract is stable.
