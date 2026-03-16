# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Workspace Overview

Cargo workspace monorepo (`tools/`) containing 5 Rust CLI tools:

| Crate | Package | Version | Description |
|-------|---------|---------|-------------|
| `cli-common` | `tftio-cli-common` | 0.5.0 | Shared library (not installable) |
| `prompter` | `tftio-prompter` | 2.1.0 | Compose prompt snippets from TOML profiles |
| `unvenv` | `tftio-unvenv` | 1.8.0 | Detect Python venvs ignored by Git |
| `asana-cli` | `tftio-asana-cli` | 1.2.0 | Asana API interface |
| `todoer` | `tftio-todoer` | 1.1.0 | Global todo manager for LLM agents |

See [`CRATES.md`](/Users/jfb/Projects/tools/feature/gator/CRATES.md) for expanded crate documentation.

## Quick Commands

```bash
# Build & test
just dev           # format + lint + test
just build         # debug, all crates
just build-release # release, all crates
just ci            # full pipeline: format-check + lint + test + build-release + audit + deny
just test          # all tests
just lint          # clippy workspace

# Single crate operations
just build-crate X
just test-crate X
just run X *args   # run a binary

# Quality gates
just audit         # cargo audit
just deny          # cargo deny check
```

## Architecture

### Workspace Structure

```
tools/
├── Cargo.toml          # workspace root with centralized deps ([workspace.dependencies])
├── Cargo.lock          # committed (binary crates)
├── deny.toml           # dependency compliance
├── rustfmt.toml        # formatting config
├── justfile            # task runner
└── crates/
    ├── cli-common/     # library crate (shared utilities)
    ├── prompter/       # binary + lib (TOML profile prompt composition)
    ├── unvenv/         # binary (git venv detection)
    ├── asana-cli/      # binary + lib (Asana API with async, multipart, tracing)
    └── todoer/         # binary + lib (SQLite-backed todo manager)
```

See [`CRATES.md`](/Users/jfb/Projects/tools/feature/gator/CRATES.md) for the full crate structure.

### Dependency Pattern

- All dependencies in root `Cargo.toml` under `[workspace.dependencies]`
- Per-crate uses `dep.workspace = true` for external deps
- `asana-cli` overrides `reqwest` with extra features: `multipart`, `stream`
- `asana-cli` overrides `tokio` with extra features: `fs`, `signal`, `time`
- `todoer` overrides `lints.rust.missing_docs = "allow"` (local lints)

### Lint Configuration

**Workspace-level lints** (`[workspace.lints]`):
- `rust.unsafe_code = "warn"`
- `rust.missing_docs = "deny"`
- `clippy.all = "deny"`
- `clippy.pedantic = "deny"`
- `clippy.wildcard_imports = "deny"`
- `clippy.enum_glob_use = "deny"`

**Per-crate overrides**: `todoer` disables `missing_docs` locally.

### Code Organization Patterns

Each binary crate follows this structure:
```
crates/NAMESPACE/
├── Cargo.toml
├── src/
│   ├── lib.rs          # public API, core logic
│   ├── main.rs         # CLI entrypoint
│   ├── cli.rs/mod.rs   # clap CLI definition
│   ├── config.rs       # configuration management
│   ├── error.rs        # error types
│   ├── output/         # output formatting helpers
│   ├── models/         # data models (serde serialization)
│   └── [crate-specific modules]
└── tests/              # integration tests (if any)
```

### Testing Pattern

- Unit tests live in `src/` files after `#[cfg(test)]` modules
- Integration tests (if present) in `tests/` directory
- `asana-cli` uses `mockito` for API mocking, `serial_test` for test isolation
- Run single crate tests: `cargo test -p <crate-name>`

### Versioning

- Each crate independently versioned via release-please
- Release PRs created on push to `main`
- Tags format: `{crate}-v{version}` (e.g., `prompter-v2.0.1`, `todoer-v1.1.0`)
- Use `versioneer` for version number changes

### Toolchain

