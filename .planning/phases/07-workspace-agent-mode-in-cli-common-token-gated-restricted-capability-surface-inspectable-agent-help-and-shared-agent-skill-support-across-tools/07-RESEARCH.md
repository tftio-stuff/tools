# Phase 7: Workspace agent mode in cli-common: token-gated restricted capability surface, inspectable agent help, and shared --agent-skill support across tools - Research

**Researched:** 2026-03-23
**Domain:** Rust workspace CLI architecture (`clap`/`cli-common`) for role-scoped agent surfaces
**Confidence:** MEDIUM-HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

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

### Claude's Discretion
- Exact env var names
- Exact structured-text formatting and section titles for `--agent-help` and `--agent-skill`
- Internal Rust type names for capability declarations and filtering helpers
- Migration sequencing across workspace crates, as long as all `cli-common` consumers are covered in this phase

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Project Constraints (from CLAUDE.md)

- Base CLI UX belongs in `cli-common`. Shared metadata/help/completion/runner behavior should move there instead of staying tool-local.
- Keep crate-local CLI code limited to domain behavior. If a helper is reusable across tools, move it into `cli-common`.
- Update `just cli-metadata-consistency` and `just cli-consistency` when changing binary crates so shared UX remains enforced.
- Prefer existing workspace dependencies via `[workspace.dependencies]` and `dep.workspace = true`. No new crate is justified for this phase yet.
- Workspace MSRV is Rust 1.94.0. The design must compile on that toolchain.
- Workspace lints are strict (`missing_docs = deny`, clippy pedantic) except for crate-local overrides already present in `todoer` and `silent-critic`.
- Nightly is only required for formatting. Verification commands should remain stable-toolchain friendly.
- Python helpers must use `uv run python`, not `python3`.
- No project-local `.claude/skills/` or `.agents/skills/` directories were present.

## Summary

The existing workspace already has the right insertion point for this phase. `tftio-cli-common` owns `ToolSpec`, shared command execution, completion rendering, error/render helpers, and the workspace-wide UX boundary. Every workspace CLI that matters for Phase 7 already depends on `tftio-cli-common`: `bsky-comment-extractor`, `todoer`, `silent-critic`, `prompter`, `asana-cli`, `unvenv`, and `gator`. The current `bce --agent-help` stub is a local proof-of-concept, while `silent-critic` already demonstrates the intended security model: an exogenous token gates a reduced worker-visible surface.

The key planning insight is that this phase is not a documentation feature. It is a parser-surface feature. `clap` already provides the pieces needed to build a filtered command tree (`CommandFactory::command`, mutable command/arg/subcommand editing, `try_get_matches_from_mut`, `FromArgMatches`), and `clap_complete` already generates completions from a mutable `Command`. The safest design is therefore a declarative capability policy in `cli-common` that prunes a `clap::Command` before help rendering, completion generation, and agent-mode parsing.

A hide-only strategy is insufficient for this phase. The full parser can still influence suggestions and error context, which conflicts with D-04 through D-06. The planner should treat this as a shared substrate migration: add a policy model and filtered-command pipeline in `cli-common`, then migrate each consumer onto that path, with special handling for `prompter` because it currently routes through its own `AppMode` parser instead of the workspace-standard runner.

**Primary recommendation:** Extend `tftio-cli-common` with a declarative `AgentSurfacePolicy` and a two-stage `clap` flow: build and prune a `CommandFactory::command()` tree for agent mode, use that filtered tree for `--agent-help`, `--agent-skill`, completions, and pre-parse validation, then dispatch only commands that have already been validated against the filtered surface.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tftio-cli-common` | `0.5.0` | Shared workspace CLI substrate (`ToolSpec`, standard commands, completions, runners) | Already the enforced workspace UX boundary and already depended on by all Phase 7 consumers |
| `clap` | `4.6.0` | Command tree construction, mutation, parsing, help rendering, hidden/visible alias handling | Already the workspace CLI parser; official docs expose the mutable `Command` APIs this phase needs |
| `clap_complete` | `4.6.0` | Shell completion generation from a mutable `Command` | Already used in the workspace and can emit completions from the same filtered command tree used for agent mode |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_json` | `1.0.149` | Preserve existing JSON stderr / JSON mode behavior alongside agent text surfaces | Keep existing structured human/API outputs; do not switch agent help to JSON in this phase |
| Rust `std::env` | `1.94.0` toolchain | Process-start token detection and exact string comparison | Use for the fail-closed presented-token / expected-token contract |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Pruning `clap::Command` in `cli-common` | Building a separate agent-only parser per tool | Duplicates syntax, drifts from the real CLI, and makes `--agent-help`/`--agent-skill` a second source of truth |
| Shared declarative capability policy | Tool-local procedural `if agent_mode { ... }` checks | Fast to start, but fails D-10 through D-12 and guarantees drift across tools |
| `clap_complete` over filtered commands | Handwritten completion templates | High drift risk and no guarantee that completions match the actual allowed surface |

