# Project Roadmap

## Milestones

- [x] **v1.0 Gator Sandbox Hardening** -- Phases 1-2 (shipped 2026-03-18)
- [x] **v1.1 bsky-comment-extractor** -- Phases 3-4 (shipped 2026-03-22)
- [x] **bce-query-mode** -- Phases 5 and 7 (shipped 2026-03-23)

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

<details>
<summary>[x] bce-query-mode (Phases 5 and 7) -- SHIPPED 2026-03-23</summary>

- [x] Phase 5: Query Subcommand (3/3 plans) -- completed 2026-03-22
- [x] Phase 7: Workspace agent mode in cli-common (6/6 plans) -- completed 2026-03-23

The earlier tool-local agent-help work was absorbed into the shared workspace agent-mode rollout.

</details>

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

### Phase 7: Workspace agent mode in cli-common: token-gated restricted capability surface, inspectable agent help, and shared --agent-skill support across tools

**Goal:** Workspace CLIs that depend on `cli-common` expose a shared token-gated agent mode with inspectable `--agent-help` / `--agent-skill` output and no hidden-surface leakage
**Requirements**: D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10, D-11, D-12, D-13, D-14
**Depends on:** Phase 5
**Plans:** 6 plans

Plans:
- [x] 07-01-PLAN.md — define shared cli-common agent-mode contracts, env vars, and capability policy types
- [x] 07-02-PLAN.md — implement shared filtered parse/help/completion pipeline plus agent renderers
- [x] 07-03-PLAN.md — migrate `bce` and `gator` to the shared restricted agent surface
- [x] 07-04-PLAN.md — migrate `todoer` and `unvenv` to the shared restricted agent surface
- [x] 07-05-PLAN.md — migrate `asana-cli` and `silent-critic` to the shared restricted agent surface
- [x] 07-06-PLAN.md — adapt `prompter` and restore workspace-wide agent-mode consistency checks

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Sandbox Isolation | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. YOLO Injection | v1.0 | 1/1 | Complete | 2026-03-18 |
| 3. Extraction Engine | v1.1 | 2/2 | Complete | 2026-03-22 |
| 4. CLI Surface | v1.1 | 2/2 | Complete | 2026-03-22 |
| 5. Query Subcommand | bce-query-mode | 3/3 | Complete | 2026-03-22 |
| 7. Workspace agent mode in cli-common | bce-query-mode | 6/6 | Complete | 2026-03-23 |
