---
phase: 07-add-shared-agent-help-and-agent-skill-support-across-cli-crates
verified: 2026-03-23T01:36:53Z
status: passed
score: 5/5 must-haves verified
human_verification:
  - test: "Inspect each binary's `--agent-help` YAML for exhaustiveness and operator usefulness"
    expected: "Each document fully covers commands, arguments, examples, env/config/defaults, output shapes, failure modes, and operator mistakes for that tool."
    why_human: "Automated checks prove shared rendering, clap coverage, top-level-only behavior, and regression safety, but they cannot determine whether the authored prose is sufficiently exhaustive for real operator use."
  - test: "Inspect each binary's `--agent-skill` output as a ready-to-save Claude skill"
    expected: "Front matter and markdown body match the YAML source content, remain tool-specific, and include the operational caveats an operator would need."
    why_human: "Automated checks verify shared-source rendering and key strings, not overall completeness or usefulness as an operator-facing skill."
---

# Phase 7: Add shared `--agent-help` and `--agent-skill` support across CLI crates Verification Report

**Phase Goal:** All seven workspace binaries expose exhaustive top-level `--agent-help` YAML and `--agent-skill` Claude skill output from a shared `cli-common` contract without changing normal CLI behavior.
**Verified:** 2026-03-23T01:36:53Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

