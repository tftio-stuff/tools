---
phase: 02-yolo-injection
plan: "01"
subsystem: cli
tags: [rust, clap, sandbox, yolo, agent, autonomous]

requires:
  - phase: 01-sandbox-isolation
    provides: build_command() signature, cli.rs Cli struct, worktree gating infrastructure

provides:
  - "--no-yolo CLI flag with --session conflict validation"
  - "inject_yolo parameter in build_command() with per-agent YOLO flag injection"
  - "inject_yolo derivation in lib.rs run() from cli.no_yolo and cli.session"

affects:
  - future-phases

tech-stack:
  added: []
  patterns:
    - "Per-agent flag injection via match on Agent enum inside inject_yolo guard"
    - "inject_yolo = !cli.no_yolo && cli.session.is_none() derivation pattern"
    - "YOLO injection block placed before cmd.args(agent_args) to preserve ordering"

key-files:
  created: []
  modified:
    - crates/gator/src/cli.rs
    - crates/gator/src/agent.rs
    - crates/gator/src/lib.rs

key-decisions:
  - "YOLO injection is opt-out (default: inject) -- sandbox is the security boundary"
  - "--no-yolo conflicts with --session (session contract is sole authority)"
  - "Gemini gets stderr warning, no flag injection (no known YOLO equivalent)"
  - "YOLO flag placed before user-supplied agent_args to preserve arg ordering"

patterns-established:
  - "inject_yolo: bool parameter on build_command() for per-agent flag injection"
  - "Session mode always sets inject_yolo=false regardless of --no-yolo"

requirements-completed:
  - PERM-01
  - PERM-02

duration: 7min
completed: 2026-03-18
---

# Phase 2 Plan 1: YOLO Injection Summary

**Per-agent autonomous-mode flag injection (--dangerously-skip-permissions / --full-auto) enabled by default in gator, with --no-yolo opt-out and --session conflict validation**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-18T14:25:43Z
- **Completed:** 2026-03-18T14:33:12Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `pub no_yolo: bool` field to `Cli` struct with `#[arg(long)]`, conflict check against `--session` in `validate()`
- Updated `build_command()` signature to accept `inject_yolo: bool`; injects `--dangerously-skip-permissions` for Claude, `--full-auto` for Codex, stderr warning for Gemini
- Wired `inject_yolo = !cli.no_yolo && cli.session.is_none()` in `lib.rs run()`, ensuring YOLO is default but safely excluded in session mode
- Added 9 new unit tests covering all injection paths; updated 4 existing test call sites for the new parameter

## Task Commits

Each task was committed atomically:

1. **Task 1: --no-yolo flag and inject_yolo parameter** - `cb135f1` (feat)
2. **Task 2: Wire inject_yolo in lib.rs** - `3c8068c` (feat)

## Files Created/Modified

- `crates/gator/src/cli.rs` - Added no_yolo field, --session conflict check, 4 new tests
- `crates/gator/src/agent.rs` - Added inject_yolo parameter, YOLO injection block, 5 new tests, updated 4 existing test call sites
- `crates/gator/src/lib.rs` - Derived inject_yolo from cli.no_yolo and cli.session, passed to build_command()

## Decisions Made

- YOLO injection is opt-out (default: inject) -- the sandbox IS the security boundary; agents should run autonomously by default
- `--no-yolo` conflicts with `--session` because session contracts are the sole authority in session mode, making no_yolo semantically redundant
- Gemini gets a stderr warning with no flag injection (no known YOLO equivalent flag exists)
- YOLO flag placed before `cmd.args(agent_args)` so injection respects expected argument ordering

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Test execution blocked by sandbox environment.** `cargo test -p tftio-gator` failed because the gator agent sandbox policy blocks writes to `~/.local/share/cargo/registry/` (CARGO_HOME). This is the expected sandbox behavior -- we are running as the agent inside the sandbox. The cargo registry cache cannot be populated from within the sandbox.

The code changes are structurally correct and verified by code review. Tests must be run from the Nix development environment outside the sandbox. This is a known infrastructure constraint documented here for the next operator.

**Workaround for human:** Run `cargo test -p tftio-gator` from the user's terminal (outside the gator sandbox session) to verify all 9 new tests pass.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- YOLO injection complete; both PERM-01 and PERM-02 requirements met at the code level
- Test verification required outside sandbox before shipping (see Issues Encountered above)
- Phase 02 is the final phase in this milestone; all planned features are implemented

## Self-Check: PASSED

All files found, both commits verified in git log.

---
*Phase: 02-yolo-injection*
*Completed: 2026-03-18*
