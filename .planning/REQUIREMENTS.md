# Requirements: bsky-comment-extractor

**Defined:** 2026-03-22
**Core Value:** Complete, reliable extraction of a single BlueSky user's entire post history into a queryable local store.

## bce-query-mode Requirements

### Query

- [x] **QUERY-01**: `bce query` reads posts from local SQLite and outputs JSONL to stdout (one JSON object per line)
- [x] **QUERY-02**: `--limit N` controls page size (default: 50)
- [x] **QUERY-03**: `--offset N` skips N records for pagination
- [x] **QUERY-04**: `--db <path>` specifies database path (XDG default)

### Agent Interface

- [x] **AGENT-01**: BCE exposes structured LLM-agent reference documentation through the shared restricted agent surface (`--agent-help` and `--agent-skill`)
- [x] **AGENT-02**: Query output wrapped in JSON envelope with pagination metadata (total, offset, limit, has_more)

## Future Requirements

### Additional Activity Types

- **LIKE-01**: Retrieve all `app.bsky.feed.like` records for a user
- **RPST-01**: Retrieve all `app.bsky.feed.repost` records for a user
- **BLCK-01**: Retrieve all `app.bsky.graph.block` records for a user
- **FILT-01**: `--type` flag to filter by activity type

## Out of Scope

| Feature | Reason |
|---------|--------|
| Firehose/streaming | Batch retrieval only |
| Multi-user extraction | Single user per invocation |
| Real-time monitoring | No polling or watch mode |
| OAuth authentication | App passwords sufficient |
| Keyword search | Extracts activity, not search results |
| Query-side filtering (--since, --author) | Agents filter client-side; keep query simple |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| QUERY-01 | Phase 5 | Complete |
| QUERY-02 | Phase 5 | Complete |
| QUERY-03 | Phase 5 | Complete |
| QUERY-04 | Phase 5 | Complete |
| AGENT-02 | Phase 5 | Complete |
| AGENT-01 | Phase 7 | Complete |

**Coverage:**
- bce-query-mode requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0

---
*Requirements defined: 2026-03-22*
*Last updated: 2026-03-23 after Phase 7 completion*