- MSRV: 1.94.0 (declared in `rust-version` workspace field)
- Nightly: required only for `cargo fmt`
- Use `uv run python` for Python-related tasks (unvenv)

### CI/CD

- `ci.yml`: Format, lint, test (matrix), MSRV, audit, deny
- `release-please.yml`: Creates release PRs on push to main
- `release.yml`: Builds cross-platform binaries and publishes to crates.io on tag push

## Crate-Specific Notes

### `tftio-cli-common`

- Library-only, not installable
- Provides: completions, doctor, license, output, types, update helpers
- Dependencies: `clap`, `clap_complete`, `colored`, `is-terminal`

### `tftio-prompter`

- Composes prompt snippets from TOML profiles
- Supports recursive profile dependencies
- Deduplicates markdown files
- Dependencies include `chrono`, `indicatif` (progress bars), `serde`, `toml`

### `tftio-unvenv`

- Scans repo for Python venvs not in `.gitignore`
- Uses `git2` (vendored libgit2) + `walkdir`
- Outputs colored warnings for problematic venvs

### `tftio-asana-cli`

- Asana API wrapper with async operations
- Features: multipart uploads, streaming responses
- Uses `tracing` for observability, `secrecy` for token handling
- Models: tasks, projects, workspaces, users, tags, stories, sections, attachments, custom_fields
- API client abstraction with pagination support

### `tftio-todoer`

- SQLite-backed global todo manager for LLM agents
- Reads `.todoer.toml` config
- CLI commands: `new`, `list`, `init`
- Uses `rusqlite` (bundled), `uuid`, `chrono`
- License: CC0-1.0 (exception in workspace)

## The Silent Critic Framework

The Silent Critic is a supervision framework for software development in a world where software is becoming effectively free to produce and human attention is the scarce resource.

See [`docs/the-silent-critic.md`](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic.md) for the framework overview.

### Core Concepts

- **Acceptance surface**: The full criteria for task acceptance, including explicit and tacit criteria
- **Visible criteria**: Criteria shown to the worker during execution
- **Hidden criteria**: Criteria hidden from the worker during execution
- **Tool-authored evidence**: Evidence recorded by the system, not worker narration
- **Residual uncertainty**: Uncertainty that requires human judgment
- **Decision log**: The canonical record of task adjudication

### Key Principles

1. The worker cannot be shown the full acceptance surface during execution
2. Human review should focus on residual uncertainty, not raw diffs
3. Tool-authored evidence is preferred over worker narration
4. Hidden criteria are disclosed after adjudication for transparency
5. The decision record must be shareable and social

### System Specification

For the full system specification and formal appendix, see:

- [`docs/the-silent-critic-system-spec.md`](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic-system-spec.md)
- [`docs/the-silent-critic-formal-appendix.md`](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_formal_appendix.md)
- [`docs/the-silent-critic-tooling-design.md`](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_tooling_design.md)

### Polemic and Argument Documents

For the detailed critique of existing review practices:

- [`docs/the-silent-critic-polemic-revised.md`](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_polemic_revised.md)
- [`docs/the-silent-critic-argument-memo.md`](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic-argument-memo.md)

## Security & Compliance

- `deny.toml` enforces:
  - Allowed licenses: MIT, Apache-2.0, CC0-1.0, etc.
  - `confidence-threshold = 0.9`
  - Banned: `wildcards = "deny"`, unknown sources = deny
  - Windows `windows-sys` family: skipped
- CI includes: `cargo audit`, `cargo deny check`, `cargo deny test`

## Git Workflow

- Default branch: `main`
- Feature branches created from `main`
- PRs created against `main`
- Use `git worktrees` for isolation (see `.cursor/rules/git-worktrees.md` for patterns)

## Related Documentation

- [README.md](/Users/jfb/Projects/tools/feature/gator/README.md) - Project introduction
- [CRATES.md](/Users/jfb/Projects/tools/feature/gator/CRATES.md) - Crate documentation reference
