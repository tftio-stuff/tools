# Codebase Structure

**Analysis Date:** 2026-03-17

## Directory Layout

```
tools/main/
├── Cargo.toml                  # Workspace root; all shared deps under [workspace.dependencies]
├── Cargo.lock                  # Committed (binary workspace)
├── justfile                    # Task runner (dev, build, ci, lint, audit, deny)
├── deny.toml                   # cargo-deny license + dependency compliance
├── rustfmt.toml                # Formatting config (requires nightly fmt)
├── CLAUDE.md                   # Claude Code workspace instructions
├── CRATES.md                   # Expanded crate reference documentation
├── README.md                   # Project introduction
├── .github/
│   └── workflows/
│       ├── ci.yml              # Format, lint, test matrix, MSRV, audit, deny
│       ├── release-please.yml  # Creates release PRs on push to main
│       └── release.yml         # Tag-triggered binary builds + crates.io publish
├── docs/
│   ├── plans/                  # Phase/implementation planning documents
│   └── *.md                    # Design documents (silent-critic framework)
├── scripts/
│   └── new-crate.sh            # Scaffold a new crate in the workspace
├── .planning/
│   └── codebase/               # GSD codebase mapping documents
└── crates/
    ├── cli-common/             # Shared library (not a binary)
    ├── prompter/               # Binary + lib: TOML profile prompt composition
    ├── unvenv/                 # Binary only: Python venv git detection
    ├── asana-cli/              # Binary + lib: Asana API client
    ├── todoer/                 # Binary + lib: SQLite-backed todo manager
    ├── silent-critic/          # Binary + lib: Agentic supervision framework
    └── gator/                  # Binary + lib: Agent sandbox harness
```

## Directory Purposes

Each binary crate follows a consistent internal structure:

```
crates/<name>/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Public API, module declarations, crate-level docs
│   ├── main.rs         # CLI entrypoint (parses args, dispatches, exits)
│   ├── cli.rs          # clap CLI definitions (commands, args, flags)
│   ├── config.rs       # Config struct and loading/resolution logic
│   ├── models.rs       # Serde data models (or models/ subdir for many types)
│   ├── output.rs       # Output formatting (or output/ subdir)
│   └── commands/       # Subcommand handlers (one file per subcommand)
│       ├── mod.rs
│       └── <verb>.rs
└── tests/              # Integration tests (if present)
```

The library-only crate `crates/cli-common` has no `main.rs` or `commands/` directory.
`crates/unvenv` is a single-file binary (`src/main.rs`) with no separate lib or commands directory.

## Per-Crate Source Structure

**`crates/cli-common/src/`** - Shared utilities consumed by all binary crates:
- `lib.rs`, `completions.rs`, `doctor.rs`, `license.rs`, `output.rs`, `types.rs`, `update.rs`

**`crates/prompter/src/`** - TOML profile-driven prompt composition:
- `lib.rs`, `main.rs`, `completions.rs`, `doctor.rs`
- Tests: `crates/prompter/tests/`

**`crates/unvenv/src/`** - Single-module binary, no lib:
- `main.rs` only (all logic self-contained)
- Tests: `crates/unvenv/tests/`

**`crates/asana-cli/src/`** - Largest crate; uses subdirectories for all major modules:
- `lib.rs`, `main.rs`, `config.rs`, `error.rs`, `doctor.rs`
- `api/`: `mod.rs`, `client.rs`, `auth.rs`, `pagination.rs`, `error.rs`, plus one file per resource: `tasks.rs`, `projects.rs`, `sections.rs`, `stories.rs`, `tags.rs`, `users.rs`, `workspaces.rs`, `attachments.rs`, `custom_fields.rs`
- `cli/`: `mod.rs`, `task.rs`, `project.rs`, `section.rs`, `tag.rs`, `user.rs`, `workspace.rs`, `custom_field.rs`
- `models/`: `mod.rs`, `task.rs`, `project.rs`, `section.rs`, `story.rs`, `tag.rs`, `user.rs`, `workspace.rs`, `attachment.rs`, `custom_field.rs`
- `output/`: `mod.rs`, `task.rs`, `project.rs`
- `filters/mod.rs`, `templates/mod.rs`, `templates/templates/`
- Man page: `crates/asana-cli/docs/man/asana-cli.1`
- Tests: `crates/asana-cli/tests/`

**`crates/todoer/src/`** - SQLite-backed todo manager:
- `lib.rs`, `main.rs`, `cli.rs`, `config.rs`, `db.rs`, `models.rs`, `output.rs`, `project.rs`, `repo.rs`, `input.rs`
- `commands/`: `mod.rs`, `init.rs`, `list.rs`, `new.rs`, `task.rs`
- Schema: `crates/todoer/schema/todoer-output.schema.json`
- Tests: `crates/todoer/tests/`

