# Requirements: Gator Sandbox Hardening

**Defined:** 2026-03-17
**Core Value:** An agent launched by gator cannot read peer worktrees unless explicitly granted access.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Sandbox Isolation

- [ ] **SAND-01**: Sibling worktree RO grants are not added to sandbox policy by default
- [ ] **SAND-02**: Common git dir RW grant is preserved for linked worktrees
- [ ] **SAND-03**: `--share-worktrees` CLI flag opts in to RO access for all peer worktrees

### Agent Permissions

- [ ] **PERM-01**: Gator injects agent-appropriate YOLO flag by default (Claude: `--dangerously-skip-permissions`, Codex: `--full-auto`, Gemini: equivalent)
- [ ] **PERM-02**: `--no-yolo` CLI flag disables automatic YOLO injection

### Compatibility

- [ ] **COMPAT-01**: Existing `--add-dirs-ro`, `.safehouse`, and `--policy` mechanisms continue to work for manual peer worktree grants
- [ ] **COMPAT-02**: Session mode (`--session`) behavior is unchanged

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
| SAND-01 | TBD | Pending |
| SAND-02 | TBD | Pending |
| SAND-03 | TBD | Pending |
| PERM-01 | TBD | Pending |
| PERM-02 | TBD | Pending |
| COMPAT-01 | TBD | Pending |
| COMPAT-02 | TBD | Pending |

**Coverage:**
- v1 requirements: 7 total
- Mapped to phases: 0
- Unmapped: 7

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-17 after initial definition*
