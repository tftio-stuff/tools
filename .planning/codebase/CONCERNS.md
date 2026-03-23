# Codebase Concerns

**Analysis Date:** 2026-03-22

## Tech Debt

**Documentation drift across the workspace:**
- Issue: `CLAUDE.md`, `CRATES.md`, and `README.md` do not describe the current workspace accurately.
- Files: `CLAUDE.md`, `CRATES.md`, `README.md`, `Cargo.toml`, `install.sh`
- Impact: Operator-facing docs describe 5-6 tools while `Cargo.toml` and `install.sh` expose 8 workspace members and 7 binaries. New work is easy to place against stale assumptions.
- Fix approach: Reconcile crate counts, binary lists, versions, and install instructions with `Cargo.toml`, `release-please-config.json`, and `install.sh`.

**Large multi-purpose modules:**
- Issue: several files concentrate too much behavior in single modules.
- Files: `crates/asana-cli/src/cli/task.rs`, `crates/prompter/src/lib.rs`, `crates/asana-cli/src/api/client.rs`, `crates/silent-critic/src/commands/session.rs`, `crates/silent-critic/src/db.rs`
- Impact: Review cost is high, localized changes have broad regression risk, and small feature additions require loading large files into context.
- Fix approach: Split command parsing, API operations, state transitions, and DB queries into narrower modules with smaller public surfaces.

**Lint exceptions are accumulating in newer crates:**
- Issue: workspace lint strictness is bypassed in `todoer` and `silent-critic` with broad allow-lists.
- Files: `crates/todoer/Cargo.toml`, `crates/silent-critic/Cargo.toml`, `Cargo.toml`
- Impact: New code can keep landing without the documentation and refactoring pressure enforced elsewhere in the workspace.
- Fix approach: Reduce crate-local `allow` lists incrementally and track which exceptions remain necessary.

## Known Bugs

**`bce --agent-help` is still placeholder output:**
- Symptoms: the command prints `status: pending_phase_06` instead of a real agent reference.
- Files: `crates/bsky-comment-extractor/src/main.rs`, `.planning/PROJECT.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`
- Trigger: run `bce --agent-help`
- Workaround: none in the binary; users must read source and planning docs.

**Broken absolute links in repository docs:**
- Symptoms: multiple links point at `/Users/jfb/Projects/tools/feature/gator/...`, which does not exist in this checkout.
- Files: `CLAUDE.md`, `CRATES.md`, `docs/the_silent_critic.md`, `docs/the_silent_critic_tooling_design.md`
- Trigger: follow the linked paths from the markdown files.
- Workaround: manually navigate to the equivalent files under `docs/` and the repository root.

## Security Considerations

**Arbitrary shell execution from stored criteria:**
- Risk: `silent-critic` executes `criterion.check_spec` via `sh -c`, and criteria can be created, updated, or imported from files.
- Files: `crates/silent-critic/src/commands/session.rs`, `crates/silent-critic/src/commands/criterion.rs`
- Current mitigation: session state and token checks gate who can submit evidence.
- Recommendations: treat criteria as trusted code only, add allowlist or executor abstraction, and avoid direct shell evaluation for common checks.

**Session tokens are exposed and stored in plaintext:**
- Risk: `operator_token` is printed to stdout on `session new`, `worker_token` is returned on `session go`, and both are persisted as plain text in SQLite.
- Files: `crates/silent-critic/src/main.rs`, `crates/silent-critic/src/db.rs`
- Current mitigation: opaque random UUID-based tokens.
- Recommendations: redact tokens from normal output, hash or encrypt stored tokens, and limit token reuse windows.

**Default autonomous-mode injection in `gator`:**
- Risk: `gator` injects `--dangerously-skip-permissions` or `--full-auto` unless `--no-yolo` is set.
- Files: `crates/gator/src/lib.rs`, `crates/gator/src/agent.rs`, `crates/gator/src/cli.rs`
- Current mitigation: sandbox policy generation exists, and session mode disables YOLO injection.
- Recommendations: make dangerous mode opt-in, not default, and surface the final agent command more explicitly before exec.

## Performance Bottlenecks

**Offset pagination will degrade on large `bce` databases:**
- Problem: query mode uses `LIMIT ? OFFSET ?` against `posts`.
- Files: `crates/bsky-comment-extractor/src/db.rs`, `crates/bsky-comment-extractor/src/main.rs`
- Cause: deep offsets force SQLite to walk past skipped rows before returning the requested page.
- Improvement path: add keyset pagination on `(created_at, uri)` or resumable cursors for agent consumers.

**Full raw JSON storage increases DB growth quickly:**
- Problem: every fetched record stores full `raw_json` alongside curated columns.
- Files: `crates/bsky-comment-extractor/src/db.rs`
- Cause: each post is duplicated as extracted fields plus full payload text.
- Improvement path: document expected DB size, compress or prune fields, or move raw payload storage behind an option.

