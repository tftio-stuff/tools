# Phase 7: Add shared --agent-help and --agent-skill support across CLI crates - Research

**Researched:** 2026-03-22
**Domain:** Rust CLI architecture for shared agent-facing documentation across a Cargo workspace
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Claude's Discretion
- Exact Rust API shape in `cli-common` for representing shared agent-help and agent-skill payloads
- How to minimize duplication between YAML reference output and skill-document rendering
- How to integrate shared flag handling into crates with different clap entrypoint structures
- Test layout and helper abstractions, as long as every binary crate ends with the required flags
  and the documented behavior above

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

## Summary

Phase 7 is not primarily a clap-flag task. It is a shared documentation-architecture task with
CLI wiring at the edges. The hard part is producing one exhaustive source of truth per tool and
rendering it two ways: canonical YAML for `--agent-help`, and a Claude-style skill file for
`--agent-skill`. The workspace already shows why a naive rollout will fail: `bsky-comment-extractor`
currently uses a hidden `global = true` clap flag, and `bce query --agent-help` succeeds today,
which violates the new top-level-only requirement. Several other crates also require a subcommand
or positional argument, so making these flags “just another clap arg” would force parser shape
changes across multiple tools.

The planning-safe approach is: add a shared `cli-common::agent_docs` module with a typed document
model, shared renderers, and a raw-argv detector for the exact top-level invocations. Each binary
crate then supplies its own `AgentDoc` builder and a small early-exit hook in `main` or equivalent
entrypoint. Use clap reflection for validation, not for full document generation: clap can tell you
what commands and arguments exist, but it cannot author exhaustive examples, output contracts,
environment variables, config defaults, or likely operator mistakes.

**Primary recommendation:** Plan around one shared `AgentDoc` model plus a shared top-level argv
detector in `cli-common`, with per-crate authored content and clap-backed coverage tests.

## Project Constraints (from CLAUDE.md)

- Use the Cargo workspace dependency pattern: external dependencies belong in root
  `Cargo.toml` under `[workspace.dependencies]`.
- Keep `cli-common` as a library crate.
- Respect workspace lints: `missing_docs = "deny"` at workspace level, `clippy::all = "deny"`,
  `clippy::pedantic = "deny"`, `wildcard_imports = "deny"`, `enum_glob_use = "deny"`.
- Follow existing crate structure patterns: `main.rs` entrypoint, optional `cli.rs` or
  `cli/mod.rs`, unit tests in `src/`, integration tests in `tests/`.
- Preferred verification commands are workspace `cargo test`, `cargo clippy`, and `just` recipes.
- Toolchain target is Rust 1.94.0 MSRV/workspace toolchain.
- Use `uv run python` for Python-related tasks.
- Workspace reality differs from the stale crate table in `CLAUDE.md`: current `Cargo.toml`
  members include `gator` and `bsky-comment-extractor`, and Phase 7 scope must follow the
  workspace plus `07-CONTEXT.md`, not the outdated summary table.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tftio-cli-common` | `0.5.0` | Shared home for agent-doc model, rendering, and detection helpers | Matches workspace convention that cross-crate CLI behavior lives here |
| `clap` | workspace `4.x` | Existing parser and reflection layer for all binaries | Already used by every binary crate; supports command/arg reflection and help rendering |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `std` (`std::env`, `std::ffi`, `std::io`) | toolchain | Raw argv detection and stdout emission | Use for exact top-level flag interception before normal clap parsing |
| Existing per-crate test harnesses (`std::process::Command`, `env!("CARGO_BIN_EXE_*")`) | existing | End-to-end CLI assertions | Use for top-level invocation and hidden-help regression tests |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Shared raw-argv detection in `cli-common` | Hidden clap args on every parser | Hidden clap args are fine for optional top-level parsers, but they collide with required subcommands/positionals and `global = true` permits subcommand placement |
| Shared typed Rust model + two renderers | Two independent static strings per tool | Faster to start, but drift between YAML and skill output becomes likely immediately |
| No new YAML dependency | `serde_yaml` / `serde_yml` | `serde_yaml` is deprecated; `serde_yml` is young and its latest docs.rs build is failing, so adding YAML-specific dependency risk is unnecessary for this phase |

**Installation:**

```bash
# Preferred Phase 7 plan: no new third-party dependencies required
# Extend existing workspace crates only
```

**Version verification:**
- No new third-party dependency is required for the recommended plan.
- If the plan changes and adds YAML serialization later, re-verify before implementation;
  current Rust YAML crate options are not strong enough to assume by default.

## Architecture Patterns

### Recommended Project Structure

```text
crates/
├── cli-common/
│   └── src/
│       ├── agent_docs.rs     # shared model, renderers, argv detection, test helpers
│       └── lib.rs            # re-export agent_docs API
├── prompter/src/             # per-crate agent doc builder + early exit wiring
├── unvenv/src/
├── asana-cli/src/
├── todoer/src/
├── silent-critic/src/
├── gator/src/
└── bsky-comment-extractor/src/
```

### Pattern 1: Early top-level interception before clap parsing
**What:** Detect only the exact invocations `tool --agent-help` and `tool --agent-skill` before
normal CLI parsing, emit stdout, and exit `0`.

**When to use:** All seven binaries, especially crates with required subcommands or required
positionals (`prompter`, `asana-cli`, `todoer`, `silent-critic`, `gator`).

**Why this pattern:** It preserves current parser semantics for normal usage and avoids weakening
existing required subcommands or required positional arguments just to accommodate agent docs.

**Example:**
```rust
use tftio_cli_common::agent_docs::{detect_request, AgentDocRequest};

