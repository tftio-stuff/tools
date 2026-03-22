---
phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding
plan: 04
subsystem: testing
tags: [rust, just, shell, cli-common, documentation]
requires:
  - phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding
    provides: shared metadata, runner, response, and doctor helpers already adopted by workspace tools
provides:
  - repository shell enforcement for the final cli-common boundary
  - documented final shared-vs-local CLI boundary for future tool changes
  - green workspace verification across shell contracts, tests, and lint
affects: [cli-consistency, cli-metadata-consistency, CLAUDE.md, PROJECT.md]
tech-stack:
  added: []
  patterns:
    - shell CLI contracts exercise shared metadata, JSON, doctor, and boilerplate boundaries
    - documentation names intentional crate-local exceptions explicitly
key-files:
  created:
    - scripts/test-cli-metadata-consistency.sh
    - tests/cli/01-metadata-commands.sh
    - tests/cli/02-json-contracts.sh
    - tests/cli/03-primary-flows.sh
    - tests/cli/04-prompter-doctor.sh
    - tests/cli/05-shared-boilerplate.sh
    - tests/cli/lib.sh
  modified:
    - justfile
    - CLAUDE.md
    - .planning/PROJECT.md
key-decisions:
  - "`just cli-consistency` now routes through `just cli-metadata-consistency` so repository enforcement has one shell-suite entrypoint."
  - "The final documented boundary names the remaining crate-local exceptions explicitly instead of describing them only by tool family."
  - "HOME-isolated shell tests preserve `CARGO_HOME` and `RUSTUP_HOME` so cargo can build tools while commands read isolated user state."
patterns-established:
  - "Shell contract suite: add behavior checks first, then source-level boundary assertions for deleted glue patterns."
  - "CLI boundary docs: record both what moved into `cli-common` and the narrow exceptions that remain local."
requirements-completed: [CLI-SHARE-04, CLI-SHARE-05, CLI-SHARE-06]
duration: 2m12s
completed: 2026-03-22
---

# Phase 03 Plan 04: Final CLI boundary enforcement Summary

**Repository shell contracts now enforce the final `cli-common` boundary, and workspace guidance names the remaining crate-local CLI exceptions after a green full-suite verification run.**

## Performance

- **Duration:** 2m12s
- **Started:** 2026-03-22T22:46:39Z
- **Completed:** 2026-03-22T22:48:51Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Added a repository shell suite that exercises shared metadata, JSON/error contracts, doctor rendering, primary command flows, and deleted boilerplate patterns.
- Routed `just cli-consistency` through the shared shell suite entrypoint so the repository enforces the same CLI boundary in one place.
- Updated `CLAUDE.md` and `.planning/PROJECT.md` to describe the final `cli-common` surface and the remaining intentional per-tool exceptions.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend repository shell enforcement for the newly deleted glue patterns** - `829bee8` (test)
2. **Task 2: Document the final boundary and run the full verification suite** - `ae75526` (docs)

**Plan metadata:** Pending final docs commit for this summary and planning-state updates.

## Files Created/Modified
- `scripts/test-cli-metadata-consistency.sh` - Runs the repository shell suite across numbered CLI contract scripts.
- `tests/cli/01-metadata-commands.sh` - Verifies shared metadata commands and completion generation across tools.
- `tests/cli/02-json-contracts.sh` - Verifies shared JSON success/error envelopes, including `silent-critic` response emission.
- `tests/cli/03-primary-flows.sh` - Guards primary command trees and non-metadata invocation syntax for migrated tools.
- `tests/cli/04-prompter-doctor.sh` - Verifies shared doctor output plus prompter-specific completion augmentation.
- `tests/cli/05-shared-boilerplate.sh` - Guards deleted boilerplate patterns and shared-helper adoption at the source level.
- `tests/cli/lib.sh` - Provides shell helpers for isolated-HOME runs and file-content assertions.
- `justfile` - Routes `cli-consistency` through the shared shell suite entrypoint before direct cargo smoke checks.
- `CLAUDE.md` - Documents the final `cli-common` boundary and the remaining intentional local exceptions.
- `.planning/PROJECT.md` - Records Phase 03 completion and the final shared/local split.

## Decisions Made
- Kept the shell suite behavior-first by exercising commands directly, then added targeted source assertions only for boundary drift that behavior checks cannot catch cleanly.
- Documented exceptions per tool (`gator`, `bce`, `todoer`, `prompter`, `unvenv`, `asana-cli`) so future work has an explicit final boundary instead of an implicit one.
- Reused HOME-isolated shell runs for doctor/JSON checks while pinning cargo toolchain paths to avoid rustup lookup failures.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Preserved cargo toolchain paths for HOME-isolated shell tests**
- **Found during:** Task 1 (Extend repository shell enforcement for the newly deleted glue patterns)
- **Issue:** Shell tests that overrode `HOME` caused `cargo run` to fail because rustup could not resolve the toolchain from the temporary HOME directory.
- **Fix:** Updated `run_with_home` in `tests/cli/lib.sh` to preserve `CARGO_HOME` and `RUSTUP_HOME` while still isolating tool runtime state.
- **Files modified:** `tests/cli/lib.sh`
- **Verification:** `just cli-metadata-consistency && just cli-consistency`
- **Committed in:** `829bee8` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix was required to make repository shell enforcement runnable under isolated HOME setups. No scope expansion.

## Issues Encountered
- `unvenv doctor` renders a tool-specific health-check header (`unvenv health check`) rather than the shared `tools health check` text used by some other tools, so the new doctor assertion checks the shared health-check pattern instead of one literal header string.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 03 now has repository enforcement, explicit documentation, and a green full-suite verification pass.
- Future CLI changes can extend `cli-common` or local tool behavior against the documented exception list and the shell contract suite.

## Self-Check: PASSED

- Confirmed `.planning/phases/03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding/03-04-SUMMARY.md` exists.
- Confirmed task commits `829bee8` and `ae75526` exist in `git log --oneline --all`.

---
*Phase: 03-extract-remaining-cli-glue-into-cli-common-metadata-mapping-fatal-runners-response-emitters-and-doctor-scaffolding*
*Completed: 2026-03-22*
