# Roadmap: Gator Sandbox Hardening

## Overview

Two focused changes to gator's launch behavior: tighten the default sandbox policy so agents see only their own worktree, then inject agent-appropriate YOLO flags so sandbox-exec is the sole security boundary. Both changes are independently verifiable and can ship sequentially against the existing codebase.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Sandbox Isolation** - Remove default sibling worktree grants and add --share-worktrees opt-in (completed 2026-03-18)
- [ ] **Phase 2: YOLO Injection** - Inject agent YOLO flags by default and add --no-yolo opt-out

## Phase Details

### Phase 1: Sandbox Isolation
**Goal**: Agents are isolated to their own worktree by default; users who need cross-worktree reads have a clear opt-in
**Depends on**: Nothing (first phase)
**Requirements**: SAND-01, SAND-02, SAND-03, COMPAT-01, COMPAT-02
**Success Criteria** (what must be TRUE):
  1. Launching gator in a linked worktree does not grant RO access to any sibling worktree in the sandbox policy
  2. The common git dir still receives RW access so commits, index updates, and ref operations succeed
  3. Running gator with --share-worktrees produces a sandbox policy that grants RO access to all peer worktrees (restoring pre-hardening behavior)
  4. Existing --add-dirs-ro, .safehouse, and --policy invocations continue to expand sandbox access without modification
  5. Running gator with --session is unaffected by sandbox isolation changes
**Plans:** 1/1 plans complete

Plans:
- [x] 01-01-PLAN.md -- Add --share-worktrees flag, sibling gating, and dry-run diagnostics

### Phase 2: YOLO Injection
**Goal**: Agents launch in autonomous mode by default, with a clear opt-out for users who want manual approval
**Depends on**: Phase 1
**Requirements**: PERM-01, PERM-02
**Success Criteria** (what must be TRUE):
  1. Launching gator with Claude injects --dangerously-skip-permissions into the agent command automatically
  2. Launching gator with Codex injects --full-auto into the agent command automatically
  3. Launching gator with --no-yolo starts the agent without any injected YOLO flag (agent uses its default permission mode)
  4. YOLO injection does not affect --session launches (session path unchanged)
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Sandbox Isolation | 1/1 | Complete    | 2026-03-18 |
| 2. YOLO Injection | 0/? | Not started | - |
