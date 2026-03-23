# External Integrations

**Analysis Date:** 2026-03-23

## APIs & External Services

**Asana REST API:**
- Service: Asana project management platform
  - SDK/Client: `reqwest` 0.13 HTTP client, implementation in `crates/asana-cli/src/api/`
  - Base URL: `https://app.asana.com/api/1.0` (default); overridable via `ASANA_BASE_URL` env var
  - Auth: Personal Access Token (PAT) via `ASANA_PAT` env var or stored in config file
  - Token storage: `~/.config/asana-cli/asana-cli.toml`, Unix permissions enforced to 0o600; wrapped in `secrecy::SecretString` to prevent accidental log exposure
  - HTTP features: Multipart uploads (attachments), streaming responses
  - Pagination: Cursor-based pagination via async streams (`async-stream`, `futures-core`)
  - Resources: Tasks, projects, workspaces, users, tags, stories, sections, attachments, custom fields
  - Logging: All API calls instrumented via `tracing`; level via `RUST_LOG`

**crates.io:**
- Publishing: All installable crates are published to crates.io on release
  - Auth: `CARGO_REGISTRY_TOKEN` GitHub Actions secret
  - Workflow: `release.yml` triggers `cargo publish -p <crate>` on version tag push
  - Registry: `https://github.com/rust-lang/crates.io-index` (sole allowed registry, enforced by `deny.toml`)

**GitHub:**
- Repository hosting: `https://github.com/tftio-stuff/tools`
- Release automation: `googleapis/release-please-action@v4` creates release PRs
  - Auth: `RELEASE_PLEASE_TOKEN` GitHub Actions secret
  - Workflow: `.github/workflows/release-please.yml`
- CI: `.github/workflows/ci.yml` (format, lint, test matrix, MSRV, audit, deny)
- Binary release: `taiki-e/upload-rust-binary-action@v1` uploads cross-compiled binaries on tag push
  - Auth: `GITHUB_TOKEN` (automatic)
- Security audit: `rustsec/audit-check@v2` in CI
- Dependency compliance: `EmbarkStudios/cargo-deny-action@v2` in CI

## Data Storage

**Databases:**

- **SQLite 3 (bundled via `rusqlite` 0.38):**
  - `todoer` database: `~/.local/share/todoer/todoer.db`
    - Implementation: `crates/todoer/src/db.rs`
    - Schema: `projects`, `tasks`, `task_notes` tables
    - Config override: `db_path` in `.todoer.toml`
  - `silent-critic` database: `~/.local/share/silent-critic/{project-sha256-hash}/db.sqlite`
    - Implementation: `crates/silent-critic/src/db.rs`
    - Schema: `projects`, `criteria`, `sessions`, `contracts`, `contract_criteria`, `discovery_contexts`, `evidence`, `decisions`, `audit_events`
    - WAL mode enabled: `PRAGMA journal_mode = WAL`
    - Foreign keys enforced: `PRAGMA foreign_keys = ON`
    - Config override: `db_dir` in `~/.config/silent-critic/config.toml`

**File Storage:**
- Local filesystem only; no cloud object storage (no S3, GCS, Azure Blob)
- `asana-cli` cache: `~/.local/share/asana-cli/cache/` (HTTP response caching via `Config.cache_dir`)
- `gator` sandbox profiles: `~/.config/sandbox-exec/agent.sb` (static base SBPL policy, user-managed)
- `prompter` profiles: User-configured directory of TOML profile files and markdown snippet files
- Temporary files: `tempfile` crate for secure temp directories during sandbox policy assembly

**Caching:**
- `asana-cli` maintains a local cache directory for API responses at `~/.local/share/asana-cli/cache/`
- No Redis, Memcached, or distributed caching

## Authentication & Identity

**Auth Provider:**
- No third-party identity provider (no OAuth, OIDC, SAML, JWT)
- Custom token-based authentication per service:
  - Asana PAT: stored in config file at `~/.config/asana-cli/asana-cli.toml` with 0o600 Unix permissions; `secrecy::SecretString` prevents log leakage; `rpassword` used for interactive input
  - Silent Critic worker auth: opaque session token via `SILENT_CRITIC_TOKEN` env var; transient, not persisted

## Monitoring & Observability

**Error Tracking:**
- None - No Sentry, Bugsnag, Rollbar, or external error reporting service