fn main() {
    let args = std::env::args_os().collect::<Vec<_>>();
    if let Some(request) = detect_request(&args) {
        let doc = build_agent_doc();
        match request {
            AgentDocRequest::Help => print!("{}", doc.render_yaml()),
            AgentDocRequest::Skill => print!("{}", doc.render_skill()),
        }
        return;
    }

    // existing clap flow stays unchanged
    run_normal_cli();
}
```

### Pattern 2: One typed document model, two renderers
**What:** Define a shared `AgentDoc` model in `cli-common` that captures the canonical data once,
then render it as YAML or as a skill file.

**When to use:** Every tool.

**Why this pattern:** D-04 and D-05 require both outputs to represent the same underlying content.
The phase should make divergence structurally difficult.

**Example:**
```rust
pub struct AgentDoc {
    pub tool: &'static str,
    pub summary: &'static str,
    pub commands: Vec<AgentCommand>,
    pub env_vars: Vec<AgentEnvVar>,
    pub config_files: Vec<AgentConfigFile>,
    pub examples: Vec<AgentExample>,
    pub failure_modes: Vec<AgentFailureMode>,
    pub operator_mistakes: Vec<AgentMistake>,
}

impl AgentDoc {
    pub fn render_yaml(&self) -> String { /* shared renderer */ }
    pub fn render_skill(&self) -> String { /* shared skill renderer */ }
}
```

### Pattern 3: Use clap reflection for coverage validation, not authoring
**What:** Build a helper that inspects `Cli::command()` and verifies documented subcommands and
arguments still match the real parser tree.

**When to use:** In `cli-common` unit tests and in per-crate doc coverage tests.

**Why this pattern:** clap can enumerate commands and arguments, which is enough to catch drift,
but clap metadata is not rich enough to auto-author exhaustive agent documentation.

**Example:**
```rust
use clap::CommandFactory;

let mut cmd = Cli::command();
let documented = build_agent_doc();
let actual_subcommands = cmd.get_subcommands().map(|sub| sub.get_name()).collect::<Vec<_>>();

