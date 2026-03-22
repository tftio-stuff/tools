# Roadmap: bsky-comment-extractor

## Milestones

- [x] **v1.0 Gator Sandbox Hardening** -- Phases 1-2 (shipped 2026-03-18)
- [x] **v1.1 bsky-comment-extractor** -- Phases 3-4 (shipped 2026-03-22)
- [x] **cli-common-unification** -- Phase 01 (symbolic milestone, shipped 2026-03-22)
- [ ] **cli-common-maximal-sharing** -- Phases 02-03 (symbolic milestone, in progress)

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
<summary>[x] cli-common-unification (Phase 01) -- SHIPPED 2026-03-22</summary>

- [x] Phase 01: CLI Common Unification (4/4 plans) -- completed 2026-03-22

See: `.planning/milestones/cli-common-unification-ROADMAP.md` for full details.

</details>

### cli-common-maximal-sharing (In Progress)

**Milestone Goal:** Maximize the amount of reusable CLI code that lives in `tftio-cli-common`, leaving per-tool crates with only domain-specific behavior.

- [x] **Phase 02: Maximize cli-common sharing** - Audit the tools and `cli-common`, then move duplicated and generally useful functionality into `cli-common`
- [x] **Phase 03: Extract remaining CLI glue into `cli-common`** - Remove the remaining metadata mapping, fatal runner, response-emitter, and doctor scaffolding glue from tool crates (completed 2026-03-22)

## Phase Details

### Phase 02: Maximize cli-common sharing

**Goal:** Audit the tools and the `cli-common` crate, then move as much shared or generally useful code as possible out of individual tools and into `cli-common`.
**Depends on:** Phase 01
**Requirements**: CLI-SHARE-01, CLI-SHARE-02, CLI-SHARE-03, CLI-SHARE-04
**Plans:** 4 plans

Plans:
- [x] 02-01-PLAN.md -- Expand `cli-common` with shared adapters, completion rendering buffers, and doctor-report primitives
- [x] 02-02-PLAN.md -- Migrate the adapter-heavy tools off local metadata and response boilerplate
- [x] 02-03-PLAN.md -- Migrate `unvenv`, `asana-cli`, and `prompter` onto the richer shared surface
- [x] 02-04-PLAN.md -- Finish dependency cleanup, repository enforcement, and documentation of the final boundary

### Phase 03: Extract remaining CLI glue into `cli-common`

**Goal:** Remove the remaining reusable metadata-mapping, fatal-runner, response-emitter, and doctor-provider scaffolding from individual tools so `cli-common` owns the full shared CLI substrate.
**Depends on:** Phase 02
**Requirements**: CLI-SHARE-04, CLI-SHARE-05, CLI-SHARE-06
**Plans:** 4/4 plans complete

Plans:
- [x] 03-01-PLAN.md -- Expand `cli-common` with the final shared metadata, runner, response, and doctor helper layer
- [x] 03-02-PLAN.md -- Migrate `gator`, `bce`, and `todoer` off the remaining thin-entrypoint glue
- [x] 03-03-PLAN.md -- Migrate `silent-critic`, `unvenv`, `asana-cli`, and `prompter` off the remaining richer glue
- [x] 03-04-PLAN.md -- Enforce the thinner boundary, document it, and run the full suite

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Sandbox Isolation | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. YOLO Injection | v1.0 | 1/1 | Complete | 2026-03-18 |
| 3. Extraction Engine | v1.1 | 2/2 | Complete | 2026-03-22 |
| 4. CLI Surface | v1.1 | 2/2 | Complete | 2026-03-22 |
| 01. CLI Common Unification | cli-common-unification | 4/4 | Complete | 2026-03-22 |
| 02. Maximize cli-common sharing | cli-common-maximal-sharing | 4/4 | Complete | 2026-03-22 |
| 03. Extract remaining CLI glue into `cli-common` | cli-common-maximal-sharing | 4/4 | Complete   | 2026-03-22 |
