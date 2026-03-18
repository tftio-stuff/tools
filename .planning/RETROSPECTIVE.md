# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 -- Gator Sandbox Hardening

**Shipped:** 2026-03-18
**Phases:** 2 | **Plans:** 2 | **Sessions:** 3

### What Was Built
- Least-privilege worktree sandbox: agents see only their own worktree by default
- Per-agent YOLO flag injection (Claude, Codex) with Gemini stderr warning
- Two new CLI flags (`--share-worktrees`, `--no-yolo`) with session conflict validation
- Dry-run diagnostic SBPL comments for detected-but-ungated siblings

### What Worked
- Small, focused milestone (2 phases, 2 plans) shipped cleanly in one day of execution
- Phase 2 built directly on Phase 1's patterns without breaking anything
- Two-variable split pattern (wt_for_policy + ungated_siblings) kept code clean and testable
- 13 new unit tests written inline with implementation

### What Was Inefficient
- Sandbox environment blocked `cargo test` execution during both phases -- tests had to be verified structurally rather than run
- Build environment setup (nix CC/AR wrappers, CARGO_HOME relocation) consumed time in Phase 1

### Patterns Established
- Policy assembly receives two worktree inputs: granted (wt_for_policy) and diagnostic (ungated_siblings)
- inject_yolo derivation pattern: `!cli.no_yolo && cli.session.is_none()`
- YOLO injection block placed before `cmd.args(agent_args)` to preserve argument ordering
- Unified conflict accumulator in `validate()` for session-incompatible flags

### Key Lessons
1. Sandbox agent environments cannot run `cargo test` due to write restrictions on CARGO_HOME -- plan for out-of-sandbox test verification
2. Small milestones with focused scope (2 phases, 4 files modified) ship faster and cleaner than broad scope

### Cost Observations
- Model mix: sonnet-dominated (research, planning, execution, verification agents all sonnet)
- Sessions: 3 (planning, phase 1 execution, phase 2 execution + audit)
- Notable: Phase 2 executed in 7 minutes -- shortest plan execution observed

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 3 | 2 | Initial milestone -- established sandbox hardening patterns |

### Cumulative Quality

| Milestone | Tests | Coverage | Files Changed |
|-----------|-------|----------|---------------|
| v1.0 | 55 total (13 new) | Unit + structural | 4 |

### Top Lessons (Verified Across Milestones)

1. (Single milestone -- lessons pending cross-validation)