assert!(actual_subcommands.iter().all(|name| documented.has_command(name)));
```

### Pattern 4: Keep shared sections narrow
**What:** Share only sections that are genuinely identical across tools: output conventions for
shared `license`, `doctor`, `completions`, or update behavior if the crate actually exposes them.

**When to use:** Only where the runtime behavior is truly common.

**Why this pattern:** D-03 explicitly requires tool-specific commands, config, env, and errors to
remain per-crate.

### Anti-Patterns to Avoid
- **`global = true` for agent-doc flags:** This makes the flag valid inside subcommands. Current
  `bce query --agent-help` already succeeds, so this does not satisfy D-07.
- **Fully auto-generated docs from clap metadata:** clap cannot supply output contracts, edge-case
  examples, config defaults, env vars, or operator mistakes.
- **Separate authored YAML and skill files:** They will drift.
- **Making every subcommand optional just to fit the new flags:** This changes normal CLI UX and
  error behavior.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Shared flag behavior across 7 binaries | Seven ad-hoc `if args.contains("--agent-help")` implementations | One `cli-common::agent_docs::detect_request` helper | Keeps semantics identical across crates |
| YAML + skill duplication | Two independent documents per crate | One shared `AgentDoc` model + two renderers | D-04 requires same underlying information |
| Command/flag coverage checking | Manual test lists that duplicate clap tree | clap `CommandFactory` + `get_subcommands()` / `get_arguments()` validation helper | Catch parser/doc drift automatically |
| Exhaustive doc generation | Custom parser traversal that tries to infer runtime semantics | Per-crate authored semantics plus reflection-backed validation | Runtime behavior, env, config, and failure guidance are not recoverable from clap alone |

**Key insight:** The phase should hand-author semantics once per tool, but it should not
hand-author the shared mechanics or drift checks seven times.

## Common Pitfalls

### Pitfall 1: Assuming hidden clap globals are “top-level only”
**What goes wrong:** The flag works under subcommands.
**Why it happens:** `global = true` propagates to child command contexts.
**How to avoid:** Use exact top-level argv interception for behavior; use clap reflection only for
validation.
**Warning signs:** `tool subcommand --agent-help` exits `0`.

### Pitfall 2: Changing parser shapes to make flags parse
**What goes wrong:** Tools that used to require a subcommand or positional argument suddenly accept
empty invocations or different error paths.
**Why it happens:** The easiest way to parse `--agent-help` via clap is often to make required
fields optional.
**How to avoid:** Short-circuit before clap for the agent-doc invocations.
**Warning signs:** Help text, usage lines, or missing-subcommand exit behavior changes in unrelated
paths.

### Pitfall 3: Treating clap help text as exhaustive agent help
**What goes wrong:** The output misses config files, default paths, output schemas, failure modes,
or operator mistakes.
**Why it happens:** clap only knows parser metadata.
**How to avoid:** Require per-crate doc builders to fill semantic sections explicitly.
**Warning signs:** The generated output looks like reformatted `--help`.

### Pitfall 4: YAML and skill drift
**What goes wrong:** `--agent-help` and `--agent-skill` disagree on commands, examples, or errors.
**Why it happens:** Two render paths own separate content.
**How to avoid:** Render both from the same `AgentDoc`.
**Warning signs:** Fixes applied to only one output format.

### Pitfall 5: Forgetting hidden-help regressions
**What goes wrong:** `--agent-help` or `--agent-skill` leaks into normal `--help`.
**Why it happens:** New args get added to clap without `hide`, or custom help tests are missing.
**How to avoid:** Add explicit `--help` / subcommand-help assertions in every binary.
**Warning signs:** Snapshot-like help output changes without targeted test coverage.

## Code Examples

Verified patterns from official clap docs and local workspace patterns:

### Flatten shared args into a top-level parser when it does not change normal semantics
```rust
#[derive(clap::Args, Debug, Default)]
pub struct AgentDocArgs {
    #[arg(long, hide = true)]
    pub agent_help: bool,

    #[arg(long, hide = true)]
    pub agent_skill: bool,
}

#[derive(clap::Parser, Debug)]
pub struct Cli {
    #[command(flatten)]
    pub agent_docs: AgentDocArgs,
}
```

### Reflect the clap tree in tests
```rust
use clap::CommandFactory;

let mut cmd = Cli::command();
let subcommands = cmd
    .get_subcommands()
    .map(|sub| sub.get_name().to_owned())
    .collect::<Vec<_>>();

let top_level_longs = cmd
    .get_arguments()
    .filter_map(|arg| arg.get_long().map(str::to_owned))
    .collect::<Vec<_>>();
