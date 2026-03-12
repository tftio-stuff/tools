# CLAUDE.md

## Repository Overview

Cargo workspace monorepo containing tftio Rust CLI tools:

- **cli-common** (`tftio-cli-common`): Shared library for CLI functionality (completions, doctor, output, update)
- **prompter** (`tftio-prompter`): Compose reusable prompt snippets from markdown libraries using TOML profiles
- **unvenv** (`tftio-unvenv`): Detect Python virtual environments not ignored by Git
- **asana-cli** (`tftio-asana-cli`): Interface to the Asana API
- **todoer** (`tftio-todoer`): Global todo list manager for LLM agents

## Essential Commands

```bash
# Primary workflow
just dev            # format + lint + test
just ci             # full CI: quality + test + build-release + audit + deny

# Building
just build          # debug, all crates
just build-release  # release, all crates
just build-crate X  # single crate

# Testing
just test           # all crates
just test-crate X   # single crate

# Quality
just format         # requires nightly
just format-check
just lint           # clippy --workspace
just audit          # cargo audit
just deny           # cargo deny check

# Running
just run prompter -- list
just run todoer -- new "task"

# Scaffolding
just new-crate my-tool
```

## Architecture

### Workspace Layout

```
tools/
├── Cargo.toml          # workspace root with centralized deps
├── Cargo.lock          # committed (binary crates)
├── deny.toml           # dependency compliance
├── rustfmt.toml        # formatting config
├── justfile            # task runner
└── crates/
    ├── cli-common/     # library
    ├── prompter/       # binary + lib
    ├── unvenv/         # binary
    ├── asana-cli/      # binary + lib
    └── todoer/         # binary + lib
```

### Dependency Management

All dependencies are centralized in the root `Cargo.toml` under `[workspace.dependencies]`. Per-crate `Cargo.toml` files use `dep.workspace = true`.

### Lints

Workspace lints are shared via `[workspace.lints]` and inherited with `[lints] workspace = true`. Exception: todoer overrides `missing_docs = "allow"` (uses local lints instead of workspace).

### Versioning

Each crate is independently versioned via release-please. Tags follow the format `{component}-v{version}` (e.g., `prompter-v2.0.1`).

### Key Overrides

- **asana-cli**: Has additional reqwest features (multipart, stream) and tokio features (fs, signal, time)
- **todoer**: License is CC0-1.0 (not MIT), binary name is `todoer`

## Toolchain

- Edition: 2024
- MSRV: 1.94.0
- Nightly required for: `cargo fmt` only

## CI/CD

- **ci.yml**: Format, lint, test (matrix), MSRV, audit, deny
- **release-please.yml**: Creates release PRs on push to main
- **release.yml**: Builds cross-platform binaries and publishes to crates.io on tag push
