# External Integrations

**Analysis Date:** 2026-03-23

## APIs & External Services

**Asana API:**
- Service: Asana project management platform
  - Endpoint: `https://app.asana.com/api/1.0` (default, overridable via `ASANA_BASE_URL`)
  - SDK/Client: `reqwest` 0.13 HTTP client via `crates/asana-cli/src/api/` module
  - Auth: Personal Access Token (PAT) via `ASANA_PAT` environment variable
  - Auth storage: Secure disk storage with `secrecy` redaction in `crates/asana-cli/src/config.rs`
  - Features: Multipart upload support, streaming responses
  - Models: Tasks, projects, workspaces, users, tags, stories, sections, attachments, custom fields
  - Pagination: Automatic cursor-based pagination via async streams

**Silent Critic Worker Communication:**
- Service: Internal agent-supervisor protocol
  - Auth: Opaque session token via `SILENT_CRITIC_TOKEN` environment variable
  - Protocol: JSON-based contract exchange via stdout/stdin (see `crates/silent-critic/src/`)
  - Usage: `gator` shells out to `silent-critic session sandbox <id> --format json`
  - Integration: `crates/gator/src/session.rs:fetch_session_sandbox()` parses contract sandbox specifications

## Data Storage

**Databases:**

- **SQLite 3 (bundled):**
  - Todoer backend: `~/.local/share/todoer/todoer.db` (XDG-compliant, overridable)
    - Location: `crates/todoer/src/db.rs`
    - Tables: projects, tasks, task_notes
    - Config file: `.todoer.toml` (reads `db_path` override)
  - Silent Critic backend: `~/.local/share/silent-critic/{project-hash}/db.sqlite` (XDG-compliant)
    - Location: `crates/silent-critic/src/db.rs`
    - Tables: projects, criteria, sessions, contracts, contract_criteria, discovery_contexts, evidence, decisions, audit_events
    - WAL mode enabled: `PRAGMA journal_mode = WAL`
    - Foreign keys enforced: `PRAGMA foreign_keys = ON`
    - Config file: `~/.config/silent-critic/config.toml` (reads `db_dir` override)

**File Storage:**
- Local filesystem only - No cloud storage integrations detected
- Cache directory: `~/.local/share/{app}/cache/` (XDG-compliant, used by `asana-cli`)
- Config paths: `~/.config/{app}/` (XDG-compliant)
- Temporary: `tempfile` crate for secure temp files (`gator` sandbox policies)

**Caching:**
- HTTP response caching in `asana-cli` (cache directory managed via `Config.cache_dir`)
- No Redis, Memcached, or distributed caching

## Authentication & Identity

**Auth Provider:**
- Custom token-based auth
  - Asana: Personal Access Token (PAT) - stored encrypted on disk, retrieved via `ASANA_PAT` env var
  - Silent Critic: Opaque session tokens - `SILENT_CRITIC_TOKEN` for worker processes
  - Implementation: No OAuth, JWT, or third-party identity provider
  - Secure input: `rpassword` 7 for interactive password prompts (`asana-cli`)

**Token Storage:**
- Asana PAT: `~/.config/asana-cli/asana-cli.toml` (unencrypted, permissions: 0o600 on Unix)
  - Code: `crates/asana-cli/src/config.rs` lines 200-220
  - Redaction: `secrecy::SecretString` prevents accidental logging

## Monitoring & Observability

**Error Tracking:**
- None detected - No Sentry, Rollbar, or error tracking service

**Logs:**
- Structured logging via `tracing` framework
- Sink: `tracing-subscriber` to stderr with `EnvFilter`
- Default level: `info` (overridable via `RUST_LOG` environment variable)
- Code: `crates/asana-cli/src/lib.rs:init_tracing()`

**Spans & Metrics:**
- `asana-cli` uses `tracing` instrumentation on async API operations
- No dedicated metrics/monitoring service (Prometheus, Datadog, etc.)

## CI/CD & Deployment

**Hosting:**
- GitHub (repository: `https://github.com/tftio-stuff/tools`)
- No containerization detected (no Docker/OCI)
- No cloud platform deployments (AWS, GCP, Azure)

**CI Pipeline:**
- GitHub Actions (inferred from release-please and CI workflows)
  - Workflows: `ci.yml` (format, lint, test, MSRV, audit, deny), `release-please.yml`, `release.yml`
  - Matrix testing: Multiple Rust versions
  - Cross-platform builds: Tests run on Linux and macOS

**Package Distribution:**
- crates.io publication (inferred from release workflow)
- Binaries: Likely published via GitHub Releases

## Environment Configuration

**Required env vars:**
- `ASANA_PAT` - Asana API authentication (required for `asana-cli`)
- `SILENT_CRITIC_TOKEN` - Worker session auth (required for `silent-critic` session execution via `gator`)

**Optional env vars:**
- `ASANA_BASE_URL` - Custom Asana API endpoint (default: `https://app.asana.com/api/1.0`)
- `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, `ASANA_PROJECT` - Asana command defaults
- `ASANA_CLI_CONFIG_HOME`, `ASANA_CLI_DATA_HOME` - Custom paths
- `XDG_CONFIG_HOME` - Override config directory
- `XDG_DATA_HOME` - Override data directory
- `RUST_LOG` - Tracing level control

**Secrets location:**
- Asana PAT: `~/.config/asana-cli/asana-cli.toml` (disk-based, file permissions enforced)
- Silent Critic tokens: Environment variable only (transient, no persistent storage)
- `.env` files: Not used in this codebase (all via environment variables or config files)

## Webhooks & Callbacks

**Incoming:**
- None detected - No webhook endpoints implemented

**Outgoing:**
- None detected - No external webhooks triggered

## Git Integration

**Repository Operations:**
- `git2` (vendored libgit2 0.20) for:
  - Repository discovery (`gator`, `silent-critic`)
  - Project identification via `sha2` hashing of repo path
  - Worktree detection in `crates/gator/src/worktree.rs`
  - Git status inspection

**Repository Metadata:**
- Repository URL: `https://github.com/tftio-stuff/tools` (workspace-level)

## macOS Sandbox Integration (Gator)

**Sandbox Framework:**
- `sandbox-exec` (built-in macOS utility)
- SBPL (Sandbox Policy Language) - Native macOS security model
- Policy generation: `crates/gator/src/sandbox.rs`
- Static base profile: `~/.config/sandbox-exec/agent.sb` (user-provided)
- Policy enforcement: Dynamic rules appended for workdir, worktrees, extra directories

**Process Execution:**
- `std::os::unix::process::CommandExt` for low-level process control
- SBPL policy injected via `sandbox-exec` command wrapper
- No cgroup, seccomp, or Linux security module support

## System Integration Points

**Prompter Integration:**
- `gator` uses `tftio-prompter` library to compose system prompts from TOML profiles
- Code: `crates/gator/src/prompt.rs`
- Features: Profile recursion, markdown deduplication, completion generation

**CLI Common Library:**
- Shared utilities: Completions, health checks, license reporting, auto-update helpers
- Dependency: All CLI crates depend on `tftio-cli-common`

---

*Integration audit: 2026-03-23*