**`crates/silent-critic/src/`** - Supervision framework:
- `lib.rs`, `main.rs`, `cli.rs`, `config.rs`, `db.rs`, `discovery.rs`, `models.rs`, `output.rs`, `project.rs`
- `commands/`: `mod.rs`, `contract.rs`, `criterion.rs`, `decide.rs`, `log.rs`, `project.rs`, `session.rs`

**`crates/gator/src/`** - Agent sandbox harness:
- `lib.rs`, `main.rs`, `cli.rs`, `config.rs`, `agent.rs`, `sandbox.rs`, `session.rs`, `prompt.rs`, `worktree.rs`
- No separate tests directory (unit tests co-located in source files)

## Key File Locations

**Workspace configuration:**
- `Cargo.toml`: All shared dependency versions under `[workspace.dependencies]`
- `deny.toml`: Allowed licenses and banned crates
- `rustfmt.toml`: Formatting rules (nightly)
- `justfile`: All dev workflow commands (`dev`, `build`, `build-release`, `ci`, `test`, `lint`, `audit`, `deny`)

**Entry points (per crate):**
- `crates/<name>/src/main.rs`: CLI binary entrypoint
- `crates/<name>/src/lib.rs`: Library public API and module declarations

**CLI definitions:**
- `crates/<name>/src/cli.rs`: clap struct/enum definitions (simple crates)
- `crates/asana-cli/src/cli/mod.rs`: clap subcommand tree for asana-cli (complex crate uses subdir)

**CI/CD:**
- `.github/workflows/ci.yml`: Full quality gate pipeline
- `.github/workflows/release-please.yml`: Release PR automation
- `.github/workflows/release.yml`: Binary builds and publish on tag

## Naming Conventions

**Crate directories:** `kebab-case` (e.g., `asana-cli`, `cli-common`, `silent-critic`)

**Package names:** Prefixed with `tftio-` (e.g., `tftio-asana-cli`, `tftio-cli-common`)

**Binary names:** No prefix (e.g., `prompter`, `asana-cli`, `silent-critic`)

**Library names:** `snake_case` matching crate directory (e.g., `asana_cli`, `silent_critic`)

**Source files:** `snake_case.rs` throughout

**Module directories:** Use `mod.rs` as the module entry (e.g., `api/mod.rs`, `commands/mod.rs`)

**Resource modules:** Named after the domain object, singular (e.g., `task.rs`, `project.rs`, `criterion.rs`)

**Test files:** Descriptive names matching what is tested (e.g., `commands_init.rs`, `db_schema.rs`)

**Git tags:** `{crate}-v{version}` (e.g., `prompter-v2.1.0`, `todoer-v1.1.0`)

## Where to Add New Code

**New crate:**
- Run `scripts/new-crate.sh <name>` to scaffold
- Package name must be `tftio-<name>`
- Add to `members` array in root `Cargo.toml`
- Add to `release-please-config.json`

**New subcommand in an existing crate:**
- Add handler: `crates/<crate>/src/commands/<verb>.rs`
- Register in `crates/<crate>/src/commands/mod.rs`
- Add clap variant to `crates/<crate>/src/cli.rs`
- Add dispatch match arm in `crates/<crate>/src/main.rs`

**New API resource (asana-cli):**
- Model: `crates/asana-cli/src/models/<resource>.rs`
- API methods: `crates/asana-cli/src/api/<resource>.rs`
- CLI subcommand: `crates/asana-cli/src/cli/<resource>.rs`
- Output formatter: `crates/asana-cli/src/output/<resource>.rs` (if needed)

**New shared utility:**
- Add to `crates/cli-common/src/` with a descriptive module name
- Export from `crates/cli-common/src/lib.rs`
- Add `tftio-cli-common.workspace = true` to consuming crate's `Cargo.toml`

**New dependency:**
- Add version to `[workspace.dependencies]` in root `Cargo.toml`
- Reference with `<dep>.workspace = true` in the crate's `Cargo.toml`
- Per-crate feature overrides are allowed (see `asana-cli` overriding `reqwest` features in `crates/asana-cli/Cargo.toml`)

**New integration test:**
- Place in `crates/<name>/tests/<what_is_tested>.rs`
- For `todoer`, use the descriptive `commands_<verb>.rs` naming pattern

## Special Directories

**`.planning/codebase/`:**
- GSD codebase mapping documents (STACK, ARCHITECTURE, STRUCTURE, CONVENTIONS, TESTING, CONCERNS, INTEGRATIONS)
- Committed to repo

**`docs/plans/`:**
- Implementation phase plans from GSD planning commands
- Committed to repo

**`crates/todoer/schema/`:**
- `todoer-output.schema.json`: JSON Schema for todoer CLI output format
- Committed to repo; generated: No

**`crates/asana-cli/docs/man/`:**
- `asana-cli.1`: Man page for asana-cli (static, hand-authored)
- Committed to repo

**`target/`:**
- Cargo build output
- Not committed; excluded from git

---

*Structure analysis: 2026-03-17*