**Installation:**
```bash
# No new dependencies are required by default.
cargo info clap
cargo info clap_complete
```

**Version verification:** Verified against workspace lockfile and current docs/registry metadata on 2026-03-23:
```bash
cargo info clap
cargo info clap_complete
```

## Architecture Patterns

### Recommended Project Structure
```text
crates/cli-common/src/
├── agent.rs         # token gate, policy types, filtered-command builder, help/skill renderers
├── app.rs           # ToolSpec extensions for agent metadata
├── command.rs       # shared pre-parse / shared flag wiring / standard-command integration
├── completions.rs   # render/generate from an already-pruned Command
└── lib.rs           # re-exports

crates/*/src/
├── cli.rs|main.rs   # tool-local capability declarations only
└── tests/           # tool-specific smoke tests for adopted agent surface
```

### Pattern 1: Fail-closed activation at process start
**What:** Agent mode is a derived runtime context, not a flag. Detect it exactly once from the shared presented-token env var and shared expected-token env var before command dispatch.
**When to use:** Every workspace CLI entrypoint, before rendering help, completions, or parsing agent-only flags.
**Example:**
```rust
pub struct AgentModeContext {
    pub active: bool,
}

pub fn detect_agent_mode() -> AgentModeContext {
    let presented = std::env::var(PRESENTED_TOKEN_ENV).ok();
    let expected = std::env::var(EXPECTED_TOKEN_ENV).ok();

    AgentModeContext {
        active: matches!((presented, expected), (Some(p), Some(e)) if p == e),
    }
}
```
**Source:** Phase 7 decisions D-01 through D-03; existing `silent-critic` worker token pattern in `crates/silent-critic/src/commands/session.rs` and `src/main.rs`.

### Pattern 2: Parse against a filtered command tree
**What:** Build the derived parser with `Cli::command()`, prune it according to the active capability policy, parse agent-mode argv against that pruned tree, and only then instantiate the typed CLI.
**When to use:** Agent-mode parsing, `--agent-help`, `--agent-skill`, help rendering, and completion generation.
**Example:**
```rust
use clap::{CommandFactory, FromArgMatches};

let mut cmd = Cli::command();
policy.apply_to_command(&mut cmd, &agent_ctx);

let mut matches = cmd.try_get_matches_from_mut(argv.clone())?;
let cli = Cli::from_arg_matches_mut(&mut matches)?;
```
**Source:** `clap` `CommandFactory`, `Command::try_get_matches_from_mut`, and `FromArgMatches`.

### Pattern 3: Capability-first declaration in `ToolSpec`
**What:** Each tool registers named capabilities/skills with summary text, examples, output semantics, and the command/flag selectors that belong to that capability. The policy allowlists capabilities; it does not hand-pick raw parser nodes ad hoc.
**When to use:** All workspace tools. This is the shared contract that powers agent visibility, `--agent-help`, and `--agent-skill`.
**Example:**
```rust
pub const TOOL_SPEC: ToolSpec = workspace_tool(...)
    .with_agent_surface(AgentSurface::new()
        .capability(
            Capability::new("query-posts")
                .summary("Read paginated post records from local SQLite")
                .command_path(&["query"])
                .flag("db")
                .flag("limit")
                .flag("offset")
                .example("bce query --limit 25 --offset 50"),
        ));
```
**Source:** Existing `ToolSpec` ownership in `crates/cli-common/src/app.rs`; Phase 7 decisions D-10 through D-12.

### Pattern 4: Derive help and skill text from the same policy object
**What:** `--agent-help` renders the top-level visible manifest; `--agent-skill <name>` renders the single-capability view. Both come from the same policy declarations that filtered the parser.
**When to use:** Always. Manual strings should disappear except for tool-local summaries/examples stored in the policy declaration itself.
**Example:**
```rust
println!("{}", render_agent_help(&tool_spec, &policy, &agent_ctx));
println!("{}", render_agent_skill(&tool_spec, &policy, &agent_ctx, skill_name)?);
```
**Source:** Phase 7 decisions D-07 through D-09; `silent-critic` manifest concept in `docs/the_silent_critic_tooling_design.md`.

