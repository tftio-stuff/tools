# External Integrations

**Analysis Date:** 2026-03-23

## APIs & External Services

**Task/work management:**
- Asana REST API - used by `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs` for authenticated resource access, pagination, rate limiting, retries, and cache-backed offline reads.
  - SDK/Client: `reqwest` with async helpers in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`
  - Auth: `ASANA_PAT` from `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`
  - Overrides: `ASANA_BASE_URL`, `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, `ASANA_PROJECT`, `ASANA_CLI_CONFIG_HOME`, `ASANA_CLI_DATA_HOME` from `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`

**Social/AT Protocol:**
- BlueSky public AppView and PDS endpoints - used by `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs` for handle resolution, authenticated session creation, token refresh, and `com.atproto.repo.listRecords` pagination.
  - SDK/Client: `reqwest` in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs`
  - Auth: `BSKY_APP_PASSWORD` from `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/lib.rs` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`
  - Base URLs: `https://public.api.bsky.app` and `https://bsky.social` in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs`

**Source hosting/downloads:**
- GitHub raw content and releases - used by the shared updater in `/Users/jfb/Projects/tools/main/crates/cli-common/src/update.rs` and described in `/Users/jfb/Projects/tools/main/README.md`.
  - SDK/Client: shell `curl | sh` launched from `/Users/jfb/Projects/tools/main/crates/cli-common/src/update.rs`
  - Auth: none for public downloads

## Data Storage

**Databases:**
- SQLite for `todoer` at XDG-resolved paths from `/Users/jfb/Projects/tools/main/crates/todoer/src/config.rs`, opened by `/Users/jfb/Projects/tools/main/crates/todoer/src/db.rs`.
  - Connection: filesystem path from `XDG_DATA_HOME` or `~/.local/share/todoer/todoer.db`
  - Client: `rusqlite` in `/Users/jfb/Projects/tools/main/crates/todoer/src/db.rs`
- SQLite for `silent-critic` at per-project hashed paths from `/Users/jfb/Projects/tools/main/crates/silent-critic/src/config.rs`, schema in `/Users/jfb/Projects/tools/main/crates/silent-critic/src/db.rs`.
  - Connection: filesystem path from `XDG_DATA_HOME`, config override `db_dir`, or `~/.local/share/silent-critic/<project-hash>/db.sqlite`
  - Client: `rusqlite` in `/Users/jfb/Projects/tools/main/crates/silent-critic/src/db.rs`
- SQLite for BlueSky extraction output at `ProjectDirs`-resolved default location from `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`, schema in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/db.rs`.
  - Connection: default `.../bce/bsky-posts.db` or explicit `--db`
  - Client: `rusqlite` in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/db.rs`

**File Storage:**
- Local filesystem only.
- Asana CLI persists config, templates, filters, and cache under paths resolved in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`.
- Prompter stores profile config and markdown prompt library under `~/.config/prompter/config.toml` and `~/.local/prompter/library` from `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs`.
- Gator reads project-local `.safehouse` and `.gator/policies/*.toml` plus user-global `~/.config/gator/policies/*.toml` from `/Users/jfb/Projects/tools/main/crates/gator/src/config.rs`.

**Caching:**
- Asana CLI maintains in-memory and disk-backed API cache in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`.
  - Default cache path: ProjectDirs data-local cache under `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`
  - TTL/offline controls: `cache_ttl` and `offline` in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`

## Authentication & Identity

**Auth Provider:**
- Custom token/env-var based auth.
  - Asana uses PAT handling with `secrecy::SecretString` in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`.
  - BlueSky uses app-password login to exchange for access/refresh JWTs in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs`.
  - Silent Critic worker flows require `SILENT_CRITIC_TOKEN` in `/Users/jfb/Projects/tools/main/crates/silent-critic/src/main.rs`.

## Monitoring & Observability

**Error Tracking:**
- None detected as an external SaaS integration.

**Logs:**
- `tracing` and `tracing-subscriber` are initialized in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/lib.rs`.
- CLI stderr/JSON error outputs are emitted directly in `/Users/jfb/Projects/tools/main/crates/todoer/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/gator/src/main.rs`, and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`.
- Silent Critic stores durable audit/evidence data in SQLite tables declared in `/Users/jfb/Projects/tools/main/crates/silent-critic/src/db.rs`.