```

### Render shared skill front matter from the same model
```rust
fn render_skill(doc: &AgentDoc) -> String {
    format!(
        "---\nname: {}\ndescription: {:?}\n---\n\n# {}\n\n{}",
        doc.tool,
        doc.summary,
        doc.tool,
        render_markdown_body(doc),
    )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| One-off placeholder string in a single crate | Shared agent-doc model with per-tool builders | Phase 7 | Scales to all binaries |
| Hidden `global = true` clap flag | Exact top-level interception plus validation helpers | Phase 7 | Satisfies top-level-only requirement |
| Separate documentation formats | One source model rendered twice | Phase 7 | Reduces drift |
| YAML serializer as default assumption | No YAML dependency unless truly needed | 2024-2026 ecosystem state | Avoids deprecated or immature YAML crates |

**Deprecated/outdated:**
- `serde_yaml` as the default YAML choice: the crate is explicitly marked deprecated/no longer
  maintained.
- “Just reuse `--help`”: this is insufficient for D-02.

## Open Questions

1. **What is the exact canonical YAML schema and field ordering?**
   - What we know: D-01 and D-02 require one canonical exhaustive shape.
   - What's unclear: Final field names, ordering, and whether long prose belongs in scalars,
     arrays, or nested objects.
   - Recommendation: Freeze the schema in Plan Wave 0 before writing crate content.

2. **Where should large tool-specific prose live?**
   - What we know: Every crate needs substantial tool-specific content.
   - What's unclear: Inline Rust builders vs `include_str!` markdown/YAML fragments under each
     crate.
   - Recommendation: Prefer Rust builders for structured fields and `include_str!` only for long
     markdown sections if readability becomes a problem.

3. **Should hidden clap args exist at all if early interception handles behavior?**
   - What we know: Early interception is the safest way to preserve normal parser semantics.
   - What's unclear: Whether the team wants clap to “know about” these flags for internal
     validation or future completion tooling.
   - Recommendation: Do not require clap-visible flags in Phase 7; prioritize correct runtime
     behavior first.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/test/CLI integration checks | ✓ | `1.94.0` | — |
| `rustc` | Workspace compilation | ✓ | `1.94.0` | — |
| `just` | Preferred workspace verification commands | ✓ | `1.40.0` | Run underlying `cargo` commands directly |

**Missing dependencies with no fallback:**
- None

**Missing dependencies with fallback:**
- None

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` with unit + integration tests |
| Config file | none — workspace conventions only |
| Quick run command | `cargo test -p tftio-cli-common agent_docs --lib` |
| Full suite command | `just test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 / D-04 / D-05 | Shared model renders canonical YAML and skill output from same source | unit | `cargo test -p tftio-cli-common agent_docs --lib` | ❌ Wave 0 |
| D-02 / D-03 | Per-crate docs include exhaustive tool-specific sections | integration | `cargo test -p tftio-bsky-comment-extractor agent_help` | ❌ Wave 0 |
| D-06 / D-07 / D-08 | Flags are hidden, top-level only, stdout success, exit `0` | integration | `cargo test -p tftio-gator agent_help` | ❌ Wave 0 |
| D-09 / D-10 | All seven binaries expose required behavior | integration | `cargo test --workspace agent_help` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p tftio-cli-common agent_docs --lib`
- **Per wave merge:** `cargo test --workspace agent_help`
- **Phase gate:** `just test`

### Wave 0 Gaps
- [ ] `crates/cli-common/src/agent_docs.rs` — shared model, renderer, detection, and unit tests
- [ ] `crates/prompter/tests/agent_help.rs` — top-level `--agent-help` / `--agent-skill` behavior
- [ ] `crates/unvenv/tests/agent_help.rs` — hidden flag and stdout coverage
- [ ] `crates/asana-cli/tests/agent_help.rs` — required-subcommand bypass coverage
- [ ] `crates/todoer/tests/agent_help.rs` — required-subcommand bypass coverage
- [ ] `crates/silent-critic/tests/agent_help.rs` — required-subcommand bypass coverage
- [ ] `crates/gator/tests/agent_help.rs` — required-positional bypass coverage
- [ ] `crates/bsky-comment-extractor/tests/agent_help.rs` — regression for top-level-only behavior

## Sources

### Primary (HIGH confidence)
- Local repository: `07-CONTEXT.md`, `CLAUDE.md`, `Cargo.toml`, crate entrypoints, and existing
  tests — current workspace architecture and phase constraints
- clap docs: `Args`, `Parser`, `Command`, and `Arg` reflection/help APIs
- docs.rs crate pages for `serde_yaml` and `serde_yml`

### Secondary (MEDIUM confidence)
- None

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — driven by current workspace architecture and clap docs
- Architecture: MEDIUM — recommended design is well-supported by local constraints, but exact YAML
  schema remains open
- Pitfalls: HIGH — grounded in current local parser shapes and observed `bce query --agent-help`
  behavior

**Research date:** 2026-03-22
**Valid until:** 2026-04-21
