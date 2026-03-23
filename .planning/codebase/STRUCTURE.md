# Codebase Structure

**Analysis Date:** 2026-03-23

## Directory Layout

```text
tools/
├── Cargo.toml                         # Workspace members, shared dependencies, lint policy
├── justfile                           # Common build/test/lint entrypoints
├── .github/workflows/                 # CI, release, and release-please automation
├── crates/                            # One directory per Rust crate
│   ├── cli-common/                    # Shared library crate
│   ├── prompter/                      # Prompt composition CLI + library
│   ├── unvenv/                        # Python venv scanner CLI
│   ├── asana-cli/                     # Asana API CLI with layered modules
│   ├── todoer/                        # SQLite-backed todo manager
│   ├── silent-critic/                 # Supervision/session framework
│   ├── gator/                         # Agent sandbox harness
│   └── bsky-comment-extractor/        # BlueSky extractor/query CLI
├── docs/                              # Product docs and design notes
├── scripts/                           # Repository maintenance scripts
└── .planning/                         # GSD planning, roadmap, and codebase maps
```

## Directory Purposes

**`crates/`:**
- Purpose: Hold all workspace members declared in `Cargo.toml`.
- Contains: Per-crate `Cargo.toml`, `src/`, optional `tests/`, optional crate-local `docs/` or `schema/`.
- Key files: `crates/*/Cargo.toml`, `crates/*/src/main.rs`, `crates/*/src/lib.rs`.

**`docs/`:**
- Purpose: Long-form design and concept documents.
- Contains: Narrative docs such as `docs/the-silent-critic-system-spec.md` and dated plan notes under `docs/plans/`.
- Key files: `docs/outline.md`, `docs/plans/2026-03-16-gator-agent-harness.md`.

**`scripts/`:**
- Purpose: Repository helper scripts that do not belong to a crate binary.
- Contains: Shell scripts such as `scripts/new-crate.sh`.
- Key files: `scripts/new-crate.sh`.

**`.github/workflows/`:**
- Purpose: CI and release automation.
- Contains: `ci.yml`, `release.yml`, and `release-please.yml`.
- Key files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`.

**`.planning/`:**
- Purpose: Planning artifacts for ongoing product work.
- Contains: State, milestone documents, phase folders, and generated codebase maps.
- Key files: `.planning/STATE.md`, `.planning/milestones/v1.1-ROADMAP.md`, `.planning/codebase/`.

## Key File Locations

**Entry Points:**
- `justfile`: developer-facing root command surface.
- `crates/asana-cli/src/main.rs`: Asana binary entry point.
- `crates/bsky-comment-extractor/src/main.rs`: `bce` binary entry point.
- `crates/gator/src/main.rs`: `gator` binary entry point.
- `crates/prompter/src/main.rs`: `prompter` binary entry point.
- `crates/silent-critic/src/main.rs`: `silent-critic` binary entry point.
- `crates/todoer/src/main.rs`: `todoer` binary entry point.
- `crates/unvenv/src/main.rs`: `unvenv` binary entry point.

**Configuration:**
- `Cargo.toml`: workspace configuration and dependency source of truth.
- `rustfmt.toml`: rustfmt settings.
- `deny.toml`: cargo-deny policy.
- `release-please-config.json` and `.release-please-manifest.json`: per-crate release management.
- `crates/asana-cli/src/config.rs`, `crates/todoer/src/config.rs`, `crates/silent-critic/src/config.rs`, `crates/gator/src/config.rs`: tool-local runtime config logic.

**Core Logic:**
- `crates/cli-common/src/`: shared CLI infrastructure.
- `crates/asana-cli/src/api/`: Asana client and endpoint modules.
- `crates/asana-cli/src/models/`: Asana domain DTOs.
- `crates/todoer/src/commands/` and `crates/todoer/src/repo.rs`: command handlers and SQLite access.
- `crates/silent-critic/src/commands/`, `crates/silent-critic/src/db.rs`, `crates/silent-critic/src/models.rs`: session framework internals.
- `crates/gator/src/agent.rs`, `crates/gator/src/sandbox.rs`, `crates/gator/src/session.rs`: harness composition.
- `crates/bsky-comment-extractor/src/client.rs` and `crates/bsky-comment-extractor/src/db.rs`: BlueSky fetch/query pipeline.

**Testing:**
- `crates/asana-cli/tests/`: integration tests around CLI and API behavior.
- `crates/bsky-comment-extractor/tests/`: CLI contract tests for query mode.
- `crates/prompter/tests/`: integration tests for CLI behavior.
- `crates/todoer/tests/`: extensive integration coverage by feature area.
- `crates/unvenv/tests/`: integration tests for repository scanning.
- Inline unit tests appear at the bottom of many source files such as `crates/gator/src/cli.rs` and `crates/bsky-comment-extractor/src/db.rs`.

## Naming Conventions

**Files:**
- Crate roots follow Cargo defaults: `Cargo.toml`, `src/main.rs`, `src/lib.rs`.
- Rust module files are snake_case, for example `crates/gator/src/worktree.rs` and `crates/asana-cli/src/api/custom_fields.rs`.
- Command-specific files in nested modules also stay snake_case, for example `crates/todoer/tests/project_resolution.rs`.

**Directories:**
- Crate directories use kebab-case, for example `crates/asana-cli/` and `crates/bsky-comment-extractor/`.
- Nested Rust module directories are lowercase or snake_case, for example `crates/asana-cli/src/output/` and `crates/silent-critic/src/commands/`.
- Planning directories use milestone and phase numbering, for example `.planning/milestones/v1.1-phases/04-cli-surface/`.

## File Organization Examples

**Binary + library crate pattern:**
```text
crates/gator/
├── Cargo.toml
└── src/
    ├── main.rs        # CLI/process boundary
    ├── lib.rs         # orchestration entrypoint
    ├── cli.rs         # clap definitions
    ├── config.rs      # config resolution
    ├── sandbox.rs     # policy generation
    └── agent.rs       # child-process execution