## Fragile Areas

**`gator` depends on external binaries and per-user files at runtime:**
- Files: `crates/gator/src/agent.rs`, `crates/gator/src/session.rs`, `crates/gator/src/sandbox.rs`
- Why fragile: successful execution requires `sandbox-exec`, agent CLIs, `silent-critic`, and `~/.config/sandbox-exec/agent.sb`. Missing pieces fail only at runtime.
- Safe modification: keep interface boundaries narrow and add preflight validation before exec.
- Test coverage: unit tests validate command construction and parsing, but there is no `crates/gator/tests/` end-to-end coverage.

**Cross-crate `gator` ↔ `silent-critic` contract is unverified end to end:**
- Files: `crates/gator/src/session.rs`, `crates/silent-critic/src/main.rs`, `crates/silent-critic/src/commands/session.rs`
- Why fragile: `gator` shells out to `silent-critic session sandbox ... --format json` and trusts the JSON shape.
- Safe modification: version or schema-check the JSON payload before rollout.
- Test coverage: only parse-level tests exist; no integration test exercises both binaries together.

**Repository planning state is BCE-specific while the workspace is broader:**
- Files: `.planning/STATE.md`, `.planning/PROJECT.md`, `.planning/ROADMAP.md`
- Why fragile: shared repo guidance can bias future automation toward `bsky-comment-extractor` assumptions and away from other crates.
- Safe modification: separate workspace-level documentation from BCE milestone state.
- Test coverage: not applicable.

## Release and CI Risks

**Release workflow publishes with `--no-verify`:**
- Issue: tag-driven publish skips Cargo package verification.
- Files: `.github/workflows/release.yml`
- Impact: packaging mistakes can reach crates.io even if `cargo package` would fail locally.
- Fix approach: run `cargo package --allow-dirty --no-verify` only for inspection, or prefer verified publish after a package check step.

**Release workflow does not gate publication on release-specific validation:**
- Issue: `.github/workflows/release.yml` builds and publishes on tag push without rerunning tests, install checks, or smoke tests for produced tarballs.
- Files: `.github/workflows/release.yml`, `install.sh`
- Impact: release artifacts and installation flow can drift from what CI on `main` validated.
- Fix approach: add binary smoke tests, archive checks, and `install.sh` validation before publish.

**`gator` is released for Linux despite macOS-specific runtime behavior:**
- Issue: the release matrix builds Linux artifacts for every non-library crate, while `gator` hard-depends on `sandbox-exec`.
- Files: `.github/workflows/release.yml`, `crates/gator/Cargo.toml`, `crates/gator/src/agent.rs`
- Impact: Linux users can install a binary that compiles but cannot perform its primary function.
- Fix approach: restrict `gator` release targets or add a runtime compatibility notice and smoke test per target.

## Dependencies at Risk

**Nightly formatting remains part of normal development and CI:**
- Risk: formatting depends on nightly in `justfile` and `.github/workflows/ci.yml`.
- Files: `justfile`, `.github/workflows/ci.yml`
- Impact: toolchain churn can block formatting even when stable builds are healthy.
- Migration plan: pin the nightly toolchain explicitly or reduce formatting config to a stable-compatible subset.

## Missing Critical Features

**Agent-facing BCE reference is not implemented:**
- Problem: the active requirement `AGENT-01` is still pending and the binary ships placeholder output.
- Blocks: reliable agent self-discovery for `bce` usage, pagination semantics, and error handling.
- Files: `crates/bsky-comment-extractor/src/main.rs`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`

## Test Coverage Gaps

**No integration test directories for `gator` or `silent-critic`:**
- What's not tested: binary startup, external process wiring, session lifecycle across commands, and contract exchange between crates.
- Files: `crates/gator/src/*.rs`, `crates/silent-critic/src/*.rs`
- Risk: shelling, JSON contracts, and filesystem assumptions can regress without detection.
- Priority: High

**`bce` networked fetch path lacks end-to-end CLI coverage:**
- What's not tested: authenticated fetch flow, retry behavior against a live-like CLI path, and default-path side effects.
- Files: `crates/bsky-comment-extractor/src/client.rs`, `crates/bsky-comment-extractor/src/main.rs`
- Risk: the shipped network path can diverge from unit-tested client logic.
- Priority: Medium

**Release and install flow are untested in CI:**
- What's not tested: `install.sh`, produced release archives, and post-download executable behavior.
- Files: `install.sh`, `.github/workflows/release.yml`
- Risk: successful workspace tests do not prove end-user installability.
- Priority: Medium

---

*Concerns audit: 2026-03-22*
