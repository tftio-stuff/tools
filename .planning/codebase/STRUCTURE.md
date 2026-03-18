# Codebase Structure

## Top-Level Layout

```
tools/
├── Cargo.toml                  # workspace root, centralized [workspace.dependencies]
├── Cargo.lock                  # committed (binary crates)
├── CLAUDE.md                   # Claude Code project instructions
├── CRATES.md                   # expanded crate documentation
├── README.md                   # project introduction
├── deny.toml                   # cargo-deny dependency compliance
├── rustfmt.toml                # nightly formatter config
├── justfile                    # task runner (just dev, just ci, etc.)
├── prek.toml                   # pre-commit/pre-push configuration
├── release-please-config.json  # release-please manifest config
├── scripts/                    # build/release helper scripts
├── docs/                       # framework documentation (silent-critic)
├── .github/                    # CI/CD workflows
└── crates/                     # all workspace members
```

## Crate Directory

```
crates/
├── cli-common/     # shared library (tftio-cli-common)
├── prompter/       # TOML prompt composer (tftio-prompter)
├── unvenv/         # git venv detector (tftio-unvenv)
├── asana-cli/      # Asana API client (tftio-asana-cli)
├── todoer/         # SQLite todo manager (tftio-todoer)
├── silent-critic/  # supervision framework (tftio-silent-critic)
└── gator/          # agent sandbox harness (tftio-gator)
```

## Standard Crate Layout

Each binary crate follows a consistent pattern:

```
crates/<name>/
├── Cargo.toml          # package metadata, dep overrides
├── CHANGELOG.md        # release-please managed
├── src/
│   ├── main.rs         # CLI entrypoint (binary crates)
│   ├── lib.rs          # public API, module declarations
│   ├── cli.rs          # clap CLI definitions (or cli/ module dir)
│   ├── config.rs       # configuration loading
│   ├── error.rs        # error types (if needed)
│   ├── models.rs       # data models with serde
│   ├── output.rs       # output formatting
│   └── commands/       # subcommand implementations (if complex)
└── tests/              # integration tests (if present)
```

### Variations by Crate

| Crate | Notable Differences |
|-------|-------------------|
| `cli-common` | Library only, no `main.rs` or `cli.rs` |
| `unvenv` | Single-file `main.rs`, no lib.rs |
| `asana-cli` | `src/api/` module tree for API endpoints, `src/cli/` module tree for subcommands, `src/models/` for data types, `src/output/` for formatters, `src/filters/` for query filters, `docs/man/` for manpages |
| `prompter` | Large `lib.rs` (66K), `src/completions.rs` and `src/doctor.rs` |
| `todoer` | `src/commands/` subcommand modules, `src/repo.rs` for DB queries, `schema/` for JSON schema |
| `silent-critic` | `src/commands/` subcommand modules, `src/discovery.rs` for context gathering, `src/project.rs` for project init |
| `gator` | `src/agent.rs` for agent process management, `src/sandbox.rs` for SBPL policy generation, `src/worktree.rs` for git worktree detection, `src/session.rs` for silent-critic session integration |

## Key Locations

### Configuration Files

- `Cargo.toml` (root) - workspace definition, centralized dependencies
- `deny.toml` - license allowlist, ban rules, advisory DB config
- `rustfmt.toml` - formatting rules (requires nightly)
- `justfile` - build/test/lint recipes
- `release-please-config.json` - release automation config
- `prek.toml` - pre-commit hook configuration

### CI/CD

- `.github/workflows/ci.yml` - format, lint, test matrix, MSRV, audit, deny
- `.github/workflows/release-please.yml` - automated release PRs
- `.github/workflows/release.yml` - cross-platform binary builds + crates.io publish

### Documentation

- `docs/the-silent-critic.md` - framework overview
- `docs/the-silent-critic-system-spec.md` - system specification
- `docs/the-silent-critic-formal-appendix.md` - formal appendix
- `docs/the-silent-critic-tooling-design.md` - tooling design
- `docs/the-silent-critic-polemic-revised.md` - critique of review practices
- `docs/the-silent-critic-argument-memo.md` - argument memo
- `crates/asana-cli/docs/man/asana-cli.1` - manpage

## Naming Conventions

### Package Names
- Published packages: `tftio-<name>` (e.g., `tftio-prompter`)
- Binary names: `<name>` without prefix (e.g., `prompter`, `asana-cli`)
- Library names: snake_case matching crate (e.g., `asana_cli`, `silent_critic`)

### Module Names
- Snake case throughout: `custom_field.rs`, `cli_parse.rs`
- `mod.rs` used for module directories (e.g., `src/api/mod.rs`, `src/cli/mod.rs`)

### Test File Names
- Integration tests match module names: `tests/cli_parse.rs`, `tests/integration_test.rs`
- Unit tests inline in source files under `#[cfg(test)]` modules

### Git Tags
- Format: `{crate}-v{version}` (e.g., `prompter-v2.1.0`, `todoer-v1.1.0`)
- Each crate versioned independently via release-please