**Logs:**
- Structured tracing via `tracing` 0.1 framework
- Sink: `tracing-subscriber` to stderr with `EnvFilter`; level controlled by `RUST_LOG`
- Asana API calls instrumented with spans; no metrics export (no Prometheus, Datadog, etc.)
- Tracing initialization: `crates/asana-cli/src/lib.rs` `init_tracing()` function

## CI/CD & Deployment

**CI Platform:**
- GitHub Actions
  - `.github/workflows/ci.yml`: format check (nightly), lint (clippy), test matrix (ubuntu + macos), MSRV check (1.94.0), security audit, dependency compliance
  - Caching: `Swatinem/rust-cache@v2` on lint, test, MSRV jobs
  - `CARGO_TERM_COLOR=always` set workspace-wide

**Release Automation:**
- `release-please` (`googleapis/release-please-action@v4`) creates release PRs on push to `main`
  - Config: `release-please-config.json`
  - Auth: `RELEASE_PLEASE_TOKEN` secret
- On tag push (`*-v*` pattern): `release.yml` builds cross-platform binaries and publishes to crates.io
  - Build targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`
  - Archives: `.tar.gz` with SHA-256 checksums
  - Publishing: `cargo publish` with `CARGO_REGISTRY_TOKEN` secret

**Hosting:**
- GitHub Releases for binary distribution
- crates.io for library/crate distribution
- No containerization (no Docker, no Kubernetes, no cloud platform)

## Environment Configuration

**Required env vars (per crate):**
- `asana-cli`: `ASANA_PAT` - Asana Personal Access Token (required for all API operations)
- `gator` + `silent-critic` session execution: `SILENT_CRITIC_TOKEN` - Worker auth token

**Optional env vars:**
- `ASANA_BASE_URL` - Custom Asana API base URL (default: `https://app.asana.com/api/1.0`)
- `ASANA_WORKSPACE` - Default workspace GID
- `ASANA_ASSIGNEE` - Default assignee (email or GID)
- `ASANA_PROJECT` - Default project GID
- `ASANA_CLI_CONFIG_HOME` - Override asana-cli config directory
- `ASANA_CLI_DATA_HOME` - Override asana-cli data directory
- `RUST_LOG` - Tracing level (e.g., `debug`, `info`, `warn`)
- `XDG_CONFIG_HOME`, `XDG_DATA_HOME` - Standard XDG overrides used by `directories` crate

**Secrets location:**
- Asana PAT: disk file `~/.config/asana-cli/asana-cli.toml` (permissions 0o600 on Unix; code: `crates/asana-cli/src/config.rs`)
- `CARGO_REGISTRY_TOKEN`: GitHub Actions secret (publish only)
- `RELEASE_PLEASE_TOKEN`: GitHub Actions secret (release PR creation)
- No `.env` files used; all configuration via environment variables or TOML config files

## Webhooks & Callbacks

**Incoming:**
- None - No webhook endpoints implemented

**Outgoing:**
- None - No external webhook calls

## Git Integration

**Library:** `git2` 0.20 (vendored-libgit2; no system libgit2 required)

**Used by:**
- `crates/unvenv/src/main.rs` - Repository status to detect unignored venvs
- `crates/gator/src/worktree.rs` - Worktree discovery for sandbox policy generation
- `crates/silent-critic/src/project.rs` - Repository discovery and project identity hashing

**Operations:**
- Repository discovery (walk up directory tree)
- Worktree enumeration (linked worktrees for SBPL policy sibling grants)
- SHA-256 hash of repository path for project identity (`sha2` crate)

## macOS Sandbox Integration (gator)

**Platform:** macOS only

**Framework:** `sandbox-exec` (macOS built-in) + SBPL (Sandbox Policy Language)

**Policy assembly:** `crates/gator/src/sandbox.rs`
- Reads static base profile from `~/.config/sandbox-exec/agent.sb` (user must provide this file)
- Appends dynamic rules: workdir RW grant, git common dir RW grant, sibling worktrees RO grant, extra RW/RO dirs, deny rules
- Policy passed to `sandbox-exec` command at agent launch

**No Linux equivalent:** No seccomp, cgroup, or AppArmor support

## Internal Crate Integration (gator + prompter)

**Dependency:** `gator` depends on `tftio-prompter` as a library crate

**Purpose:** `crates/gator/src/prompt.rs` uses `prompter` to compose system prompts from TOML profile files before agent launch

**Protocol:** `gator` communicates with `silent-critic` via subprocess (`silent-critic session sandbox <id> --format json`); parses JSON output in `crates/gator/src/session.rs`

---

*Integration audit: 2026-03-23*
