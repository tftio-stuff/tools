# CRATES.md

This file is a reference to CLAUDE.md and contains expanded crate documentation for the tftio workspace.

## Workspace Crate Reference

For the overview, see [`CLAUDE.md`](/Users/jfb/Projects/tools/feature-agent-mode/CLAUDE.md).

This file provides detailed documentation for each crate in the workspace.

## Crate Summaries

- **`cli-common`** (`tftio-cli-common`): Shared library for CLI functionality (completions, doctor, output, update)
- **`prompter`** (`tftio-prompter`): Compose reusable prompt snippets from markdown libraries using TOML profiles
- **`unvenv`** (`tftio-unvenv`): Detect Python virtual environments not ignored by Git
- **`asana-cli`** (`tftio-asana-cli`): Interface to the Asana API
- **`todoer`** (`tftio-todoer`): Global todo list manager for LLM agents
- **`silent-critic`** (`tftio-silent-critic`): Supervision framework for agentic software development
- **`gator`** (`tftio-gator`): Agent sandbox harness and launcher
- **`bsky-comment-extractor`** (`tftio-bsky-comment-extractor`): BlueSky extraction and query CLI

## Crate-Specific Details

### `cli-common` (`tftio-cli-common`)

**Location**: `crates/cli-common/`
**Type**: Library (not installable)
**Purpose**: Shared utilities used by other crates in the workspace

**Provided functionality**:
- CLI completions generation
- `doctor` command for health checks
- Output formatting helpers
- License detection
- Version detection and update checking

**Dependencies**:
- `clap` - CLI parsing
- `clap_complete` - Completions generation
- `colored` - Colored output
- `is-terminal` - Terminal detection

**Use cases**: Other crates should depend on this workspace library for shared CLI functionality.

---

### `prompter` (`tftio-prompter`)

**Location**: `crates/prompter/`
**Type**: Binary + Library
**Binary name**: `prompter`
**Version**: 2.2.0
**License**: MIT

**Purpose**: Compose prompt snippets from TOML profiles

**Functionality**:
- Loads prompt snippets from markdown files
- Supports TOML profile definitions
- Handles recursive profile dependencies
- Deduplicates markdown content
- Progress bars during large file operations

**Dependencies**:
- `chrono` - Date/time handling
- `indicatif` - Progress bar output
- `serde`, `toml` - Configuration parsing

**Common commands**:
```bash
prompter list              # List available profiles
prompter load <profile>    # Load a profile
```

---

### `unvenv` (`tftio-unvenv`)

**Location**: `crates/unvenv/`
**Type**: Binary
**Binary name**: `unvenv`
**Version**: 2.2.0
**License**: MIT

**Purpose**: Detect Python virtual environments not ignored by Git

**Functionality**:
- Scans repository for Python venvs
- Checks `.gitignore` compliance
- Uses `git2` (vendored libgit2) for repository traversal
- Outputs colored warnings for problematic venvs

**Dependencies**:
- `git2` - Git library (vendored libgit2)
- `walkdir` - Directory iteration

**Common usage**:
```bash
unvenv              # Scan and report unignored venvs
```

---

### `asana-cli` (`tftio-asana-cli`)

**Location**: `crates/asana-cli/`
**Type**: Binary + Library
**Binary name**: `asana-cli`
**Version**: 2.2.0
**License**: MIT

**Purpose**: Interface to the Asana API

**Functionality**:
- Asana API wrapper with async operations
- Multipart file uploads
- Streaming response support
- Tracing-based observability
- Secret token handling with `secrecy`

**Supported models**:
- Tasks
- Projects
- Workspaces
- Users
- Tags
- Stories
- Sections
- Attachments
- Custom fields

**API client features**:
- Pagination support
- Request/response logging
- Error handling and retry logic

**Dependencies**:
- `reqwest` with extra features: `multipart`, `stream`
- `tokio` with extra features: `fs`, `signal`, `time`
- `tracing`, `secrecy` - Observability and secrets

**Common commands**:
```bash
asana-cli list projects
asana-cli create task --project <id> --summary <summary>
```

---

### `todoer` (`tftio-todoer`)

**Location**: `crates/todoer/`
**Type**: Binary + Library
**Binary name**: `todoer`
**Version**: 2.2.0
**License**: CC0-1.0 (exception in workspace)

**Purpose**: Global todo list manager for LLM agents

**Functionality**:
- SQLite-backed persistence (bundled `rusqlite`)
- Reads `.todoer.toml` configuration
- CLI commands: `new`, `list`, `init`
- UUID-based task identification
- Chronological sorting

**Dependencies**:
- `rusqlite` - Bundled SQLite
- `uuid` - Task IDs
- `chrono` - Timestamps

**Configuration**: Creates and reads from `.todoer.toml` in the current directory.

**Common commands**:
```bash
todoer init              # Initialize database
todoer new "Task name"   # Create new task
todoer list              # List all tasks
```

## Version Management

The workspace uses a shared repo-wide release version via release-please. Release PRs are created on push to `main`, and component tags for a release wave share the same version number (for example `prompter-v2.2.0` and `todoer-v2.2.0`).

## License Information

| Crate | License |
|-------|---------|
| `cli-common` | MIT (workspace default) |
| `prompter` | MIT (workspace default) |
| `unvenv` | MIT (workspace default) |
| `asana-cli` | MIT (workspace default) |
| `todoer` | CC0-1.0 (workspace exception) |

## Build & Distribution

**Development build**: `just build-crate <crate>`
**Release build**: `just build-release`
**Single crate**: `cargo build -p <crate-name>`

**Installation**: Release binaries can be installed via:
```bash
curl -fsSL https://github.com/tftio-stuff/tools/releases/latest/download/<binary>-*.tar.gz | tar xz
mv <binary> /usr/local/bin/
```
