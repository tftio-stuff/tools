# Phase 2: YOLO Injection - Research

**Researched:** 2026-03-18
**Domain:** Rust CLI argument injection, clap flag extension, per-agent command branching
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- `--no-yolo` and `--session` are mutually exclusive -- add to `Cli::validate()` conflict check (same pattern as `--share-worktrees` + `--session`)
- YOLO injection is skipped entirely in session mode -- session contract controls everything, agent uses its default permission mode
- Gemini has no known YOLO flag -- skip injection and print stderr warning: `gator: no YOLO flag known for gemini, skipping`
- `--no-yolo` suppresses the Gemini warning (user explicitly doesn't want YOLO, so the missing flag is irrelevant)

### Claude's Discretion
- Where YOLO injection logic lives (in `build_command()` alongside prompt injection, or in `lib::run()` before calling it)
- Exact stderr warning format for Gemini
- Test strategy for verifying flag injection (unit tests on `build_command()` return value)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PERM-01 | Gator injects agent-appropriate YOLO flag by default (Claude: `--dangerously-skip-permissions`, Codex: `--full-auto`, Gemini: equivalent/skip+warn) | `build_command()` already has per-agent `match` branching; add YOLO args inside same arms |
| PERM-02 | `--no-yolo` CLI flag disables automatic YOLO injection | `Cli` struct gets `no_yolo: bool` field; `build_command()` or caller passes it through; `validate()` adds session conflict check |
</phase_requirements>

## Summary

Phase 2 is a narrow, mechanical extension of existing patterns. The codebase already has exactly the right structure: `build_command()` in `agent.rs` uses a `match agent { Agent::Claude => ..., Agent::Codex => ..., Agent::Gemini => ... }` block for per-agent prompt injection. YOLO injection slots into those same arms. `Cli::validate()` already enforces `--session` mutual exclusion via a `conflicts` vec -- adding `--no-yolo` follows the identical pattern. The session bypass is already implemented (session path sets `WorktreeInfo::default()` and skips all normal injection); YOLO injection simply does not apply there.

The only non-trivial decision is where the `no_yolo` bool is threaded: passing it as a parameter to `build_command()` keeps injection co-located with the rest of command construction and is consistent with how `agent_args` is passed. The alternative -- checking in `lib::run()` before calling `build_command()` -- would require either appending to `agent_args` (mutating a caller-owned slice) or prepending flags differently. The cleanest approach is adding `inject_yolo: bool` to `build_command()`'s signature.

**Primary recommendation:** Add `no_yolo: bool` to `Cli`, extend `validate()` to conflict it with `--session`, pass `inject_yolo: bool` to `build_command()`, and inject the appropriate flag inside existing per-agent match arms. Gemini arm prints to stderr and skips when `inject_yolo` is true.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | workspace | CLI flag definition (`no_yolo: bool` field) | Already in use; `#[arg(long)]` bool fields are zero-cost opt-in flags |
| std::process::Command | stdlib | Command argument insertion | Already used in `build_command()` |
| std::io (eprintln!) | stdlib | Gemini stderr warning | No dep needed; `eprintln!` is idiomatic for diagnostic output |

### No New Dependencies
This phase requires no new crate dependencies. All needed primitives are already present.

## Architecture Patterns

### Recommended Change Surface

```
crates/gator/src/
├── cli.rs        # +no_yolo field, +validate() conflict check
├── agent.rs      # +inject_yolo param to build_command(), YOLO injection in match arms
└── lib.rs        # pass !cli.no_yolo as inject_yolo (session branch: false)
```

### Pattern 1: Adding a bool flag to Cli (established pattern)

**What:** Add `pub no_yolo: bool` with `#[arg(long)]` to the `Cli` struct.
**When to use:** Any opt-out boolean flag where the default behavior is active.
**Example from existing code (`share_worktrees`):**
```rust
// Source: crates/gator/src/cli.rs line 61
/// Grant read-only access to all peer worktrees (disabled by default).
#[arg(long)]
pub share_worktrees: bool,
```
Apply same pattern for `--no-yolo`:
```rust
/// Disable automatic YOLO flag injection (default: inject).
#[arg(long)]
pub no_yolo: bool,
```

### Pattern 2: Adding a session conflict check (established pattern)

**What:** Add `no_yolo` to the `conflicts` vec inside `Cli::validate()`.
**When to use:** Any flag that is meaningless or contradictory when `--session` is active.
**Example from existing code:**
```rust
// Source: crates/gator/src/cli.rs lines 100-101
if self.share_worktrees {
    conflicts.push("--share-worktrees");
}
```
Add identically:
```rust
if self.no_yolo {
    conflicts.push("--no-yolo");
}
```

### Pattern 3: Per-agent branching in build_command() (established pattern)

**What:** Add YOLO flag injection inside the existing `match agent` block, conditioned on `inject_yolo`.
**When to use:** Any per-agent behavioral difference during command construction.
**Current signature:**
```rust
// Source: crates/gator/src/agent.rs line 32
pub fn build_command(
    agent: &Agent,
    policy_path: &Path,
    prompt: Option<&str>,
    agent_args: &[String],
) -> Result<(Command, Vec<NamedTempFile>), io::Error>
```
New signature adds `inject_yolo: bool` parameter. YOLO args are injected before `agent_args` forwarding, after the binary name is added and after prompt injection:
```rust
// YOLO injection (before forwarding agent_args)
if inject_yolo {
    match agent {
        Agent::Claude => { cmd.arg("--dangerously-skip-permissions"); }
        Agent::Codex  => { cmd.arg("--full-auto"); }
        Agent::Gemini => { eprintln!("gator: no YOLO flag known for gemini, skipping"); }
    }
}
```

### Pattern 4: Session branch passes inject_yolo: false

**What:** In `lib::run()`, the session branch already bypasses all injection. Pass `inject_yolo: false` explicitly, or derive from `!cli.no_yolo` with session guard.
**Recommended:** Compute `inject_yolo` before the branch and force it `false` in the session branch:
```rust
// Non-session path: derive from flag
// Session path: always false -- contract controls
let inject_yolo = !cli.no_yolo && cli.session.is_none();
```
This single expression is readable, correct for both branches, and requires no extra conditional logic at the `build_command()` call site.

### Call site update in lib.rs

```rust
// Source: crates/gator/src/lib.rs line 113 (current)
let (cmd, _tempfiles) = agent::build_command(
    &cli.agent,
    policy_file.path(),
    prompt.as_deref(),
    &cli.agent_args,
)

// Updated call:
let inject_yolo = !cli.no_yolo && cli.session.is_none();
let (cmd, _tempfiles) = agent::build_command(
    &cli.agent,
    policy_file.path(),
    prompt.as_deref(),
    &cli.agent_args,
    inject_yolo,
)
```

### Anti-Patterns to Avoid

- **Appending to agent_args:** Do not push YOLO flags into `cli.agent_args` before calling `build_command()`. It mutates what appears to be a user-provided list and breaks test readability.
- **Checking in lib::run() and not passing to build_command():** Splitting command construction across two locations. Keep all flag-to-arg mapping in `build_command()`.
- **Using an env var for YOLO state:** Unnecessary indirection; a bool parameter is sufficient and testable.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mutual exclusion validation | Custom conflict detection logic | Extend existing `validate()` vec pattern | Pattern already tested and understood |
| Argument ordering | Complex insertion logic | `cmd.arg()` call order in match arms | `Command` preserves insertion order; put YOLO before `agent_args` |

**Key insight:** The existing `build_command()` match-arm pattern is exactly the right place for YOLO injection. No new abstraction is needed.

## Common Pitfalls

### Pitfall 1: YOLO flag placement relative to agent_args
**What goes wrong:** If YOLO flags are appended after `agent_args`, they appear after user-supplied args, which may cause flag parsing issues with some agents.
**Why it happens:** `cmd.args(agent_args)` is the last statement in the current function. Appending after loses positional guarantees.
**How to avoid:** Insert the YOLO `cmd.arg()` call BEFORE `cmd.args(agent_args)`.
**Warning signs:** Tests that check `args` vec for YOLO flag presence pass, but real agent rejects unknown trailing flags.

### Pitfall 2: Forgetting to update all existing tests that call build_command()
**What goes wrong:** All existing `build_command` tests in `agent.rs` will fail to compile because the signature gains a new parameter.
**Why it happens:** Rust requires all arguments to be supplied at call sites -- no optional parameters without `Option`.
**How to avoid:** Update all four existing test call sites (`build_command_claude_no_prompt`, `build_command_claude_with_prompt`, `build_command_codex_with_prompt`, `build_command_gemini_with_prompt`) to pass `inject_yolo`.
**Warning signs:** `cargo test -p tftio-gator` fails with `expected 4 arguments, found 5`.

### Pitfall 3: Gemini warning fires when --no-yolo is used
**What goes wrong:** User passes `--no-yolo` with `gemini` and still sees the "no YOLO flag known" warning, which is confusing.
**Why it happens:** Warning is emitted whenever `inject_yolo` is true, but `inject_yolo` is false when `--no-yolo` is set -- so this pitfall is already prevented by the design.
**How to avoid:** The `inject_yolo` gate handles this: warning only fires when `inject_yolo: true`, which requires `!cli.no_yolo`.

### Pitfall 4: Clippy pedantic -- function argument count
**What goes wrong:** Workspace lints include `clippy::pedantic = "deny"`. Adding a 5th parameter to `build_command()` is fine, but any new function with many bools may trigger `clippy::fn_params_excessive_bools`.
**Why it happens:** Pedantic clippy warns when a function takes 3+ bool params.
**How to avoid:** `build_command` currently has one bool (`inject_yolo`). Adding one bool is safe. If future growth requires more, use a config struct. Not an issue for this phase.

### Pitfall 5: Missing docs lint
**What goes wrong:** Workspace lint `rust.missing_docs = "deny"` requires doc comments on all public items.
**Why it happens:** Any new public function, field, or struct without `///` doc comment fails `cargo clippy`.
**How to avoid:** Add doc comments to `no_yolo` field in `Cli` and to the `inject_yolo` param in `build_command()` doc block.

## Code Examples

Verified from reading the actual source files:

### Complete updated build_command() signature and YOLO block
```rust
// Source: crates/gator/src/agent.rs -- updated version
/// Build the full command for `sandbox-exec`, including prompt and YOLO injection.
///
/// `inject_yolo` controls whether to prepend the agent-appropriate
/// autonomous-mode flag. Set to `false` when `--no-yolo` is passed or
/// when running in session mode.
///
/// # Errors
/// Returns an error if prompt tempfile creation fails.
pub fn build_command(
    agent: &Agent,
    policy_path: &Path,
    prompt: Option<&str>,
    agent_args: &[String],
    inject_yolo: bool,
) -> Result<(Command, Vec<NamedTempFile>), io::Error> {
    let mut cmd = Command::new("sandbox-exec");
    cmd.arg("-f").arg(policy_path).arg("--");

    let mut tempfiles: Vec<NamedTempFile> = Vec::new();

    // Agent binary name
    cmd.arg(agent.to_string());

    // Inject prompt per agent's mechanism
    if let Some(prompt_text) = prompt {
        match agent {
            Agent::Claude => {
                cmd.arg("--append-system-prompt").arg(prompt_text);
            }
            Agent::Codex => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.arg("-c")
                    .arg(format!("experimental_instructions_file={}", path.display()));
                tempfiles.push(f);
            }
            Agent::Gemini => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.env("GEMINI_SYSTEM_MD", path);
                tempfiles.push(f);
            }
        }
    }

    // Inject YOLO flag (autonomous mode) per agent
    if inject_yolo {
        match agent {
            Agent::Claude => {
                cmd.arg("--dangerously-skip-permissions");
            }
            Agent::Codex => {
                cmd.arg("--full-auto");
            }
            Agent::Gemini => {
                eprintln!("gator: no YOLO flag known for gemini, skipping");
            }
        }
    }

    // Forward remaining agent args
    cmd.args(agent_args);

    Ok((cmd, tempfiles))
}
```

### inject_yolo derivation in lib::run()
```rust
// Source: crates/gator/src/lib.rs -- before build_command() call site
let inject_yolo = !cli.no_yolo && cli.session.is_none();
let (cmd, _tempfiles) = agent::build_command(
    &cli.agent,
    policy_file.path(),
    prompt.as_deref(),
    &cli.agent_args,
    inject_yolo,
)
.map_err(|e| format!("cannot build command: {e}"))?;
```

### validate() conflict addition
```rust
// Source: crates/gator/src/cli.rs -- inside Cli::validate()
if self.no_yolo {
    conflicts.push("--no-yolo");
}
```

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| YOLO flags passed manually via `--` passthrough | Automatic injection with `--no-yolo` opt-out | Phase 2 adds this |
| No per-agent permission mode | Agent-appropriate flag per binary | Phase 2 adds this |

## Open Questions

1. **Exact Gemini stderr message format**
   - What we know: Message should be `gator: no YOLO flag known for gemini, skipping` per CONTEXT.md
   - What's unclear: Nothing; message is locked by user decision
   - Recommendation: Use verbatim string from CONTEXT.md

2. **Future Gemini YOLO flag**
   - What we know: No equivalent flag exists as of research date
   - What's unclear: Google may add one
   - Recommendation: The `Agent::Gemini` arm in the match is already the right extension point; no structural change needed when a flag is discovered

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none -- inline `#[cfg(test)]` modules |
| Quick run command | `cargo test -p tftio-gator` |
| Full suite command | `just test` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| PERM-01 | Claude gets `--dangerously-skip-permissions` in command args | unit | `cargo test -p tftio-gator build_command_claude_yolo` | New test in agent.rs |
| PERM-01 | Codex gets `--full-auto` in command args | unit | `cargo test -p tftio-gator build_command_codex_yolo` | New test in agent.rs |
| PERM-01 | Gemini prints stderr warning, no arg injected | unit | `cargo test -p tftio-gator build_command_gemini_yolo_warn` | New test; stderr capture via std redirect or check no extra arg |
| PERM-01 | YOLO not injected in session mode | unit | `cargo test -p tftio-gator yolo_skipped_in_session` | Test via inject_yolo=false path |
| PERM-02 | `--no-yolo` parses correctly from CLI | unit | `cargo test -p tftio-gator parse_no_yolo` | New test in cli.rs |
| PERM-02 | `--no-yolo` + `--session` rejected by validate() | unit | `cargo test -p tftio-gator validate_no_yolo_with_session` | New test in cli.rs |
| PERM-02 | With `--no-yolo`, no YOLO flag in command | unit | `cargo test -p tftio-gator build_command_no_yolo_skips_injection` | New test in agent.rs |
| PERM-02 | `--no-yolo` with Gemini suppresses warning | unit | manual observation or stderr capture | Document in test comment if not automated |

### Sampling Rate
- **Per task commit:** `cargo test -p tftio-gator`
- **Per wave merge:** `just test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None -- existing test infrastructure (inline `#[cfg(test)]` modules, `cargo test -p`) covers all phase requirements. No new framework, fixtures, or config files are needed.

## Sources

### Primary (HIGH confidence)
- Direct source read: `crates/gator/src/agent.rs` -- `build_command()` full implementation and existing tests
- Direct source read: `crates/gator/src/cli.rs` -- `Cli` struct, `Agent` enum, `validate()` implementation and tests
- Direct source read: `crates/gator/src/lib.rs` -- `run()` full implementation, session branch, `build_command()` call at line 113
- Direct source read: `.planning/phases/02-yolo-injection/02-CONTEXT.md` -- locked decisions and integration points

### Secondary (MEDIUM confidence)
- Rust/clap documentation (training knowledge): `#[arg(long)]` bool fields produce `--flag` / absent pattern with `false` default

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, no new dependencies
- Architecture: HIGH -- patterns derived from reading actual source, not assumed
- Pitfalls: HIGH -- derived from static analysis of call sites and workspace lint config
- Test strategy: HIGH -- existing test module structure directly observed

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable codebase; only changes if agent CLI flags change upstream)
