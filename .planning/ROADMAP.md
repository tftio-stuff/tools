# Roadmap: bsky-comment-extractor

## Milestones

- [x] **v1.0 Gator Sandbox Hardening** -- Phases 1-2 (shipped 2026-03-18)
- [x] **v1.1 bsky-comment-extractor** -- Phases 3-4 (shipped 2026-03-22)
- [ ] **bce-query-mode** -- Phases 5-6 (active)

## Phases

<details>
<summary>[x] v1.0 Gator Sandbox Hardening (Phases 1-2) -- SHIPPED 2026-03-18</summary>

- [x] Phase 1: Sandbox Isolation (1/1 plans) -- completed 2026-03-18
- [x] Phase 2: YOLO Injection (1/1 plans) -- completed 2026-03-18

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

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
- [ ] 05-01-PLAN.md — add query models and read-only SQLite pagination helpers
- [x] 05-02-PLAN.md — migrate clap parser to fetch/query subcommands with top-level `--agent-help`
- [ ] 05-03-PLAN.md — wire main.rs query JSONL output, structured query errors, and integration tests

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

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Sandbox Isolation | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. YOLO Injection | v1.0 | 1/1 | Complete | 2026-03-18 |
| 3. Extraction Engine | v1.1 | 2/2 | Complete | 2026-03-22 |
| 4. CLI Surface | v1.1 | 2/2 | Complete | 2026-03-22 |
| 5. Query Subcommand | bce-query-mode | 1/3 | In Progress | - |
| 6. Agent Help | bce-query-mode | 0/? | Not started | - |