### Anti-Patterns to Avoid
- **Hide-only implementation:** `#[arg(hide = true)]` or `Command::hide` on the full parser is not enough for D-04 through D-06 because the larger parser can still affect completions, suggestions, or error context.
- **Tool-local manual help blobs:** A hard-coded `print_agent_help()` stub will drift from real parser behavior.
- **Raw parser-fragment allowlists:** If policy is attached only to flags/subcommands and not to named capabilities, `--agent-skill` becomes impossible to keep coherent.
- **Mode env without token equality:** Any design that treats “agent mode env var is present” as sufficient violates D-01 through D-03.
- **Permanent special cases:** `bce` and `prompter` need migration seams, not permanent one-off agent implementations.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Agent-mode parser | A second bespoke parser per tool | Pruned `clap::Command` trees from `CommandFactory::command()` | Keeps syntax/help/errors/completions aligned with the real CLI |
| Shell completions | Manual completion templates | `clap_complete::generate` over the already-filtered command tree | One source of truth for visible subcommands and flags |
| Agent help/skill docs | Static markdown strings unrelated to CLI definitions | Renderers over declarative capability policy | Eliminates drift and satisfies D-08/D-09 |
| Capability security boundary | Per-tool `match` statements with ad hoc denies | Shared declarative allowlist policy in `cli-common` | Centralizes enforcement and enables workspace rollout |
| Token gate | Multi-step handshake or writable local state | Exact env-var compare at process start | Satisfies fail-closed activation with minimal moving parts |

**Key insight:** The hard part in this domain is not text rendering. It is keeping parser behavior, discoverability, and self-description identical. Reuse `clap` as the canonical parser tree and derive everything else from that filtered tree.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `silent-critic` SQLite stores `worker_token` / `operator_token`, but no workspace CLI stores agent-help policy or env-var names. | Code edit only if Phase 7 reuses those token values; no data migration identified. |
| Live service config | None found in repo or planning artifacts. No UI-owned external service configuration tied to `--agent-help` or workspace agent-mode names was identified. | None. |
| OS-registered state | None found. | None. |
| Secrets/env vars | `SILENT_CRITIC_TOKEN` is the only current agent token env var in repo. Phase 7 adds one shared presented-token env var plus one shared expected-token env var across tools. Orchestrators and prompts that currently export only `SILENT_CRITIC_TOKEN` will need updating. | Code/config edit and orchestration update; no stored-data migration. |
| Build artifacts | Installed binaries, generated shell completions, and generated help/manpage artifacts can retain the old surface until rebuilt. `just cli-consistency` currently references an external script path that is missing in this checkout. | Rebuild/reinstall binaries, regenerate completions/manpage artifacts, and replace or vendor the missing validation script. |

## Common Pitfalls

### Pitfall 1: The full parser still leaks the hidden surface
**What goes wrong:** Help looks filtered, but invalid-command suggestions, accepted values, or completion output still reflect the larger parser.
**Why it happens:** The implementation hides text instead of pruning the actual command tree used for parse/help/completion.
**How to avoid:** Filter the `Command` first, then use that exact filtered command for parse, `--agent-help`, `--agent-skill`, and completions.
**Warning signs:** Error output includes “did you mean …” suggestions for disallowed commands or flags.

### Pitfall 2: Capability docs drift from parser behavior
**What goes wrong:** `--agent-help` says one thing while parse errors or accepted flags say another.
**Why it happens:** Manual documentation strings live outside the command/policy source of truth.
**How to avoid:** Make capability declarations the input for both parser filtering and agent help rendering.
**Warning signs:** Tests need to assert on copied strings instead of rendered metadata.

### Pitfall 3: Consumer CLI shapes differ more than they first appear
**What goes wrong:** A shared helper fits `bce` and `gator`, then breaks on `prompter` or a tool using top-level metadata commands.
**Why it happens:** Workspace consumers are heterogeneous: `todoer`/`gator` use `meta`, `asana-cli`/`unvenv` use top-level metadata commands, `prompter` has a custom `AppMode`, and `bce` currently has a hidden top-level flag stub.
**How to avoid:** Plan explicit migration seams per consumer. Shared policy and rendering live in `cli-common`, but adoption glue will differ.
**Warning signs:** The design assumes every tool already uses `parse_and_exit` or `maybe_run_standard_command`.

