# Roadmap: bsky-comment-extractor

## Overview

Two focused changes to gator's launch behavior: tighten the default sandbox policy so agents see only their own worktree, then inject agent-appropriate YOLO flags so sandbox-exec is the sole security boundary. Both changes are independently verifiable and can ship sequentially against the existing codebase.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Sandbox Isolation** - Remove default sibling worktree grants and add --share-worktrees opt-in
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
**Plans:** 1 plan

Plans:
- [ ] 01-01-PLAN.md -- Add --share-worktrees flag, sibling gating, and dry-run diagnostics

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

<details>
<summary>[x] v1.1 bsky-comment-extractor (Phases 3-4) -- SHIPPED 2026-03-22</summary>

- [x] Phase 3: Extraction Engine (2/2 plans) -- completed 2026-03-22
- [x] Phase 4: CLI Surface (2/2 plans) -- completed 2026-03-22

See: `.planning/milestones/v1.1-ROADMAP.md` for full details.

</details>

### bce-query-mode (Phases 5-6)

- [ ] **Phase 5: Query Subcommand** - `bce query` reads stored posts as paginated JSONL with envelope metadata
- [ ] **Phase 6: Agent Help** - `--agent-help` outputs structured LLM-consumable reference documentation

## Phase Details

### Phase 5: Query Subcommand
**Goal**: Users can query stored posts from the local database as paginated JSONL output
**Depends on**: Phase 4 (existing SQLite store from v1.1)
**Requirements**: QUERY-01, QUERY-02, QUERY-03, QUERY-04, AGENT-02
**Success Criteria** (what must be TRUE):
  1. `bce query` prints one JSON object per line to stdout for stored posts
  2. `--limit N` controls how many records appear in the output
  3. `--offset N` skips records, enabling sequential page traversal
  4. `--db <path>` points the query at a non-default database file
  5. Each output batch is wrapped in a JSON envelope with `total`, `offset`, `limit`, and `has_more` fields
**Plans**: 3 plans
Plans:
- [x] 05-01-PLAN.md — add query models and read-only SQLite pagination helpers
- [x] 05-02-PLAN.md — migrate clap parser to fetch/query subcommands with top-level `--agent-help`
- [x] 05-03-PLAN.md — wire main.rs query JSONL output, structured query errors, and integration tests

### Phase 6: Agent Help
**Goal**: LLM agents can discover how to use `bce` without reading source code
**Depends on**: Phase 5
**Requirements**: AGENT-01
**Success Criteria** (what must be TRUE):
  1. `bce --agent-help` prints a structured reference document to stdout
  2. The document covers capabilities, all flags, output format, pagination usage, and error codes
  3. The output format is machine-parseable (structured text or JSON suitable for agent consumption)
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Sandbox Isolation | 0/1 | Not started | - |
| 2. YOLO Injection | 0/? | Not started | - |