## CI/CD & Deployment

**Hosting:**
- Source and release distribution are GitHub-centric per `/Users/jfb/Projects/tools/main/README.md`, `/Users/jfb/Projects/tools/main/.github/workflows/release-please.yml`, and `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`.
- Crate publishing targets crates.io through `cargo publish` in `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`.

**CI Pipeline:**
- GitHub Actions is the only CI/CD service detected.
  - CI checks: `/Users/jfb/Projects/tools/main/.github/workflows/ci.yml`
  - Release PR automation: `/Users/jfb/Projects/tools/main/.github/workflows/release-please.yml`
  - Tagged binary builds and crates.io publish: `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`

## Environment Configuration

**Required env vars:**
- `ASANA_PAT`, `ASANA_BASE_URL`, `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, `ASANA_PROJECT`, `ASANA_CLI_CONFIG_HOME`, `ASANA_CLI_DATA_HOME` in `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`
- `BSKY_APP_PASSWORD` in `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/lib.rs` and `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/main.rs`
- `SILENT_CRITIC_TOKEN` in `/Users/jfb/Projects/tools/main/crates/silent-critic/src/main.rs`
- `XDG_CONFIG_HOME` and `XDG_DATA_HOME` in `/Users/jfb/Projects/tools/main/crates/todoer/src/config.rs` and `/Users/jfb/Projects/tools/main/crates/silent-critic/src/config.rs`
- `HOME` is used by `/Users/jfb/Projects/tools/main/crates/prompter/src/lib.rs` for default config/library resolution

**Secrets location:**
- Environment variables for active credentials.
- Persisted Asana PAT can be written to the Asana config file resolved by `/Users/jfb/Projects/tools/main/crates/asana-cli/src/config.rs`.
- GitHub Actions secrets are referenced as `GITHUB_TOKEN`, `RELEASE_PLEASE_TOKEN`, and `CARGO_REGISTRY_TOKEN` in `/Users/jfb/Projects/tools/main/.github/workflows/*.yml`.

## Network Integrations

**Git and repo inspection:**
- `git2` integration is used in `/Users/jfb/Projects/tools/main/crates/unvenv/src/main.rs`, `/Users/jfb/Projects/tools/main/crates/gator/src/config.rs`, and Silent Critic project/session discovery modules under `/Users/jfb/Projects/tools/main/crates/silent-critic/src/`.

**Agent tooling:**
- Gator launches external agent binaries (`claude`, `codex`, `gemini`) under `sandbox-exec` in `/Users/jfb/Projects/tools/main/crates/gator/src/agent.rs`.
- Gator composes sandbox policy text from a base SBPL profile at `~/.config/sandbox-exec/agent.sb` in `/Users/jfb/Projects/tools/main/crates/gator/src/sandbox.rs`.
- Gator integrates with the Prompter library from `/Users/jfb/Projects/tools/main/crates/gator/Cargo.toml` and `/Users/jfb/Projects/tools/main/crates/gator/src/prompt.rs`.

## Release & Distribution Integrations

**Incoming release triggers:**
- Tag pattern `*-v*` in `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`
- Pushes to `main` in `/Users/jfb/Projects/tools/main/.github/workflows/release-please.yml`

**Outgoing distribution:**
- `taiki-e/upload-rust-binary-action` uploads release archives in `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`
- `cargo publish` pushes crates to crates.io in `/Users/jfb/Projects/tools/main/.github/workflows/release.yml`
- Release version state is tracked in `/Users/jfb/Projects/tools/main/.release-please-manifest.json`

## Webhooks & Callbacks

**Incoming:**
- None detected in workspace code under `/Users/jfb/Projects/tools/main/crates/`.

**Outgoing:**
- Asana API requests from `/Users/jfb/Projects/tools/main/crates/asana-cli/src/api/client.rs`
- BlueSky/AT Protocol requests from `/Users/jfb/Projects/tools/main/crates/bsky-comment-extractor/src/client.rs`
- GitHub raw install script download from `/Users/jfb/Projects/tools/main/crates/cli-common/src/update.rs`

---

*Integration audit: 2026-03-23*
