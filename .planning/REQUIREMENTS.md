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

- [ ] **AGENT-01**: `--agent-help` outputs structured LLM-agent reference doc (capabilities, flags, output format, pagination examples, error codes)
- [x] **AGENT-02**: Query output wrapped in JSON envelope with pagination metadata (total, offset, limit, has_more)

### Shared Agent Documentation

- [x] **ADOC-01**: `cli-common` defines the canonical YAML agent-doc schema and shared renderer used by every binary for `--agent-help`
- [ ] **ADOC-02**: Each Phase 7 binary authors exhaustive tool-specific agent docs covering commands, arguments, examples, outputs, env/config/defaults, failure modes, and likely operator mistakes, sharing only truly common inherited sections
- [x] **ADOC-03**: `--agent-skill` renders the same underlying agent-doc content as a ready-to-save Claude-style skill file with YAML front matter and markdown body
- [x] **ADOC-04**: `--agent-help` and `--agent-skill` are hidden top-level-only flags that print to stdout on success, exit `0`, and stay out of normal help output
- [ ] **ADOC-05**: All seven workspace binaries (`prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`, `gator`, and `bce`) expose the shared agent-doc behavior through `cli-common` plus per-crate wiring

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
| AGENT-01 | Phase 6 | Pending |
| ADOC-01 | Phase 7 | Complete |
| ADOC-02 | Phase 7 | Pending |
| ADOC-03 | Phase 7 | Complete |
| ADOC-04 | Phase 7 | Complete |
| ADOC-05 | Phase 7 | Pending |

**Coverage:**
- bce-query-mode requirements: 11 total
- Mapped to phases: 11
- Unmapped: 0

---
*Requirements defined: 2026-03-22*
*Last updated: 2026-03-22 after bce-query-mode roadmap creation*
