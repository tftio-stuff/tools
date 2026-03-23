# Requirements Archive: cli-common-maximal-sharing

**Archived:** 2026-03-22
**Status:** SHIPPED
**Milestone Type:** symbolic release-please planning milestone

This milestone had no semantic version tag because repository releases are managed by release-please.

---

# Requirements: cli-common-maximal-sharing

**Defined:** 2026-03-22
**Core Value:** Shared CLI behavior should live in `tftio-cli-common`, leaving individual tools with only domain-specific code and command trees.

## Requirements

### Shared Surface Expansion

- [x] **CLI-SHARE-01**: Reusable CLI boilerplate that appears across tools moves into `tftio-cli-common` (tool specs, no-doctor adapters, standard-command/meta-command wiring, completion wrappers)
- [x] **CLI-SHARE-02**: Generally useful doctor/output/completion/reporting primitives move into `tftio-cli-common` instead of staying embedded in individual tools
- [x] **CLI-SHARE-03**: Workspace tools migrate onto the expanded shared surface without breaking their existing primary command flows
- [x] **CLI-SHARE-04**: Repository-level automation detects remaining shared-behavior drift and guards the intentionally crate-specific exceptions
- [x] **CLI-SHARE-05**: The remaining metadata mapping, fatal runner, and doctor-provider scaffolding move into `tftio-cli-common` wherever they are shared or clearly general-purpose
- [x] **CLI-SHARE-06**: The remaining reusable JSON-vs-text success-path plumbing moves into `tftio-cli-common`, leaving per-tool crates responsible only for domain text/data preparation

## Requirement Outcomes

- `CLI-SHARE-01` through `CLI-SHARE-04` were completed across Phases 02-03 and are now enforced by repository shell tests plus cargo verification.
- `CLI-SHARE-05` completed in Phase 03 by moving metadata mapping, fatal runners, and doctor-report scaffolding into `cli-common`.
- `CLI-SHARE-06` completed in Phase 03 by adding lazy shared response emission while keeping domain-specific text formatting local.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Replacing domain-specific command handlers | This milestone targeted shared UX and reusable CLI infrastructure only |
| Removing crate-specific dynamic completion augmentation | Tool-specific data sources stay local even if the rendering wrapper moves to `cli-common` |
| Flattening domain summaries/tables into `cli-common` | Shared response emitters wrap JSON/text selection, but task tables and tool-specific reports remain local |
| Release tagging/versioning policy changes | Release automation remains owned by release-please |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-SHARE-01 | Phase 02 | Complete |
| CLI-SHARE-02 | Phase 02 | Complete |
| CLI-SHARE-03 | Phase 02 | Complete |
| CLI-SHARE-04 | Phase 02, Phase 03 | Complete |
| CLI-SHARE-05 | Phase 03 | Complete |
| CLI-SHARE-06 | Phase 03 | Complete |

**Coverage:**
- Requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0

---
*Requirements defined: 2026-03-22*
*Archived: 2026-03-22*