### Pitfall 4: Completion generation bypasses the filtered surface
**What goes wrong:** Agent help is correct, but shell completions still expose hidden flags because completions instantiate the unfiltered parser internally.
**Why it happens:** Current `cli-common::render_completion<T>` and `generate_completions<T>` always call `T::command()` themselves.
**How to avoid:** Add `render_completion_from_command` / `generate_completions_from_command` helpers that accept an already-pruned `Command`.
**Warning signs:** Tests can only validate help, not completion scripts, for redaction.

### Pitfall 5: Workspace validation is already partially broken on this branch
**What goes wrong:** The planner assumes “run the workspace suite” is enough, but current smoke infrastructure does not fully work.
**Why it happens:** `just cli-consistency` references `/Users/jfb/Projects/tools/feature-add-agent-help-to-all-tools/scripts/test-cli-metadata-consistency.sh`, which is missing here, and `cargo run -p tftio-bsky-comment-extractor` currently fails because `crates/bsky-comment-extractor/src/main.rs` has no `fn main()`.
**How to avoid:** Budget a Wave 0 validation repair step or scope Phase 7 checks to crates that currently compile until the branch is stabilized.
**Warning signs:** `just cli-consistency` or `cargo run -p tftio-bsky-comment-extractor -- --help` fails before any Phase 7 code changes.

## Code Examples

Verified patterns from official sources:

### Build, prune, and parse a command tree
```rust
use clap::{CommandFactory, FromArgMatches};

let mut cmd = Cli::command();
policy.apply_to_command(&mut cmd, &agent_ctx);

let mut matches = cmd.try_get_matches_from_mut(argv.clone())?;
let cli = Cli::from_arg_matches_mut(&mut matches)?;
```
// Source: `clap` `CommandFactory`, `Command::try_get_matches_from_mut`, `FromArgMatches`

### Generate completions from the already-filtered command
```rust
use clap_complete::{generate, Shell};

let mut cmd = Cli::command();
policy.apply_to_command(&mut cmd, &agent_ctx);

generate(
    Shell::Bash,
    &mut cmd,
    cmd.get_name().to_string(),
    &mut std::io::stdout(),
);
```
// Source: `clap_complete::generate`

### Shared fail-closed activation helper
```rust
pub fn is_agent_mode_active() -> bool {
    matches!(
        (
            std::env::var(PRESENTED_TOKEN_ENV).ok(),
            std::env::var(EXPECTED_TOKEN_ENV).ok(),
        ),
        (Some(presented), Some(expected)) if presented == expected
    )
}
```
// Source: Phase 7 decisions D-01 through D-03

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tool-local `bce --agent-help` stub | Shared agent surface in `cli-common` derived from declarative capability policy | Planned in Phase 7 | Eliminates one-off help implementations and enables workspace rollout |
| `silent-critic`-only worker token env | Workspace-shared presented-token + expected-token contract | Planned in Phase 7 | Makes the visibility boundary reusable across all workspace tools |
| “Hide it in help” as the boundary | Filter the actual parser/help/completion surface | Current best approach with `clap` 4.6 APIs | Required to satisfy D-04 through D-06 |

**Deprecated/outdated:**
- `bce`’s hard-coded `print_agent_help()` stub in `crates/bsky-comment-extractor/src/main.rs`
- Any implementation that relies on `#[arg(hide = true)]` alone as the security boundary
- The external `cli-metadata-consistency` script path currently referenced by `justfile`

## Open Questions

1. **What exact env var names should the workspace use?**
   - What we know: The names are explicitly left to discretion, but they must be shared across all tools and require exact equality at process start.
   - What's unclear: Whether naming should remain generic to the whole workspace or mention “agent” explicitly for operator ergonomics.
   - Recommendation: Pick names once in `cli-common`, document them in the shared renderer/tests, and avoid per-tool aliases.

2. **Should typed dispatch use `FromArgMatches` or a second parse of the same argv?**
   - What we know: `clap` supports both filtered-command parsing and typed reconstruction from `ArgMatches`.
   - What's unclear: Which approach produces the least migration friction across all consumer shapes.
   - Recommendation: Validate argv against the filtered command first. Then either instantiate via `FromArgMatches` where easy, or re-run the normal typed parser only after the filtered parse has already succeeded.

