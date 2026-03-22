# Project Roadmap

## Milestones

- checkmark **v1.0 Gator Sandbox Hardening** -- Phases 1-2 (shipped 2026-03-18)
- wip **v1.1 bsky-comment-extractor** -- Phases 3-4 (in progress)

## Phases

<details>
<summary>checkmark v1.0 Gator Sandbox Hardening (Phases 1-2) -- SHIPPED 2026-03-18</summary>

- [x] Phase 1: Sandbox Isolation (1/1 plans) -- completed 2026-03-18
- [x] Phase 2: YOLO Injection (1/1 plans) -- completed 2026-03-18

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

### v1.1 bsky-comment-extractor (In Progress)

**Milestone Goal:** A working Rust workspace crate that exhaustively fetches a BlueSky user's post history via the AT Protocol and stores it in a queryable local SQLite database.

- [ ] **Phase 3: Extraction Engine** - AT Protocol client, auth, exhaustive pagination, SQLite storage
- [ ] **Phase 4: CLI Surface** - clap interface, workspace integration, progress indicator

**Goal**: Make bce's stored data queryable by LLM agents via JSON output with pagination.

**Features delivered:**
- `bce query` subcommand: read-only against local SQLite
- Offset/limit pagination with QueryEnvelope metadata
- JSONL output format (envelope + posts)
- Query-specific error handling and reporting

### Phase 3: Extraction Engine
**Goal**: A user's complete BlueSky post history can be fetched and stored in SQLite
**Depends on**: Nothing (new crate, no prior phases)
**Requirements**: AUTH-01, AUTH-02, EXTR-01, EXTR-02, EXTR-03, EXTR-04, STOR-01, STOR-02, STOR-03
**Success Criteria** (what must be TRUE):
  1. Given a BlueSky handle and app password, the tool authenticates and receives a valid session token via `com.atproto.server.createSession`
  2. All posts for a user are retrieved via `com.atproto.repo.listRecords` with cursor-based pagination until no cursor remains
  3. A handle (e.g., `alice.bsky.social`) is resolved to a DID before fetching records
  4. On HTTP 429, the client backs off and retries rather than crashing or returning partial results
  5. Posts are written to SQLite with AT URI, author DID, text, created_at, reply parent, and raw JSON; re-running does not produce duplicate rows
**Plans:** 2 plans
Plans:
- [x] 03-01-PLAN.md -- Crate scaffold, types/models/error, SQLite storage layer
- [ ] 03-02-PLAN.md -- AT Protocol client: auth, handle resolution, exhaustive pagination, rate-limit backoff

### Phase 4: CLI Surface
**Goal**: The extraction engine is usable as a first-class CLI tool following workspace conventions
**Depends on**: Phase 3
**Requirements**: CLI-01, CLI-02, CLI-03, CLI-04
**Success Criteria** (what must be TRUE):
  1. Running `bsky-comment-extractor <handle>` with credentials triggers extraction and writes to `./bsky-posts.db` by default
  2. Running with `--db /path/to/file.db` writes to the specified path instead
  3. A progress indicator updates in the terminal during extraction showing records retrieved
  4. The crate compiles and passes `just ci` (format, lint, test, audit, deny) as a workspace member
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 3 -> 4

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Sandbox Isolation | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. YOLO Injection | v1.0 | 1/1 | Complete | 2026-03-18 |
| 3. Extraction Engine | v1.1 | 1/2 | In progress | - |
| 4. CLI Surface | v1.1 | 0/? | Not started | - |
