# Requirements: cli-common-maximal-sharing

**Defined:** 2026-03-22
**Core Value:** Shared CLI behavior should live in `tftio-cli-common`, leaving individual tools with only domain-specific code and command trees.

## Requirements

### Shared Surface Expansion

- [ ] **CLI-SHARE-01**: Reusable CLI boilerplate that appears across tools moves into `tftio-cli-common` (tool specs, no-doctor adapters, standard-command/meta-command wiring, completion wrappers)
- [ ] **CLI-SHARE-02**: Generally useful doctor/output/completion/reporting primitives move into `tftio-cli-common` instead of staying embedded in individual tools
- [ ] **CLI-SHARE-03**: Workspace tools migrate onto the expanded shared surface without breaking their existing primary command flows
- [ ] **CLI-SHARE-04**: Repository-level automation detects remaining shared-behavior drift and guards the intentionally crate-specific exceptions
- [x] **CLI-SHARE-05**: The remaining metadata mapping, fatal runner, and doctor-provider scaffolding move into `tftio-cli-common` wherever they are shared or clearly general-purpose
- [x] **CLI-SHARE-06**: The remaining reusable JSON-vs-text success-path plumbing moves into `tftio-cli-common`, leaving per-tool crates responsible only for domain text/data preparation

## Out of Scope

| Feature | Reason |
|---------|--------|
| Replacing domain-specific command handlers | This milestone targets shared UX and reusable CLI infrastructure only |
| Removing crate-specific dynamic completion augmentation | Tool-specific data sources stay local even if the rendering wrapper moves to `cli-common` |
| Flattening domain summaries/tables into `cli-common` | Shared response emitters may wrap data/text selection, but task tables and tool-specific reports remain local |
| Release tagging/versioning policy changes | Release automation remains owned by release-please |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-SHARE-01 | Phase 02 | Complete |
| CLI-SHARE-02 | Phase 02 | Complete |
| CLI-SHARE-03 | Phase 02 | Complete |
| CLI-SHARE-04 | Phase 02, Phase 03 | Planned |
| CLI-SHARE-05 | Phase 03 | Planned |
| CLI-SHARE-06 | Phase 03 | Planned |

**Coverage:**
- Requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0

---
*Requirements updated: 2026-03-22 for Phase 03 planning*
