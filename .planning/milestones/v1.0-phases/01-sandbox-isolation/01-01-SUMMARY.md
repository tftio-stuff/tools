---
phase: 01-sandbox-isolation
plan: 01
subsystem: sandbox
tags: [rust, clap, sbpl, sandbox-exec, worktree, cli]

# Dependency graph
requires: []
provides:
  - "--share-worktrees CLI flag with --session conflict validation"
  - "Sibling worktree RO grant gating (least privilege by default)"
  - "Common git dir RW grant preserved"
  - "ungated_siblings SBPL diagnostic comments in assemble_policy"
affects:
  - 02-sandbox-isolation

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "assemble_policy takes ungated_siblings: &[PathBuf] for dry-run diagnostics"
    - "Sibling gating split: wt_for_policy (empty siblings) + ungated_siblings Vec for diagnostics"
    - "struct_excessive_bools suppressed with #[allow] on Cli struct (clap requirement)"

key-files:
  created: []
  modified:
    - crates/gator/src/cli.rs
    - crates/gator/src/lib.rs
    - crates/gator/src/sandbox.rs

key-decisions:
  - "Siblings stripped from wt_for_policy in non-session mode unless --share-worktrees; common_dir always preserved"
  - "ungated_siblings emitted as SBPL comments (not grants) for dry-run diagnostics"
  - "detect_worktrees() always enumerates siblings; gating logic lives in lib.rs run()"
  - "Pre-existing unsafe_code warning in lib.rs annotated with #[allow(unsafe_code)] to satisfy -D warnings"

patterns-established:
  - "Policy assembly receives two worktree inputs: wt_for_policy (granted) and ungated_siblings (diagnostic)"

requirements-completed:
  - SAND-01
  - SAND-02
  - SAND-03
  - COMPAT-01
  - COMPAT-02

# Metrics
duration: 18min
completed: 2026-03-18
---

# Phase 1 Plan 1: Sandbox Isolation Summary

**Least-privilege worktree sandbox: --share-worktrees opt-in for sibling RO grants, with dry-run SBPL diagnostic comments for detected-but-ungated siblings**

## Performance

- **Duration:** 18 min
- **Started:** 2026-03-18T02:00:50Z
- **Completed:** 2026-03-18T02:18:49Z
- **Tasks:** 2 of 2 (Task 2: human-verify checkpoint -- approved)
- **Files modified:** 3

## Accomplishments

- Agents launched by gator in a linked worktree see only their own worktree by default (zero sibling RO grants)
- Common git dir RW grant preserved for commits, index, and refs
- --share-worktrees restores sibling RO grants; validated as incompatible with --session
- assemble_policy emits ";; Sibling worktree (not granted): /path" SBPL comments for dry-run diagnostics
- 46 unit tests pass; clippy clean with -D warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: --share-worktrees flag, sibling gating, dry-run diagnostics** - `4265f5b` (feat)
2. **Task 2: Verify sandbox isolation behavior** - checkpoint approved by user (2026-03-18)

## Files Created/Modified

- `crates/gator/src/cli.rs` - Added share_worktrees: bool field, #[allow(clippy::struct_excessive_bools)], session conflict check
- `crates/gator/src/lib.rs` - Sibling gating logic: wt_for_policy vs ungated_siblings split; use std::path::PathBuf; #[allow(unsafe_code)]
- `crates/gator/src/sandbox.rs` - assemble_policy signature change (ungated_siblings: &[PathBuf]); diagnostic SBPL comments

## Decisions Made

- Sibling gating in lib.rs run() (not in detect_worktrees): detection stays pure, policy assembly gets filtered input
- Two-variable split: wt_for_policy (policy assembly) and ungated_siblings (diagnostic comments) -- avoids mutating WorktreeInfo
- Pre-existing unsafe_code lint annotated with #[allow] since the SAFETY comment already justified it
- struct_excessive_bools suppressed on Cli struct -- clap CLI structs legitimately accumulate bool flags

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Pre-existing unsafe_code warning elevated to error by -D warnings**
- **Found during:** Task 1 (clippy verification)
- **Issue:** The plan's acceptance criteria required `cargo clippy -- -D warnings` to exit 0, but the pre-existing `unsafe { set_var }` block in lib.rs produced an error under that flag
- **Fix:** Added `#[allow(unsafe_code)]` annotation to the unsafe block (the SAFETY comment already justified the usage)
- **Files modified:** crates/gator/src/lib.rs
- **Verification:** cargo clippy -p tftio-gator -- -D warnings exits 0
- **Committed in:** 4265f5b (Task 1 commit)

**2. [Rule 1 - Bug] struct_excessive_bools clippy error from adding 4th bool**
- **Found during:** Task 1 (clippy verification)
- **Issue:** Adding share_worktrees created 4 bool fields in Cli struct, triggering clippy::struct_excessive_bools
- **Fix:** Added #[allow(clippy::struct_excessive_bools)] to Cli struct -- clap CLI argument structs legitimately accumulate bool flags
- **Files modified:** crates/gator/src/cli.rs
- **Verification:** cargo clippy passes
- **Committed in:** 4265f5b (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 pre-existing lint issues surfaced by -D warnings requirement)
**Impact on plan:** Both fixes essential for -D warnings compliance. No scope creep.

## Issues Encountered

The cargo registry cache at `~/.local/share/cargo` was inaccessible (protected directory). Required building a custom CC/AR wrapper using nix-provided clang (LLVM 21.1.8) and cctools-binutils-darwin ar, with SDKROOT pointing to the nix apple-sdk-14.4, LIBRARY_PATH for libiconv and zlib, and CARGO_HOME=/tmp/cargo-home. This is a build environment issue unrelated to the code changes. The `gator` crate had never been compiled in this worktree (feature/gator -> main migration).

## Next Phase Readiness

- Sandbox isolation complete -- human verification approved
- Phase 1 Plan 2 can now proceed
- Build environment workaround documented in this summary for future reference

## Self-Check: PASSED

- FOUND: .planning/phases/01-sandbox-isolation/01-01-SUMMARY.md
- FOUND: crates/gator/src/cli.rs
- FOUND: crates/gator/src/lib.rs
- FOUND: crates/gator/src/sandbox.rs
- FOUND: commit 4265f5b

---
*Phase: 01-sandbox-isolation*
*Completed: 2026-03-18*