Phase-plan must-haves collapse into five phase-wide observable truths aligned to ADOC-01 through ADOC-05.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | `cli-common` provides one shared agent-doc contract with exact top-level detection, YAML rendering, skill rendering, and clap coverage helpers. | ✓ VERIFIED | `crates/cli-common/src/lib.rs:55-68` exports `agent_docs`; `crates/cli-common/src/agent_docs.rs:185,210,241,377,391` defines detection, both renderers, and coverage helpers. |
| 2 | All seven binaries are wired to the shared contract and emit docs before normal parser dispatch. | ✓ VERIFIED | Wiring exists in `bce` `src/main.rs:24-25,61-65,204`, `unvenv` `src/main.rs:130-131,189-193,198`, `gator` `src/main.rs:14-15,41-45,50`, `todoer` `src/main.rs:23-24,33-37`, `silent-critic` `src/main.rs:15-16,25-29`, `prompter` `src/main.rs:26,37-41`, and `asana-cli` `src/main.rs:7-8` plus `src/cli/mod.rs:181-192`. |
| 3 | `--agent-help` and `--agent-skill` are top-level-only, hidden from normal help, succeed on stdout, and reject subcommand placement. | ✓ VERIFIED | `cargo test --workspace agent_help` passed; each crate has `tests/agent_help.rs` with success, hidden-help, and misplaced-flag rejection assertions (`bce` 83 lines, `unvenv` 80, `gator` 80, `todoer` 73, `silent-critic` 73, `prompter` 81, `asana-cli` 87). |
| 4 | The YAML and skill outputs come from the same authored source while tool-specific docs stay aligned with real clap surfaces. | ✓ VERIFIED | Shared-source renderer tests passed in `cargo test -p tftio-cli-common agent_docs --lib`; per-crate authored doc builders exist in `prompter/src/lib.rs:168`, `todoer/src/cli.rs:71`, `silent-critic/src/cli.rs:223`, `asana-cli/src/cli/mod.rs:192`, `bce/src/main.rs:204`, `unvenv/src/main.rs:198`, and `gator/src/main.rs:50`; coverage assertions exist in `todoer/src/cli.rs:488-526`, `silent-critic/src/cli.rs:777-858`, `prompter/src/lib.rs:2491-2523`, and `asana-cli/src/cli/mod.rs:974-1025`. |
| 5 | Normal CLI behavior across the workspace remains intact after the rollout. | ✓ VERIFIED | `cargo test --workspace --verbose` exited 0 after running the full workspace suite, including non-agent-help tests such as Asana CLI command tests, `bce` query tests, and doctests. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/cli-common/src/agent_docs.rs` | Shared request detection, renderers, and coverage helpers | ✓ VERIFIED | Exists with 991 lines; substantive public API and tests at `:185-391` and `:786-989`. |
| `crates/cli-common/src/lib.rs` | Public module export and re-exports | ✓ VERIFIED | Exists with 100 lines; `pub mod agent_docs` and re-exports at `:55-68`. |
| `crates/bsky-comment-extractor/tests/agent_help.rs` | BCE regression coverage | ✓ VERIFIED | Exists with 83 lines; asserts top-level YAML/skill, hidden help, and subcommand rejection. |
| `crates/unvenv/tests/agent_help.rs` | Unvenv regression coverage | ✓ VERIFIED | Exists with 80 lines; asserts top-level YAML/skill, hidden help, and subcommand rejection. |
| `crates/gator/tests/agent_help.rs` | Gator regression coverage | ✓ VERIFIED | Exists with 80 lines; asserts top-level YAML/skill, hidden help, and positional misuse rejection. |
| `crates/todoer/tests/agent_help.rs` | Todoer regression coverage | ✓ VERIFIED | Exists with 73 lines; asserts top-level success, hidden help, and subcommand misuse rejection. |
| `crates/silent-critic/tests/agent_help.rs` | Silent-critic regression coverage | ✓ VERIFIED | Exists with 73 lines; asserts top-level success, hidden help, and subcommand misuse rejection. |
| `crates/prompter/tests/agent_help.rs` | Prompter regression coverage | ✓ VERIFIED | Exists with 81 lines; asserts top-level success, hidden help, and subcommand misuse rejection. |
| `crates/asana-cli/tests/agent_help.rs` | Asana CLI regression coverage | ✓ VERIFIED | Exists with 87 lines; asserts top-level success, hidden help, and subcommand misuse rejection. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `crates/cli-common/src/lib.rs` | `crates/cli-common/src/agent_docs.rs` | `pub mod` and re-exports | ✓ WIRED | `lib.rs:55-68` exposes the shared contract to downstream crates. |
| `crates/bsky-comment-extractor/src/main.rs` | `tftio_cli_common::agent_docs` | early argv interception + shared renderers | ✓ WIRED | `main.rs:24-25,61-65` calls `detect_agent_doc_request`, then renders shared YAML/skill from `build_agent_doc()`. |
| `crates/unvenv/src/main.rs` | `tftio_cli_common::agent_docs` | top-level request short-circuit | ✓ WIRED | `main.rs:130-131,189-193` short-circuits before clap dispatch. |
| `crates/gator/src/main.rs` | `tftio_cli_common::agent_docs` | pre-parse short-circuit before required positional validation | ✓ WIRED | `main.rs:14-15,41-45` handles docs before `Cli::parse_from` and `validate()`. |
| `crates/todoer/src/main.rs` | `crates/todoer/src/cli.rs` + `tftio_cli_common` | doc short-circuit before clap-required subcommand dispatch | ✓ WIRED | `todoer/src/main.rs:23-37` uses shared detector/renderers with `todoer::cli::agent_doc()`. |
| `crates/silent-critic/src/main.rs` | `crates/silent-critic/src/cli.rs` + `tftio_cli_common` | pre-dispatch doc path | ✓ WIRED | `silent-critic/src/main.rs:15-29` uses shared detector/renderers with `silent_critic::cli::agent_doc()`. |
| `crates/prompter/src/main.rs` | `crates/prompter/src/lib.rs` + `tftio_cli_common` | raw argv inspection before custom `AppMode` parsing | ✓ WIRED | `prompter/src/main.rs:26,37-41,57` handles docs before `parse_args_from`. |
| `crates/asana-cli/src/main.rs` | `crates/asana-cli/src/cli/mod.rs` + `tftio_cli_common` | agent-doc short-circuit before `Cli::parse()` | ✓ WIRED | `asana-cli/src/main.rs:7-8` dispatches to `cli::run_agent_doc_request`, which renders shared YAML/skill at `src/cli/mod.rs:181-192`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `cli-common` renderers | `doc` / `_doc` | Per-crate `agent_doc()` or `build_agent_doc()` | Yes — authored `AgentDoc` instances feed both renderers | ✓ FLOWING |
| `bce` doc path | `doc` | `build_agent_doc()` in `crates/bsky-comment-extractor/src/main.rs:204` | Yes — authored commands, env, paths, output shapes, failure modes | ✓ FLOWING |
| `unvenv` doc path | `doc` | `build_agent_doc()` in `crates/unvenv/src/main.rs:198` | Yes — authored scan/doctor/completions/update sections | ✓ FLOWING |
| `gator` doc path | `doc` | `build_agent_doc()` in `crates/gator/src/main.rs:50` | Yes — authored agent/policy/session behavior and outputs | ✓ FLOWING |
| `todoer` doc path | `doc` | `agent_doc()` in `crates/todoer/src/cli.rs:71` | Yes — authored `.todoer.toml`, project resolution, JSON envelopes, SQLite behavior | ✓ FLOWING |
| `silent-critic` doc path | `doc` | `agent_doc()` in `crates/silent-critic/src/cli.rs:223` | Yes — authored command tree, token/config/db/session-state details | ✓ FLOWING |
| `prompter` doc path | `doc` | `agent_doc()` in `crates/prompter/src/lib.rs:168` | Yes — authored recursive render/config/library/output guidance | ✓ FLOWING |
| `asana-cli` doc path | `doc` | `agent_doc()` in `crates/asana-cli/src/cli/mod.rs:192` | Yes — authored config/env/cache/network/output sections across the deep clap tree | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Shared contract rejects subcommand placement and renders both formats from one source | `cargo test -p tftio-cli-common agent_docs --lib` | 5 tests passed; includes exact top-level detection, stable YAML ordering, same-source skill rendering, and coverage-helper checks | ✓ PASS |
| All seven binaries expose hidden top-level-only agent-doc behavior | `cargo test --workspace agent_help` | Passed crate-level `agent_help` suites for `asana-cli`, `bce`, `gator`, `prompter`, `silent-critic`, `todoer`, and `unvenv` | ✓ PASS |
| Normal CLI behavior remains intact after rollout | `cargo test --workspace --verbose` | Full workspace suite passed, including non-agent-doc tests and doctests | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `ADOC-01` | `07-01` | `cli-common` defines the canonical YAML agent-doc schema and shared renderer used by every binary for `--agent-help` | ✓ SATISFIED | Shared API exists in `crates/cli-common/src/agent_docs.rs` and is re-exported from `crates/cli-common/src/lib.rs:55-68`; shared tests passed via `cargo test -p tftio-cli-common agent_docs --lib`. |
| `ADOC-02` | `07-02`, `07-03`, `07-04` | Each Phase 7 binary authors exhaustive tool-specific agent docs covering commands, arguments, examples, outputs, env/config/defaults, failure modes, and likely operator mistakes | ✓ SATISFIED (automated) / human review still needed for exhaustiveness judgment | Authored builders exist in all seven binaries; integration tests assert tool-specific content strings; coverage helper tests exist in `prompter`, `todoer`, `silent-critic`, and `asana-cli`. |
| `ADOC-03` | `07-01`, `07-02`, `07-03`, `07-04` | `--agent-skill` renders the same underlying agent-doc content as a ready-to-save Claude-style skill file | ✓ SATISFIED | `render_agent_skill` is shared in `cli-common/src/agent_docs.rs:241`; same-source rendering test passed in `cargo test -p tftio-cli-common agent_docs --lib`; every crate routes top-level requests through shared renderers. |
| `ADOC-04` | `07-01`, `07-02`, `07-03`, `07-04` | `--agent-help` and `--agent-skill` are hidden top-level-only flags that print to stdout on success, exit `0`, and stay out of normal help output | ✓ SATISFIED | `detect_agent_doc_request` accepts only exact two-argument invocations (`cli-common/src/agent_docs.rs:185-206`); `cargo test --workspace agent_help` passed hidden-help and misplaced-flag rejection assertions for all seven binaries. |
| `ADOC-05` | `07-02`, `07-03`, `07-04` | All seven workspace binaries expose the shared agent-doc behavior through `cli-common` plus per-crate wiring | ✓ SATISFIED | Workspace members are the seven binaries in `Cargo.toml`; all seven crate `Cargo.toml` files depend on `tftio-cli-common`; all seven mains/libs are wired; `cargo test --workspace agent_help` passed across the workspace. |

**Plan frontmatter IDs accounted for:** `ADOC-01`, `ADOC-02`, `ADOC-03`, `ADOC-04`, `ADOC-05`

**Orphaned Phase 7 requirements:** None. `REQUIREMENTS.md` maps exactly `ADOC-01` through `ADOC-05` to Phase 7.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| None | - | No TODO/FIXME/stub markers found in inspected Phase 7 implementation and regression files | ℹ️ Info | No blocker or warning anti-patterns detected from the scan. |

### Human Verification

### 1. Exhaustiveness review of canonical YAML output

**Test:** Run each binary with `--agent-help` (`prompter`, `unvenv`, `asana-cli`, `todoer`, `silent-critic`, `gator`, `bce`) and inspect the YAML.
**Expected:** Each document is fully tool-specific and covers commands, flags, examples, env/config/defaults, output shapes, failure guidance, and operator mistakes without obvious omissions.
**Why human:** Coverage helpers and string assertions prove structure and selected content, not whether the prose is truly exhaustive for a real operator.

### 2. Exhaustiveness review of Claude skill output

**Test:** Run each binary with `--agent-skill` and inspect the front matter and markdown body.
**Expected:** Each skill file preserves the same operational facts as the YAML, is ready to save, and includes the caveats an operator needs.
**Why human:** Automated checks verify shared-source rendering and sampled content, not end-to-end usefulness as a skill document.

Human verification was approved on 2026-03-23 after reviewing the generated `--agent-help` and
`--agent-skill` outputs.

### Gaps Summary

No implementation gaps were found.

---

_Verified: 2026-03-23T01:36:53Z_
_Verifier: Claude (gsd-verifier)_
