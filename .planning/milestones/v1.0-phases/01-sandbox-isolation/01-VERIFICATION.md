---
phase: 01-sandbox-isolation
verified: 2026-03-18T00:00:00Z
status: passed
score: 7/7 must-haves verified
re_verification: false
---

# Phase 1: Sandbox Isolation Verification Report

**Phase Goal:** Agents are isolated to their own worktree by default; users who need cross-worktree reads have a clear opt-in
**Verified:** 2026-03-18
**Status:** PASSED
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Launching gator in a linked worktree does not grant RO access to sibling worktrees by default | VERIFIED | lib.rs:68-78 strips siblings from wt_for_policy when share_worktrees is false; sandbox.rs:115-120 only emits grants for worktree_info.siblings (empty by default) |
| 2 | Common git dir still receives RW access for commits, index, refs | VERIFIED | lib.rs:74-77 preserves common_dir in wt_for_policy regardless of share_worktrees; sandbox.rs:107-112 emits RW grant for common_dir unconditionally |
| 3 | Running gator --share-worktrees grants RO access to all peer worktrees | VERIFIED | lib.rs:69-70 passes wt_info (full siblings) through when share_worktrees is true; sandbox.rs:115-120 emits emit_ro_grant for each sibling in worktree_info.siblings; test assemble_policy_with_siblings_granted confirms grant presence |
| 4 | Manual grants (.safehouse, --add-dirs-ro, --policy) still work for sibling paths without --share-worktrees | VERIFIED | lib.rs:49-63 loads safehouse_extras, policy extras, and cli.add_dirs_ro into extras before gating; gating only affects wt_for_policy.siblings, extras are unaffected; sandbox.rs:136-147 always emits extra_dirs grants |
| 5 | Session mode is unaffected by sandbox isolation changes | VERIFIED | lib.rs:38-44 session branch sets wt_for_policy = WorktreeInfo::default() and ungated_siblings = Vec::new() -- no change in behavior from pre-phase baseline |
| 6 | --share-worktrees and --session are mutually exclusive | VERIFIED | cli.rs:100-102 adds "--share-worktrees" to conflicts when share_worktrees is true inside session.is_some() block; test validate_share_worktrees_with_session asserts err.contains("--share-worktrees") |
| 7 | --dry-run shows detected-but-not-granted siblings as SBPL comments | VERIFIED | sandbox.rs:122-133 emits ";; Sibling worktree (not granted): {path}" for each ungated_sibling; lib.rs:90-93 prints policy to stderr on dry_run; test assemble_policy_no_siblings_by_default asserts comment present and RO grant absent |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/gator/src/cli.rs` | --share-worktrees CLI flag and validation | VERIFIED | Line 61: `pub share_worktrees: bool`; line 57-60: `#[arg(long)]` doc comment; lines 100-102: session conflict check; tests parse_share_worktrees and validate_share_worktrees_with_session present |
| `crates/gator/src/lib.rs` | Sibling gating logic before policy assembly | VERIFIED | Lines 66-79: full gating split into wt_for_policy / ungated_siblings; line 14: `use std::path::PathBuf;`; line 86: assemble_policy call passes ungated_siblings |
| `crates/gator/src/sandbox.rs` | detected-but-not-granted sibling SBPL comments in assemble_policy | VERIFIED | Lines 84-89: signature includes `ungated_siblings: &[PathBuf]`; lines 122-133: loop emits ";; Sibling worktree (not granted): {}" comments |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `crates/gator/src/lib.rs` | `crates/gator/src/sandbox.rs` | assemble_policy call with ungated_siblings parameter | VERIFIED | lib.rs line 86: `sandbox::assemble_policy(&workdir, &wt_for_policy, &extras, &denies, &ungated_siblings)` -- pattern matches `assemble_policy.*ungated` |
| `crates/gator/src/cli.rs` | `crates/gator/src/lib.rs` | cli.share_worktrees field read in run() | VERIFIED | lib.rs line 68: `if cli.share_worktrees {` -- pattern matches `cli\.share_worktrees` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| SAND-01 | 01-01-PLAN.md | Sibling worktree RO grants not added to policy by default | SATISFIED | lib.rs:68-78 strips siblings when share_worktrees is false; sandbox test assemble_policy_no_siblings_by_default asserts no grant for sibling path |
| SAND-02 | 01-01-PLAN.md | Common git dir RW grant preserved for linked worktrees | SATISFIED | lib.rs:74-77 preserves common_dir in gated WorktreeInfo; sandbox test assemble_policy_common_dir_rw_preserved confirms RW grant present |
| SAND-03 | 01-01-PLAN.md | --share-worktrees CLI flag opts in to RO access for all peer worktrees | SATISFIED | cli.rs:57-61 defines flag; lib.rs:68-70 passes siblings through when true; sandbox test assemble_policy_with_siblings_granted confirms RO grant for siblings |
| COMPAT-01 | 01-01-PLAN.md | Existing --add-dirs-ro, .safehouse, and --policy mechanisms continue to work | SATISFIED | lib.rs:49-63 loads all manual grant sources into extras before gating; extras flow directly to assemble_policy unfiltered |
| COMPAT-02 | 01-01-PLAN.md | Session mode behavior unchanged | SATISFIED | lib.rs:38-44 session branch uses WorktreeInfo::default() and empty ungated_siblings -- identical behavior to pre-phase |

No orphaned requirements. REQUIREMENTS.md maps SAND-01, SAND-02, SAND-03, COMPAT-01, COMPAT-02 all to Phase 1. All five were claimed in 01-01-PLAN.md and all five are satisfied.

### Anti-Patterns Found

None. No TODO/FIXME/HACK/PLACEHOLDER comments, no stub return values, no unimplemented! calls found in the three modified files.

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| -- | -- | -- | -- | -- |

### Human Verification Required

#### 1. Dry-run in a real linked worktree without --share-worktrees

**Test:** Navigate to a linked git worktree. Run `cargo run -p tftio-gator -- claude --dry-run --no-prompt 2>&1 | grep -i sibling`
**Expected:** Output contains ";; Sibling worktree (not granted):" lines for each peer worktree and zero `(allow file-read* (subpath ...))` grants for sibling paths.
**Why human:** Requires a live git worktree environment; the diagnostic output path (eprint! on dry_run) cannot be exercised by unit tests without a real base profile.

#### 2. Dry-run in a real linked worktree with --share-worktrees

**Test:** Same directory. Run `cargo run -p tftio-gator -- claude --share-worktrees --dry-run --no-prompt 2>&1 | grep -i sibling`
**Expected:** Output contains `(allow file-read* (subpath "/path/to/sibling"))` grants for each peer worktree. No "(not granted)" comments.
**Why human:** Same reason as above -- base profile must exist for assemble_policy to succeed.

### Gaps Summary

No gaps. All seven observable truths are verified, all three artifacts are substantive and wired, both key links are confirmed, and all five requirement IDs are satisfied. The commit hash 4265f5b is present in git history and contains the correct files modified (cli.rs, lib.rs, sandbox.rs, Cargo.lock). No anti-patterns found. 46 tests passing per SUMMARY (human checkpoint approved).

The two human verification items above are confirmatory, not blocking -- the code logic is correct and the automated tests cover the core behavior.

---

_Verified: 2026-03-18_
_Verifier: Claude (gsd-verifier)_
