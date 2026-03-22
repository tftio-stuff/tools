# Milestones

## v1.1 bsky-comment-extractor (Shipped: 2026-03-22)

**Phases completed:** 2 phases, 4 plans, 0 tasks

**Key accomplishments:**

- (none recorded)

---

## v1.0 Gator Sandbox Hardening (Shipped: 2026-03-18)

**Phases completed:** 2 phases, 2 plans, 4 tasks
**Files changed:** 4 (+293 / -32 lines Rust)
**Timeline:** 2026-03-12 -- 2026-03-18 (6 days)

**Key accomplishments:**

- Agents launched in a linked worktree see only their own worktree by default (zero sibling RO grants)
- Common git dir RW grant preserved for commits, index, and refs
- `--share-worktrees` restores sibling RO grants with `--session` conflict validation
- Per-agent YOLO flag injection: `--dangerously-skip-permissions` (Claude), `--full-auto` (Codex), stderr warning (Gemini)
- `--no-yolo` opt-out with session conflict validation
- 13 new unit tests across both phases

---
