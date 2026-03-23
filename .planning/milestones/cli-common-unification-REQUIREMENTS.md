# Requirements Archive: cli-common-unification

**Archived:** 2026-03-22
**Status:** SHIPPED
**Milestone Type:** symbolic release-please planning milestone

This milestone had no semantic version because repository releases are managed by release-please.

---

# Requirements: cli-common-unification

**Defined:** 2026-03-22
**Core Value:** Workspace CLIs share one consistent base UX contract instead of re-implementing metadata commands, JSON envelopes, error output, doctor behavior, and progress handling per crate.

## Requirements

### Shared CLI Foundation

- [x] **CLI-UNIFY-01**: `cli-common` exposes one shared tool metadata contract and standard metadata-command dispatcher
- [x] **CLI-UNIFY-02**: `gator`, `todoer`, and `silent-critic` use one shared JSON envelope and top-level error contract
- [x] **CLI-UNIFY-03**: `unvenv`, `bce`, and `asana-cli` use shared metadata/error/progress primitives without breaking their primary invocation paths
- [x] **CLI-UNIFY-04**: `prompter` joins the shared base contract and workspace-level consistency checks enforce it

## Out of Scope

| Feature | Reason |
|---------|--------|
| Release tagging from milestone completion | Release automation is handled by release-please |
| Replacing crate-specific domain workflows | This milestone standardized base UX only |
| New end-user product features | Scope was refactor and consistency, not feature expansion |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-UNIFY-01 | Phase 01 | Complete |
| CLI-UNIFY-02 | Phase 01 | Complete |
| CLI-UNIFY-03 | Phase 01 | Complete |
| CLI-UNIFY-04 | Phase 01 | Complete |

**Coverage:**
- Requirements: 4 total
- Mapped to phases: 4
- Unmapped: 0

---
*Requirements defined: 2026-03-22*
*Archived: 2026-03-22*
