# Phase 1: Sandbox Isolation - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Remove default sibling worktree RO grants from the sandbox policy. Agents launched by gator in a linked worktree see only their own worktree by default. Add `--share-worktrees` CLI flag as an opt-in to restore RO access to all peer worktrees. Common git dir RW grant is preserved. Existing manual grant mechanisms (.safehouse, --add-dirs-ro, --policy) are unaffected. Session mode is unaffected.

</domain>

<decisions>
## Implementation Decisions

### Worktree detection behavior
- `detect_worktrees()` always enumerates siblings regardless of `--share-worktrees` -- detection stays pure and reusable
- Main worktree is treated the same as linked siblings for grant purposes (no special RO grant)
- Gating logic lives in `lib::run()` before calling `assemble_policy()` -- if `--share-worktrees` is absent, clear `worktree_info.siblings` before policy assembly
- Store full detected `WorktreeInfo` separately so dry-run can still report detected-but-not-granted siblings

### Flag interaction with .safehouse
- Manual grants (.safehouse `add-dirs-ro=`, `--add-dirs-ro`, `--policy` profiles) always work, even for sibling worktree paths, without requiring `--share-worktrees`
- `--share-worktrees` is the auto-discover-all shortcut; manual grants are explicit user intent and always honored
- Named policy profiles follow the same rule as .safehouse -- explicit grants always honored
- All grant sources remain additive (same as current behavior)

### Diagnostic output
- `--dry-run` shows SBPL comments for detected-but-not-granted siblings: `;; Sibling worktree (not granted): /path/to/sibling`
- Normal (non-dry-run) invocations are silent about sibling isolation -- no stderr output
- `assemble_policy()` receives a new parameter (e.g., `detected_siblings: &[PathBuf]`) and emits the not-granted comments, keeping all policy formatting in one place

### Error handling edge cases
- `--share-worktrees` outside a git repo: silent no-op (detect_worktrees already returns empty info)
- `--share-worktrees` on main worktree with no linked siblings: silent no-op (consistent behavior)
- `--share-worktrees` and `--session` are mutually exclusive -- add to existing `cli.validate()` conflict check

### Claude's Discretion
- Exact implementation of how full WorktreeInfo is preserved alongside the cleared-for-policy version (struct field, separate variable, etc.)
- Whether `assemble_policy()` signature change uses a separate slice parameter or an extended `WorktreeInfo` struct
- Test strategy details (unit vs integration test balance)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Sandbox policy assembly
- `crates/gator/src/sandbox.rs` -- SBPL policy generation, emit helpers, `assemble_policy()` function
- `crates/gator/src/worktree.rs` -- `WorktreeInfo` struct and `detect_worktrees()` function

### CLI and configuration
- `crates/gator/src/cli.rs` -- clap CLI definitions, `Cli` struct, `validate()` method for mutual exclusion
- `crates/gator/src/config.rs` -- `ExtraDirs`, `.safehouse` loading, policy loading, `merge_extra_dirs()`

### Orchestration
- `crates/gator/src/lib.rs` -- `run()` function that wires detection, config, and policy assembly together
- `crates/gator/src/session.rs` -- Session mode path (unchanged by this phase)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `WorktreeInfo` struct already has `common_dir` and `siblings` fields -- siblings list is what gets gated
- `emit_ro_grant()` and `emit_ancestors()` in sandbox.rs -- existing helpers for grant emission
- `Cli::validate()` -- existing mutual-exclusion checker to extend with `--share-worktrees` vs `--session`

### Established Patterns
- All policy formatting in `sandbox.rs` with SBPL comment annotations (`;; Sibling worktree: ...`)
- Grant merging is additive across all sources (`.safehouse` + `--add-dirs-*` + `--policy`)
- Session mode bypasses all auto-detection (uses `WorktreeInfo::default()`)

### Integration Points
- `lib::run()` non-session branch (line 41-61) -- where `detect_worktrees()` result flows into `assemble_policy()`
- `Cli` struct -- needs new `--share-worktrees` field
- `assemble_policy()` signature -- needs detected-but-not-granted siblings parameter

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 01-sandbox-isolation*
*Context gathered: 2026-03-17*