3. **Should standard metadata commands be visible in agent mode?**
   - What we know: D-04 requires strong hiding of disallowed functionality, but does not require every existing metadata command to remain visible.
   - What's unclear: Whether commands like `version`, `license`, `doctor`, `update`, or `completions` add value to agent mode.
   - Recommendation: Default them to hidden in agent mode unless a tool declares a concrete capability for them.

4. **How should `prompter` adopt the shared path?**
   - What we know: `prompter` currently routes through a custom `AppMode` parser instead of the workspace-standard shared runner path.
   - What's unclear: Whether Phase 7 should add an adapter layer or do a broader parser refactor there.
   - Recommendation: Plan an explicit `prompter` migration task. Do not assume it is a mechanical copy of `bce`/`gator`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build, tests, crate metadata verification | ✓ | `1.94.0` | — |
| `rustc` | Compile / lint / test | ✓ | `1.94.0` | — |
| `just` | Workspace validation recipes | ✓ | `1.40.0` | Run underlying `cargo` commands directly |
| `/Users/jfb/Projects/tools/feature-add-agent-help-to-all-tools/scripts/test-cli-metadata-consistency.sh` | `just cli-consistency` | ✗ | — | Replace with an in-repo smoke script or direct `cargo run` assertions |

**Missing dependencies with no fallback:**
- None.

**Missing dependencies with fallback:**
- The external metadata-consistency script referenced by `just cli-consistency`; use direct `cargo run` smoke commands until the script is vendored or replaced.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Cargo workspace unit + integration tests |
| Config file | none — Cargo defaults plus `justfile` recipes |
| Quick run command | `cargo test -p tftio-cli-common --lib` |
| Full suite command | `cargo test --workspace --verbose` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01..D-03 | Agent mode activates only on exact token match and otherwise fails closed | unit | `cargo test -p tftio-cli-common agent_mode_activation --lib` | ❌ Wave 0 |
| D-04..D-06 | Hidden commands/flags stay absent from help, completions, suggestions, and parse errors | unit + integration | `cargo test -p tftio-cli-common agent_surface_redaction --lib` | ❌ Wave 0 |
| D-07..D-09 | Shared `--agent-help` / `--agent-skill` render only the visible manifest | unit | `cargo test -p tftio-cli-common agent_help_render --lib` | ❌ Wave 0 |
| D-10..D-12 | Capability declarations drive filtering and skill lookup | unit | `cargo test -p tftio-cli-common capability_policy --lib` | ❌ Wave 0 |
| D-13..D-14 | All `cli-common` consumers adopt the shared substrate | integration / smoke | `cargo test --workspace --verbose` plus `just cli-consistency` | ⚠️ Partial — current smoke path is broken |

### Sampling Rate
- **Per task commit:** `cargo test -p tftio-cli-common --lib`
- **Per wave merge:** `cargo test --workspace --verbose`
- **Phase gate:** Full suite green plus restored `just cli-consistency` before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Add shared `cli-common` agent-mode unit tests (activation, policy filtering, help rendering).
- [ ] Add completion-redaction tests that operate on filtered commands, not just help text.
- [ ] Add one smoke test per consumer verifying `--agent-help`, `--agent-skill`, and rejection of hidden commands.
- [ ] Replace or vendor the missing external `cli-metadata-consistency` script used by `just cli-consistency`.
- [ ] Restore `crates/bsky-comment-extractor/src/main.rs` entrypoint so workspace-level run/smoke validation can execute on this branch.

## Sources

### Primary (HIGH confidence)
- Local repo: `crates/cli-common/src/app.rs`, `src/command.rs`, `src/completions.rs`, `src/lib.rs`
- Local repo: `crates/bsky-comment-extractor/src/cli.rs`, `src/main.rs`
- Local repo: `crates/silent-critic/src/commands/session.rs`, `src/main.rs`, `src/cli.rs`
- Local repo: workspace `Cargo.toml`, per-crate `Cargo.toml` files, `justfile`, `.planning/*.md`, and Silent Critic design/spec docs
- Official docs: `clap` `Command`, `Arg`, `CommandFactory`, `FromArgMatches`
- Official docs: `clap_complete`

### Secondary (MEDIUM confidence)
- `cargo info clap`
- `cargo info clap_complete`

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — existing workspace stack plus official `clap` / `clap_complete` docs already support the needed approach
- Architecture: **MEDIUM-HIGH** — shared direction is clear, but migration details differ across `prompter`, `bce`, and the meta-command tools
- Pitfalls: **HIGH** — phase decisions are explicit and current branch validation gaps were directly observed

**Research date:** 2026-03-23
**Valid until:** 2026-04-22
