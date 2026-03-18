# Requirements: Gator Sandbox Hardening

**Defined:** 2026-03-17
**Core Value:** An agent launched by gator cannot read peer worktrees unless explicitly granted access.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Sandbox Isolation

- [x] **SAND-01**: Sibling worktree RO grants are not added to sandbox policy by default
- [x] **SAND-02**: Common git dir RW grant is preserved for linked worktrees
- [x] **SAND-03**: `--share-worktrees` CLI flag opts in to RO access for all peer worktrees

### Agent Permissions

- [x] **PERM-01**: Gator injects agent-appropriate YOLO flag by default (Claude: `--dangerously-skip-permissions`, Codex: `--full-auto`, Gemini: equivalent)
- [x] **PERM-02**: `--no-yolo` CLI flag disables automatic YOLO injection

### Compatibility

- [x] **COMPAT-01**: Existing `--add-dirs-ro`, `.safehouse`, and `--policy` mechanisms continue to work for manual peer worktree grants
- [x] **COMPAT-02**: Session mode (`--session`) behavior is unchanged

## v2 Requirements

None identified.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Changing session mode behavior | Contract remains sole authority when `--session` is used |
| Changing the static base sandbox profile (`agent.sb`) | Separate concern, out of scope for this work |
| New sandbox grant types (e.g., execute-only) | Not needed for this hardening |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SAND-01 | Phase 1 | Complete |
| SAND-02 | Phase 1 | Complete |
| SAND-03 | Phase 1 | Complete |
| PERM-01 | Phase 2 | Complete |
| PERM-02 | Phase 2 | Complete |
| COMPAT-01 | Phase 1 | Complete |
| COMPAT-02 | Phase 1 | Complete |

**Coverage:**
- v1 requirements: 7 total
- Mapped to phases: 7
- Unmapped: 0

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-18 after Phase 1 Plan 01 completion*