```

**Layered domain crate pattern:**
```text
crates/asana-cli/
├── src/api/           # HTTP client + endpoint modules
├── src/cli/           # clap command families
├── src/models/        # serde DTOs
├── src/output/        # renderers/formatters
└── tests/             # integration tests
```

**SQLite-backed command crate pattern:**
```text
crates/todoer/
├── src/cli.rs         # command definitions
├── src/commands/      # per-command handlers
├── src/db.rs          # schema/bootstrap
├── src/repo.rs        # SQL access helpers
├── src/models.rs      # task/project types
└── tests/             # command- and persistence-level integration tests
```

## Where to Add New Code

**New Workspace Tool:**
- Primary code: `crates/<new-crate>/`
- Manifest: add `crates/<new-crate>` to `Cargo.toml` `[workspace].members`
- Release metadata: add `crates/<new-crate>` to `release-please-config.json` and `.release-please-manifest.json`
- Script reference: `scripts/new-crate.sh` shows the expected scaffold shape

**New CLI Subcommand in an Existing Tool:**
- `asana-cli`: add clap definitions under `crates/asana-cli/src/cli/`, endpoint logic in `crates/asana-cli/src/api/`, DTOs in `crates/asana-cli/src/models/`, and renderers in `crates/asana-cli/src/output/`.
- `todoer`: add command enum variants in `crates/todoer/src/cli.rs`, handler modules in `crates/todoer/src/commands/`, and persistence helpers in `crates/todoer/src/repo.rs` or `src/db.rs`.
- `silent-critic`: add the CLI variant in `crates/silent-critic/src/cli.rs` and implementation in `crates/silent-critic/src/commands/`.
- `bce`: add subcommand args in `crates/bsky-comment-extractor/src/cli.rs` and route in `crates/bsky-comment-extractor/src/main.rs`.

**Shared Utilities:**
- Shared helpers across multiple crates: `crates/cli-common/src/`
- Tool-local helpers: keep them under that crate’s `src/` tree instead of introducing new root directories.

**Tests for New Behavior:**
- Integration tests: `crates/<crate>/tests/`
- Small unit tests for a single module: append `#[cfg(test)]` modules to the corresponding `src/*.rs` file.

**Docs for New Behavior:**
- Tool-specific command docs: crate-local directories such as `crates/asana-cli/docs/`
- Cross-cutting design docs or planning notes: `docs/` or `.planning/milestones/` depending on whether the artifact is permanent product documentation or workflow planning.

## Special Directories

**`crates/asana-cli/docs/`:**
- Purpose: crate-local manual page source.
- Generated: No.
- Committed: Yes.

**`crates/todoer/schema/`:**
- Purpose: JSON schema for todoer output contracts.
- Generated: No.
- Committed: Yes.

**`docs/plans/`:**
- Purpose: dated design and implementation plans.
- Generated: No.
- Committed: Yes.

**`.planning/milestones/`:**
- Purpose: phase-by-phase GSD artifacts such as `*-PLAN.md`, `*-SUMMARY.md`, `*-VALIDATION.md`, and `*-VERIFICATION.md`.
- Generated: Yes, by workflow tools.
- Committed: Yes.

**`.planning/codebase/`:**
- Purpose: generated repository maps such as `.planning/codebase/ARCHITECTURE.md` and `.planning/codebase/STRUCTURE.md`.
- Generated: Yes.
- Committed: Yes.

---

*Structure analysis: 2026-03-23*
