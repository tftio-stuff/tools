# bsky-comment-extractor

## What This Is

Tighten gator's default sandbox policy to follow the principle of least privilege. Agents should only see their own worktree by default, and should run in YOLO mode since sandbox-exec is the security boundary.

## Core Value

Complete, reliable extraction of a single BlueSky user's entire post history into a queryable local store.

## Requirements

### Validated

(None yet -- ship to validate)

### Active

- [ ] Sibling worktree RO grants are no longer added by default
- [ ] Common git dir RW grant is preserved for linked worktrees
- [ ] `--share-worktrees` flag opts in to RO access to all peer worktrees (restores old behavior)
- [ ] Agent-appropriate YOLO flags are injected by default (Claude: `--dangerously-skip-permissions`, Codex: `--full-auto`, Gemini: equivalent)
- [ ] `--no-yolo` flag disables automatic YOLO injection, restoring agent default permission mode
- [ ] Existing opt-in mechanisms (`--add-dirs-ro`, `.safehouse`, `--policy`) continue to work for manual peer grants

### Out of Scope

- Firehose/streaming consumption -- batch retrieval only
- Multi-user extraction in a single invocation
- Real-time monitoring or polling
- OAuth authentication -- app passwords sufficient
- Search by keyword -- extracts activity, not search results

## Context

Gator wraps coding agents (Claude, Codex, Gemini) with macOS `sandbox-exec`. The sandbox policy is assembled from a static base profile plus dynamic rules for the workdir, git common dir, sibling worktrees, extra dirs, and policy denies.

Current behavior automatically discovers all sibling worktrees and grants them RO access. This is too permissive -- when working in a worktree, the agent should be isolated to that worktree by default.

Current behavior also launches agents in their default interactive permission mode, requiring the user to approve tool calls. Since sandbox-exec already constrains filesystem access, the agent's built-in permission prompts are redundant friction.

The codebase already has the worktree detection infrastructure (`worktree.rs`), sandbox policy assembly (`sandbox.rs`), and CLI flag parsing (`cli.rs`). The changes are primarily in `lib.rs` (conditional worktree grant), `cli.rs` (new flags), and `agent.rs` (YOLO injection).

## Constraints

- **Backwards compatibility**: Users who depend on peer worktree access need a clear opt-in path (`--share-worktrees`)
- **Agent variance**: Each agent has different YOLO flags; gator must know the correct flag per agent
- **Session mode**: No changes to session-mode behavior (`--session` path is unaffected)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Drop sibling grants by default | Least privilege -- agent sees only its own worktree | -- Pending |
| Keep common git dir RW | Agent needs write access for commits, index, refs | -- Pending |
| Add --share-worktrees opt-in | Clear escape hatch for users who need cross-worktree reads | -- Pending |
| YOLO by default, --no-yolo opt-out | Sandbox is the security boundary; agent permissions are redundant | -- Pending |

---
*Last updated: 2026-03-17 after initialization*
