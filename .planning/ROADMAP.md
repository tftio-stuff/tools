# Project Roadmap

## Milestones

- [x] **v1.0 Gator Sandbox Hardening** -- Phases 1-2 (shipped 2026-03-18)
- [x] **v1.1 bsky-comment-extractor** -- Phases 3-4 (shipped 2026-03-22)
- [x] **bce-query-mode** -- Phase 5 (symbolic milestone, shipped 2026-03-22)
- [x] **cli-common-unification** -- Phase 01 (symbolic milestone, shipped 2026-03-22)
- [x] **cli-common-maximal-sharing** -- Phases 02-03 (symbolic milestone, shipped 2026-03-22)

## Phases

<details>
<summary>[x] v1.0 Gator Sandbox Hardening (Phases 1-2) -- SHIPPED 2026-03-18</summary>

- [x] Phase 1: Worktree Isolation (1/1 plan) -- completed 2026-03-15
- [x] Phase 2: YOLO Injection (1/1 plan) -- completed 2026-03-18

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

<details>
<summary>[x] v1.1 bsky-comment-extractor (Phases 3-4) -- SHIPPED 2026-03-22</summary>

- [x] Phase 3: Extraction Engine (2/2 plans) -- completed 2026-03-21
- [x] Phase 4: CLI Surface (2/2 plans) -- completed 2026-03-22

See: `.planning/milestones/v1.1-ROADMAP.md` for full details.

</details>

<details>
<summary>[x] bce-query-mode (Phase 5) -- SHIPPED 2026-03-22</summary>

- [x] Phase 5: Query Subcommand (3/3 plans) -- completed 2026-03-22

**Goal**: Make bce's stored data queryable by LLM agents via JSON output with pagination.

**Features delivered:**
- `bce query` subcommand: read-only against local SQLite
- Offset/limit pagination with QueryEnvelope metadata
- JSONL output format (envelope + posts)
- Query-specific error handling and reporting

Plans:
1. **05-01**: Query contracts and pagination helpers (db.rs + models.rs)
2. **05-02**: Query subcommand parser (cli.rs)
3. **05-03**: Query runtime dispatch (main.rs)

</details>

<details>
<summary>[x] cli-common-unification (Phase 01) -- SHIPPED 2026-03-22</summary>

- [x] Phase 01: CLI Common Unification (4/4 plans) -- completed 2026-03-22

See: `.planning/milestones/cli-common-unification-ROADMAP.md` for full details.

</details>

<details>
<summary>[x] cli-common-maximal-sharing (Phases 02-03) -- SHIPPED 2026-03-22</summary>

- [x] Phase 02: Maximize cli-common sharing (4/4 plans) -- completed 2026-03-22
- [x] Phase 03: Extract remaining CLI glue into `cli-common` (4/4 plans) -- completed 2026-03-22

See: `.planning/milestones/cli-common-maximal-sharing-ROADMAP.md` for full details.

</details>

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1. Worktree Isolation | v1.0 | 1/1 | Complete | 2026-03-15 |
| 2. YOLO Injection | v1.0 | 1/1 | Complete | 2026-03-18 |
| 3. Extraction Engine | v1.1 | 2/2 | Complete | 2026-03-22 |
| 4. CLI Surface | v1.1 | 2/2 | Complete | 2026-03-22 |
| 5. Query Subcommand | bce-query-mode | 3/3 | Complete | 2026-03-22 |
| 01. CLI Common Unification | cli-common-unification | 4/4 | Complete | 2026-03-22 |
| 02. Maximize cli-common sharing | cli-common-maximal-sharing | 4/4 | Complete | 2026-03-22 |
| 03. Extract remaining CLI glue into `cli-common` | cli-common-maximal-sharing | 4/4 | Complete | 2026-03-22 |

---
*Last updated: 2026-03-22 after reconciling bce-query-mode and cli-common-maximal-sharing milestones*
