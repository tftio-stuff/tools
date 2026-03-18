# bsky-comment-extractor

## What This Is

Gator wraps coding agents (Claude, Codex, Gemini) with macOS `sandbox-exec`. v1.0 tightened the default sandbox policy to follow the principle of least privilege: agents see only their own worktree by default and run in autonomous mode since sandbox-exec is the security boundary.

## Core Value

Complete, reliable extraction of a single BlueSky user's entire post history into a queryable local store.

## Requirements

### Validated

- ✓ Sibling worktree RO grants are no longer added by default -- v1.0
- ✓ Common git dir RW grant is preserved for linked worktrees -- v1.0
- ✓ `--share-worktrees` flag opts in to RO access to all peer worktrees -- v1.0
- ✓ Agent-appropriate YOLO flags are injected by default (Claude: `--dangerously-skip-permissions`, Codex: `--full-auto`, Gemini: stderr warning) -- v1.0
- ✓ `--no-yolo` flag disables automatic YOLO injection -- v1.0
- ✓ Existing opt-in mechanisms (`--add-dirs-ro`, `.safehouse`, `--policy`) continue to work -- v1.0
- ✓ Session mode (`--session`) behavior unchanged -- v1.0

### Active

(None -- define in next milestone)

### Out of Scope

- Firehose/streaming consumption -- batch retrieval only
- Multi-user extraction in a single invocation
- Real-time monitoring or polling
- OAuth authentication -- app passwords sufficient
- Search by keyword -- extracts activity, not search results

## Context

Shipped v1.0 with +261 net lines of Rust across 4 files (agent.rs, cli.rs, lib.rs, sandbox.rs). The codebase has the worktree detection infrastructure (`worktree.rs`), sandbox policy assembly (`sandbox.rs`), CLI flag parsing (`cli.rs`), and agent command construction (`agent.rs`).

## Constraints

- **Backwards compatibility**: Users who depend on peer worktree access use `--share-worktrees`
- **Agent variance**: Each agent has different YOLO flags; Gemini has no known YOLO equivalent
- **Session mode**: No changes to session-mode behavior (`--session` path is unaffected)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Drop sibling grants by default | Least privilege -- agent sees only its own worktree | ✓ Good |
| Keep common git dir RW | Agent needs write access for commits, index, refs | ✓ Good |
| Add --share-worktrees opt-in | Clear escape hatch for users who need cross-worktree reads | ✓ Good |
| YOLO by default, --no-yolo opt-out | Sandbox is the security boundary; agent permissions are redundant | ✓ Good |
| Sibling gating in lib.rs run() not in detect_worktrees | Detection stays pure, policy assembly gets filtered input | ✓ Good |
| Two-variable split: wt_for_policy + ungated_siblings | No WorktreeInfo mutation; diagnostic comments for dry-run | ✓ Good |
| Gemini gets stderr warning, no YOLO flag | No known Gemini YOLO equivalent exists | ✓ Good |

---
*Last updated: 2026-03-18 after v1.0 milestone*
