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

## Milestone: v1.1 -- bsky-comment-extractor

**Shipped:** 2026-03-22
**Phases:** 2 | **Plans:** 4 | **Tasks:** 8

### What Was Built
- AT Protocol client with `createSession` auth, handle-to-DID resolution, exhaustive `listRecords` pagination, and rate-limit backoff
- SQLite storage layer with idempotent upsert, cursor persistence for resumable extraction
- `bce` CLI binary with clap args, XDG default paths, indicatif spinner, `--since` date filtering, completion summary line

### What Worked
- Two-phase split (library then CLI) kept plans focused and dependencies clean
- Plan checker caught a missing `create_dir_all` for parent directory creation before it reached execution
- Cross-phase API contract (FetchSummary fields, progress callback signature) documented in plan interfaces block -- Wave 2 consumed Wave 1 without issues
- 32 tests (unit + integration stubs) written alongside implementation

### What Was Inefficient
- `just ci` fails workspace-wide due to pre-existing rustfmt violations in unrelated crates -- Phase 4 executor had to apply fmt fixes to Phase 3 files
- SUMMARY.md `one_liner` and `requirements_completed` frontmatter fields inconsistently populated by executors -- 04-02 had empty requirements list
- VALIDATION.md `nyquist_compliant` frontmatter never updated to `true` after execution

### Patterns Established
- Sync `fn main()` + `tokio::runtime::Builder::new_current_thread()` for async CLIs (not `#[tokio::main]`)
- `Option<&dyn Fn(u64)>` progress callback pattern for library-to-CLI spinner wiring
- `db_has_uri` pre-check before upsert to distinguish new vs existing records
- XDG paths via `directories::ProjectDirs` with `create_dir_all` on parent

### Key Lessons
1. Plan checker verification loop catches real issues -- the `create_dir_all` blocker would have caused first-run failures
2. Library API changes (adding fields, callback params) should be a separate plan from CLI consumption -- vertical slicing doesn't work when the API must change first
3. SUMMARY frontmatter fields need executor enforcement -- empty `requirements_completed` reduces audit automation reliability

### Cost Observations
- Model mix: opus (planner), sonnet (researcher, executor, checker, verifier, integration checker)
- Sessions: 2 (planning + execution in one session, audit in same session)
- Notable: Plan checker revision loop completed in 1 iteration (blocker + warning fixed on first pass)

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 3 | 2 | Initial milestone -- established sandbox hardening patterns |
| v1.1 | 2 | 2 | New crate from scratch -- library + CLI split pattern |

### Cumulative Quality

| Milestone | Tests | Coverage | Files Changed |
|-----------|-------|----------|---------------|
| v1.0 | 55 total (13 new) | Unit + structural | 4 |
| v1.1 | 32 total (32 new) | Unit + integration stubs | 26 |

### Top Lessons (Verified Across Milestones)

1. Small, focused milestones (2 phases each) ship cleanly -- validated in both v1.0 and v1.1
2. Plan checker verification catches real issues before execution -- validated in v1.1 (create_dir_all blocker)
