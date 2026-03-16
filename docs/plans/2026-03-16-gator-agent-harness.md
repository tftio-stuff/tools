# Gator: Agent Sandbox Harness — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `run-sandboxed.sh` and the zshrc wrapper functions with a single Rust binary (`gator`) that handles prompter profile composition, macOS sandbox policy assembly, git worktree detection, and agent dispatch.

**Architecture:** `gator` is a thin orchestrator. It parses CLI args to determine the target agent, prompter profiles, and sandbox configuration. It calls the `prompter` library to compose profiles into a prompt string, generates SBPL policy text by concatenating the static base profile with dynamically-emitted rules, writes the assembled policy to a temp file, and execs `sandbox-exec -f <tmpfile> -- <agent> <injected-prompt-args> <remaining-args>`. The zshrc wrappers collapse to aliases.

**Tech Stack:** Rust (workspace crate), `clap` (CLI), `git2` (worktree detection), `tempfile` (policy temp file), `tftio-prompter` (as library dependency for prompt composition), `tftio-cli-common` (shared utilities).

---

## Design Decisions

### CLI Interface

```
gator <agent> [profiles...] [options] [-- agent-args...]

Arguments:
  <agent>           Agent to run: claude, codex, gemini
  [profiles...]     Prompter profile names (validated against `prompter list`)

Options:
  --workdir=<path>      Explicit working directory (default: git root or pwd)
  --add-dirs=<path>     Extra RW directory grant (repeatable)
  --add-dirs-ro=<path>  Extra RO directory grant (repeatable)
  --no-prompt           Skip prompter integration even if profiles given
  --dry-run             Print assembled policy to stderr, don't exec
  --json                JSON output for errors
  --version             Print version

Everything after `--` is forwarded verbatim to the agent command.
```

**Examples:**
```bash
gator claude rust.full -- --model opus
gator codex --add-dirs=/Users/jfb/Data
gator gemini rust.full python.full --workdir=/Users/jfb/Work/myproject
gator claude --dry-run   # debug: prints assembled SBPL to stderr
```

### Profile Collection

Profiles are positional args between `<agent>` and the first `--` flag or `--` separator. The binary validates each candidate against `prompter list` output (cached per invocation). Invalid profile names cause an error, not silent passthrough — this is a deliberate difference from the zsh wrappers, which silently stopped collecting on first non-profile arg.

Three base profiles are always prepended: `core.baseline`, `core.agent`, `core.git` (matching the current zshrc behavior). The user-specified profiles are appended after these.

### Agent Dispatch

Each agent has a different prompt injection mechanism:

| Agent | Injection |
|-------|-----------|
| `claude` | `--append-system-prompt <prompt-text>` CLI flag |
| `codex` | Write to tempfile, pass `-c experimental_instructions_file=<path>` |
| `gemini` | Write to tempfile, set `GEMINI_SYSTEM_MD=<path>` env var |

The `codex` and `gemini` tempfiles are written to the same `$TMPDIR` directory as the sandbox policy file. The sandbox grants `/tmp` RW, so these are accessible inside the sandbox.

### Sandbox Policy Assembly

The static base profile lives at `~/.config/sandbox-exec/agent.sb` (unchanged from current). Gator reads it verbatim and appends dynamic rules — identical to what `run-sandboxed.sh` currently generates:

1. Ancestor literals for workdir path components
2. Workdir RW subpath grant
3. Git common dir RW grant (if linked worktree)
4. Sibling worktree RO grants
5. Extra directory grants from `--add-dirs` / `--add-dirs-ro` / `.safehouse`

### `.safehouse` Config

Parsed from `<workdir>/.safehouse`. Format unchanged:
```
# comment
add-dirs=/path/to/rw/dir
add-dirs-ro=/path/to/ro/dir
```

### PATH Injection

The current wrappers prepend `~/.local/clankers/bin` to PATH. Gator does the same via `std::env::set_var` before exec.

### Worktree Detection

Uses `git2` (already a workspace dependency with vendored libgit2):
- `Repository::discover(workdir)` to find the repo
- `repo.path()` for git dir, `repo.commondir()` for common dir
- `repo.worktrees()` to list worktree names, then `Worktree::open()` + `worktree.path()` for each

This replaces the shell's `git rev-parse --git-dir`, `--git-common-dir`, and `git worktree list --porcelain`.

---

## Module Layout

```
crates/gator/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entrypoint: parse args, dispatch
│   ├── lib.rs           # Public API, top-level orchestration
│   ├── cli.rs           # clap CLI definitions
│   ├── sandbox.rs       # SBPL policy generation (read base, emit dynamic rules)
│   ├── worktree.rs      # Git worktree detection via git2
│   ├── config.rs        # .safehouse parsing, workdir resolution
│   ├── agent.rs         # Per-agent dispatch (prompt injection, exec)
│   └── prompt.rs        # Prompter library integration
└── tests/
    └── integration.rs   # End-to-end tests (policy assembly, config parsing)
```

---

## Task Breakdown

### Task 0: Add `tftio-prompter` to workspace dependencies and expose `render_to_vec`

The prompter crate's `render_to_writer` is public but the config resolution helpers (`resolve_config_path`, `library_path_for_config_override`) are private. The new crate needs a public function that handles config loading and renders to a buffer.

**Files:**
- Modify: `Cargo.toml` (workspace root, line ~20)
- Modify: `crates/prompter/src/lib.rs` (after `run_render_stdout` at line ~1311)

**Step 1: Add `tftio-prompter` to workspace dependencies**

In the root `Cargo.toml`, add to `[workspace.dependencies]` after the `tftio-cli-common` line:

```toml
tftio-prompter = { path = "crates/prompter", version = "2.1.0" }
```

**Step 2: Add `render_to_vec` public function to prompter**

In `crates/prompter/src/lib.rs`, after the `run_render_stdout` function (line ~1311), add:

```rust
/// Render composed profiles to a byte vector.
///
/// Convenience wrapper around [`render_to_writer`] that handles config
/// resolution and returns the rendered output as bytes. Intended for
/// use by other crates that need prompt composition as a library.
///
/// # Arguments
/// * `profiles` - Profile names to compose
/// * `config_override` - Optional path to custom config file
///
/// # Returns
/// * `Ok(Vec<u8>)` - Rendered prompt content
/// * `Err(String)` - Error message
///
/// # Errors
/// Returns an error if config resolution, profile resolution, or rendering fails.
pub fn render_to_vec(
    profiles: &[String],
    config_override: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let lib = library_path_for_config_override(config_override, &cfg_path)?;
    let mut buf = Vec::new();
    render_to_writer(&cfg, &lib, &mut buf, profiles, None, None, None, false)?;
    Ok(buf)
}

/// List available profile names.
///
/// Returns all profile names from the configuration as a sorted vector.
/// Intended for use by other crates that need to validate profile names.
///
/// # Arguments
/// * `config_override` - Optional path to custom config file
///
/// # Returns
/// * `Ok(Vec<String>)` - Sorted profile names
/// * `Err(String)` - Error message
///
/// # Errors
/// Returns an error if config resolution or parsing fails.
pub fn available_profiles(
    config_override: Option<&Path>,
) -> Result<Vec<String>, String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let mut names: Vec<String> = cfg.profiles.keys().cloned().collect();
    names.sort();
    Ok(names)
}
```

**Step 3: Write tests for the new functions**

In the existing `#[cfg(test)] mod tests` block in `crates/prompter/src/lib.rs`, add:

```rust
#[test]
fn render_to_vec_returns_bytes() {
    // This test uses the real config, so it depends on prompter being configured.
    // If no config exists, it should return an error, not panic.
    let result = render_to_vec(&[], None);
    // Empty profiles should succeed (produces empty or minimal output)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn available_profiles_returns_sorted() {
    let result = available_profiles(None);
    if let Ok(profiles) = result {
        let mut sorted = profiles.clone();
        sorted.sort();
        assert_eq!(profiles, sorted);
    }
    // If no config, error is acceptable
}
```

**Step 4: Verify**

Run: `cargo test -p tftio-prompter -- render_to_vec available_profiles -v`
Run: `cargo clippy -p tftio-prompter`

**Step 5: Commit**

```
feat(prompter): expose render_to_vec and available_profiles public API

Adds two public functions for use by other workspace crates that need
prompt composition as a library rather than a CLI tool.
```

---

### Task 1: Scaffold the `gator` crate

**Files:**
- Modify: `Cargo.toml` (workspace root — add to `members`)
- Modify: `release-please-config.json`
- Modify: `.release-please-manifest.json`
- Create: `crates/gator/Cargo.toml`
- Create: `crates/gator/src/main.rs`
- Create: `crates/gator/src/lib.rs`

**Step 1: Create `crates/gator/Cargo.toml`**

```toml
[package]
name = "tftio-gator"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
description = "Agent sandbox harness — wraps coding agents with macOS sandbox and prompter integration"
keywords = ["cli", "sandbox", "agent", "macos", "security"]
categories = ["command-line-utilities", "development-tools"]

[lints]
workspace = true

[lib]
name = "gator"
path = "src/lib.rs"

[[bin]]
name = "gator"
path = "src/main.rs"

[dependencies]
clap.workspace = true
dirs.workspace = true
git2.workspace = true
tempfile.workspace = true
thiserror.workspace = true
tftio-cli-common.workspace = true
tftio-prompter.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

**Step 2: Create `crates/gator/src/lib.rs`**

```rust
//! Gator — agent sandbox harness.
//!
//! Wraps coding agents (Claude, Codex, Gemini) with macOS sandbox-exec
//! integration and prompter-based system prompt composition.

pub mod agent;
pub mod cli;
pub mod config;
pub mod prompt;
pub mod sandbox;
pub mod worktree;
```

**Step 3: Create `crates/gator/src/main.rs`**

```rust
use clap::Parser;
use gator::cli::Cli;

fn main() {
    let _cli = Cli::parse();
    // Dispatch will be implemented in Task 3
    eprintln!("gator: not yet implemented");
    std::process::exit(1);
}
```

**Step 4: Add to workspace members**

In root `Cargo.toml`, add `"crates/gator"` to the `members` array.

**Step 5: Add to release-please config**

In `release-please-config.json`, add to `packages`:
```json
"crates/gator": {
  "release-type": "rust",
  "component": "gator",
  "package-name": "tftio-gator",
  "bump-minor-pre-major": true
}
```

In `.release-please-manifest.json`, add:
```json
"crates/gator": "0.1.0"
```

**Step 6: Create stub modules**

Create empty stub files so the crate compiles:
- `crates/gator/src/cli.rs` — with `Cli` struct (minimal clap parser)
- `crates/gator/src/sandbox.rs` — empty module
- `crates/gator/src/worktree.rs` — empty module
- `crates/gator/src/config.rs` — empty module
- `crates/gator/src/agent.rs` — empty module
- `crates/gator/src/prompt.rs` — empty module

Minimal `cli.rs`:
```rust
//! CLI argument definitions.

use clap::Parser;

/// Agent sandbox harness.
///
/// Wraps coding agents with macOS sandbox-exec and prompter integration.
#[derive(Parser, Debug)]
#[command(name = "gator", version, about)]
pub struct Cli {
    /// Agent to run (claude, codex, gemini)
    pub agent: String,
}
```

Other stubs are just:
```rust
//! Module description.
```

**Step 7: Verify**

Run: `cargo build -p tftio-gator`
Run: `cargo clippy -p tftio-gator`

**Step 8: Commit**

```
feat(gator): scaffold crate with workspace integration

Adds the gator crate skeleton: Cargo.toml, module stubs, release-please
config. Binary compiles but does not yet implement any functionality.
```

---

### Task 2: Implement CLI argument parsing

**Files:**
- Modify: `crates/gator/src/cli.rs`

**Step 1: Write tests for CLI parsing**

Create `crates/gator/src/cli.rs` with tests at the bottom:

```rust
//! CLI argument definitions.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Known agent targets.
#[derive(Debug, Clone, ValueEnum)]
pub enum Agent {
    /// Anthropic Claude Code
    Claude,
    /// OpenAI Codex
    Codex,
    /// Google Gemini CLI
    Gemini,
}

/// Agent sandbox harness.
///
/// Wraps coding agents with macOS sandbox-exec and prompter integration.
/// Profiles are prompter profile names (e.g., `rust.full`, `python.full`).
/// Base profiles `core.baseline`, `core.agent`, `core.git` are always included.
#[derive(Parser, Debug)]
#[command(name = "gator", version, about)]
pub struct Cli {
    /// Agent to run
    #[arg(value_enum)]
    pub agent: Agent,

    /// Prompter profiles to compose (validated against `prompter list`)
    #[arg(trailing_var_arg = false)]
    pub profiles: Vec<String>,

    /// Explicit working directory (default: git root or pwd)
    #[arg(long, value_name = "PATH")]
    pub workdir: Option<PathBuf>,

    /// Extra read-write directory grant (repeatable)
    #[arg(long = "add-dirs", value_name = "PATH")]
    pub add_dirs: Vec<PathBuf>,

    /// Extra read-only directory grant (repeatable)
    #[arg(long = "add-dirs-ro", value_name = "PATH")]
    pub add_dirs_ro: Vec<PathBuf>,

    /// Skip prompter integration
    #[arg(long)]
    pub no_prompt: bool,

    /// Print assembled policy to stderr without executing
    #[arg(long)]
    pub dry_run: bool,

    /// JSON output for errors
    #[arg(long, global = true)]
    pub json: bool,

    /// Arguments forwarded to the agent command (after --)
    #[arg(last = true)]
    pub agent_args: Vec<String>,
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_minimal() {
        let cli = Cli::parse_from(["gator", "claude"]);
        assert!(matches!(cli.agent, Agent::Claude));
        assert!(cli.profiles.is_empty());
        assert!(cli.agent_args.is_empty());
    }

    #[test]
    fn parse_with_profiles() {
        let cli = Cli::parse_from(["gator", "claude", "rust.full", "python.full"]);
        assert_eq!(cli.profiles, vec!["rust.full", "python.full"]);
    }

    #[test]
    fn parse_with_flags() {
        let cli = Cli::parse_from([
            "gator", "codex",
            "--workdir=/tmp/project",
            "--add-dirs=/tmp/extra",
            "--add-dirs-ro=/tmp/readonly",
        ]);
        assert!(matches!(cli.agent, Agent::Codex));
        assert_eq!(cli.workdir, Some(PathBuf::from("/tmp/project")));
        assert_eq!(cli.add_dirs, vec![PathBuf::from("/tmp/extra")]);
        assert_eq!(cli.add_dirs_ro, vec![PathBuf::from("/tmp/readonly")]);
    }

    #[test]
    fn parse_agent_args_after_separator() {
        let cli = Cli::parse_from([
            "gator", "claude", "rust.full", "--", "--model", "opus",
        ]);
        assert_eq!(cli.profiles, vec!["rust.full"]);
        assert_eq!(cli.agent_args, vec!["--model", "opus"]);
    }

    #[test]
    fn parse_dry_run() {
        let cli = Cli::parse_from(["gator", "gemini", "--dry-run"]);
        assert!(cli.dry_run);
    }

    #[test]
    fn parse_multiple_add_dirs() {
        let cli = Cli::parse_from([
            "gator", "claude",
            "--add-dirs=/a", "--add-dirs=/b", "--add-dirs-ro=/c",
        ]);
        assert_eq!(cli.add_dirs.len(), 2);
        assert_eq!(cli.add_dirs_ro.len(), 1);
    }
}
```

**Step 2: Verify tests pass**

Run: `cargo test -p tftio-gator -- --verbose`

**Step 3: Commit**

```
feat(gator): implement CLI argument parsing

Defines the CLI interface with agent selection, profile names, sandbox
flags, and agent-arg forwarding via clap derive.
```

---

### Task 3: Implement `.safehouse` config parsing and workdir resolution

**Files:**
- Modify: `crates/gator/src/config.rs`

**Step 1: Write tests and implementation**

```rust
//! Configuration loading and workdir resolution.

use std::fs;
use std::path::{Path, PathBuf};

/// Extra directory grants parsed from `.safehouse` or CLI flags.
#[derive(Debug, Default)]
pub struct ExtraDirs {
    /// Read-write directory grants.
    pub rw: Vec<PathBuf>,
    /// Read-only directory grants.
    pub ro: Vec<PathBuf>,
}

/// Resolve the working directory.
///
/// Resolution order:
/// 1. Explicit override (from `--workdir` flag)
/// 2. Git repository root (via `git2`)
/// 3. Physical current working directory
///
/// # Errors
/// Returns an error if the resolved path cannot be canonicalized.
pub fn resolve_workdir(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(dir) = explicit {
        return dir
            .canonicalize()
            .map_err(|e| format!("cannot resolve workdir {}: {e}", dir.display()));
    }

    // Try git root
    if let Ok(repo) = git2::Repository::discover(".") {
        if let Some(workdir) = repo.workdir() {
            if let Ok(canonical) = workdir.canonicalize() {
                return Ok(canonical);
            }
        }
    }

    std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .map_err(|e| format!("cannot resolve cwd: {e}"))
}

/// Parse a `.safehouse` config file from the given directory.
///
/// Returns extra directory grants found in the file. Missing file is not
/// an error — returns empty grants.
///
/// # Format
/// ```text
/// # comment
/// add-dirs=/path/to/rw/dir
/// add-dirs-ro=/path/to/ro/dir
/// ```
pub fn load_safehouse_config(workdir: &Path) -> ExtraDirs {
    let config_file = workdir.join(".safehouse");
    let mut extras = ExtraDirs::default();

    let content = match fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(_) => return extras,
    };

    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(path) = line.strip_prefix("add-dirs=") {
            extras.rw.push(PathBuf::from(path.trim()));
        } else if let Some(path) = line.strip_prefix("add-dirs-ro=") {
            extras.ro.push(PathBuf::from(path.trim()));
        }
    }

    extras
}

/// Merge CLI-provided extra dirs with `.safehouse` config dirs.
///
/// CLI dirs are appended after config dirs (both are additive).
pub fn merge_extra_dirs(
    config_extras: ExtraDirs,
    cli_rw: &[PathBuf],
    cli_ro: &[PathBuf],
) -> ExtraDirs {
    let mut merged = config_extras;
    merged.rw.extend_from_slice(cli_rw);
    merged.ro.extend_from_slice(cli_ro);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_workdir_explicit() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_workdir(Some(tmp.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn resolve_workdir_explicit_missing() {
        let result = resolve_workdir(Some(Path::new("/nonexistent/path/xyz")));
        assert!(result.is_err());
    }

    #[test]
    fn load_safehouse_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert!(extras.rw.is_empty());
        assert!(extras.ro.is_empty());
    }

    #[test]
    fn load_safehouse_with_entries() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".safehouse"),
            "# comment\nadd-dirs=/a/b\nadd-dirs-ro=/c/d\nadd-dirs=/e/f\n",
        )
        .unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert_eq!(extras.rw, vec![PathBuf::from("/a/b"), PathBuf::from("/e/f")]);
        assert_eq!(extras.ro, vec![PathBuf::from("/c/d")]);
    }

    #[test]
    fn load_safehouse_ignores_comments_and_blanks() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".safehouse"),
            "\n# full line comment\n  \nadd-dirs=/x # inline comment\n",
        )
        .unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert_eq!(extras.rw, vec![PathBuf::from("/x")]);
    }

    #[test]
    fn merge_extra_dirs_combines() {
        let config = ExtraDirs {
            rw: vec![PathBuf::from("/a")],
            ro: vec![PathBuf::from("/b")],
        };
        let merged = merge_extra_dirs(
            config,
            &[PathBuf::from("/c")],
            &[PathBuf::from("/d")],
        );
        assert_eq!(merged.rw, vec![PathBuf::from("/a"), PathBuf::from("/c")]);
        assert_eq!(merged.ro, vec![PathBuf::from("/b"), PathBuf::from("/d")]);
    }
}
```

**Step 2: Verify**

Run: `cargo test -p tftio-gator -- config -v`
Run: `cargo clippy -p tftio-gator`

**Step 3: Commit**

```
feat(gator): implement workdir resolution and .safehouse config parsing
```

---

### Task 4: Implement git worktree detection

**Files:**
- Modify: `crates/gator/src/worktree.rs`

**Step 1: Write implementation and tests**

```rust
//! Git worktree detection via git2.

use std::path::{Path, PathBuf};

/// Detected worktree topology for a working directory.
#[derive(Debug, Default)]
pub struct WorktreeInfo {
    /// Git common dir, if this is a linked worktree (needs RW grant).
    /// `None` if this is the main worktree or not a git repo.
    pub common_dir: Option<PathBuf>,
    /// Sibling worktree paths (needs RO grant). Excludes self.
    pub siblings: Vec<PathBuf>,
}

/// Detect git worktree topology for the given working directory.
///
/// Uses `git2` to discover the repository and enumerate worktrees.
/// Returns empty info if the directory is not inside a git repo.
pub fn detect_worktrees(workdir: &Path) -> WorktreeInfo {
    let mut info = WorktreeInfo::default();

    let repo = match git2::Repository::discover(workdir) {
        Ok(r) => r,
        Err(_) => return info,
    };

    // Determine if this is a linked worktree by comparing git_dir to commondir
    let git_dir = repo.path().to_path_buf(); // .git dir for this worktree
    let common_dir = match repo.commondir().canonicalize() {
        Ok(d) => d,
        Err(_) => return info,
    };

    let git_dir_canonical = match git_dir.canonicalize() {
        Ok(d) => d,
        Err(_) => return info,
    };

    // If git_dir != common_dir, this is a linked worktree
    if git_dir_canonical != common_dir {
        info.common_dir = Some(common_dir.clone());
    }

    // Enumerate sibling worktrees
    let workdir_canonical = match workdir.canonicalize() {
        Ok(d) => d,
        Err(_) => return info,
    };

    if let Ok(worktrees) = repo.worktrees() {
        for name in worktrees.iter().flatten() {
            if let Ok(wt) = git2::Worktree::open_from_repository(&repo, name) {
                if let Some(wt_path) = wt.path().canonicalize().ok() {
                    if wt_path != workdir_canonical {
                        info.siblings.push(wt_path);
                    }
                }
            }
        }
    }

    // Also check the main worktree (not listed by repo.worktrees())
    if let Some(main_workdir) = repo.workdir() {
        if let Ok(main_canonical) = main_workdir.canonicalize() {
            if main_canonical != workdir_canonical
                && !info.siblings.contains(&main_canonical)
            {
                info.siblings.push(main_canonical);
            }
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_worktrees_non_git_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let info = detect_worktrees(tmp.path());
        assert!(info.common_dir.is_none());
        assert!(info.siblings.is_empty());
    }

    #[test]
    fn detect_worktrees_main_worktree() {
        // From the main worktree, common_dir should be None (not linked)
        let tmp = tempfile::TempDir::new().unwrap();
        // Init a bare repo
        git2::Repository::init(tmp.path()).unwrap();
        let info = detect_worktrees(tmp.path());
        assert!(info.common_dir.is_none());
    }
}
```

**Step 2: Verify**

Run: `cargo test -p tftio-gator -- worktree -v`

**Step 3: Commit**

```
feat(gator): implement git worktree detection via git2
```

---

### Task 5: Implement SBPL policy generation

**Files:**
- Modify: `crates/gator/src/sandbox.rs`

**Step 1: Write implementation and tests**

```rust
//! macOS sandbox-exec (SBPL) policy generation.
//!
//! Reads the static base profile and appends dynamic rules for the
//! working directory, worktrees, and extra directory grants.

use crate::config::ExtraDirs;
use crate::worktree::WorktreeInfo;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Default location of the static base profile.
const DEFAULT_PROFILE_PATH: &str = ".config/sandbox-exec/agent.sb";

/// Resolve the static base profile path.
fn base_profile_path() -> PathBuf {
    dirs::home_dir()
        .expect("cannot determine home directory")
        .join(DEFAULT_PROFILE_PATH)
}

/// Emit SBPL ancestor literal grants for a path.
///
/// Produces `(allow file-read* (literal "/") (literal "/Users") ...)`
/// for each path component, enabling directory traversal.
fn emit_ancestors(path: &Path, out: &mut String) {
    let path_str = path.to_string_lossy();
    out.push_str("(allow file-read*\n");
    out.push_str("    (literal \"/\")\n");

    let mut cur = String::new();
    for component in path_str.trim_start_matches('/').split('/') {
        if component.is_empty() {
            continue;
        }
        cur.push('/');
        cur.push_str(component);
        write!(out, "    (literal \"{cur}\")\n").unwrap();
    }
    out.push_str(")\n");
}

/// Emit an SBPL read-write subpath grant.
fn emit_rw_grant(path: &Path, out: &mut String) {
    write!(
        out,
        "(allow file-read* file-write* (subpath \"{}\"))\n",
        path.display()
    )
    .unwrap();
}

/// Emit an SBPL read-only subpath grant.
fn emit_ro_grant(path: &Path, out: &mut String) {
    write!(
        out,
        "(allow file-read* (subpath \"{}\"))\n",
        path.display()
    )
    .unwrap();
}

/// Assemble the complete sandbox policy.
///
/// Reads the static base profile and appends dynamic rules for the
/// working directory, worktrees, and extra directory grants.
///
/// # Errors
/// Returns an error if the base profile cannot be read.
pub fn assemble_policy(
    workdir: &Path,
    worktree_info: &WorktreeInfo,
    extra_dirs: &ExtraDirs,
) -> Result<String, io::Error> {
    let base_path = base_profile_path();
    let mut policy = fs::read_to_string(&base_path)?;

    policy.push('\n');
    policy.push_str(";; ===========================================================================\n");
    policy.push_str(";; Dynamic rules — generated by gator\n");
    write!(policy, ";; workdir: {}\n", workdir.display()).unwrap();
    policy.push_str(";; ===========================================================================\n\n");

    // Workdir grants
    write!(policy, ";; Workdir: {}\n", workdir.display()).unwrap();
    emit_ancestors(workdir, &mut policy);
    emit_rw_grant(workdir, &mut policy);
    policy.push('\n');

    // Git common dir (linked worktree)
    if let Some(common) = &worktree_info.common_dir {
        write!(policy, ";; Git common dir: {}\n", common.display()).unwrap();
        emit_ancestors(common, &mut policy);
        emit_rw_grant(common, &mut policy);
        policy.push('\n');
    }

    // Sibling worktrees (RO)
    for sibling in &worktree_info.siblings {
        write!(policy, ";; Sibling worktree: {}\n", sibling.display()).unwrap();
        emit_ancestors(sibling, &mut policy);
        emit_ro_grant(sibling, &mut policy);
        policy.push('\n');
    }

    // Extra RW dirs
    for dir in &extra_dirs.rw {
        write!(policy, ";; Extra RW: {}\n", dir.display()).unwrap();
        emit_ancestors(dir, &mut policy);
        emit_rw_grant(dir, &mut policy);
    }

    // Extra RO dirs
    for dir in &extra_dirs.ro {
        write!(policy, ";; Extra RO: {}\n", dir.display()).unwrap();
        emit_ancestors(dir, &mut policy);
        emit_ro_grant(dir, &mut policy);
    }

    Ok(policy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn emit_ancestors_simple() {
        let mut out = String::new();
        emit_ancestors(Path::new("/Users/jfb/Projects"), &mut out);
        assert!(out.contains("(literal \"/\")"));
        assert!(out.contains("(literal \"/Users\")"));
        assert!(out.contains("(literal \"/Users/jfb\")"));
        assert!(out.contains("(literal \"/Users/jfb/Projects\")"));
    }

    #[test]
    fn emit_rw_grant_format() {
        let mut out = String::new();
        emit_rw_grant(Path::new("/Users/jfb/Work"), &mut out);
        assert_eq!(
            out,
            "(allow file-read* file-write* (subpath \"/Users/jfb/Work\"))\n"
        );
    }

    #[test]
    fn emit_ro_grant_format() {
        let mut out = String::new();
        emit_ro_grant(Path::new("/Users/jfb/Data"), &mut out);
        assert_eq!(
            out,
            "(allow file-read* (subpath \"/Users/jfb/Data\"))\n"
        );
    }

    #[test]
    fn assemble_policy_includes_dynamic_section() {
        // This test requires the static base profile to exist.
        // Skip if not present (CI environments).
        let base = dirs::home_dir().unwrap().join(DEFAULT_PROFILE_PATH);
        if !base.exists() {
            return;
        }

        let workdir = Path::new("/Users/jfb/Projects/test");
        let wt = WorktreeInfo::default();
        let extras = ExtraDirs::default();

        let policy = assemble_policy(workdir, &wt, &extras).unwrap();
        assert!(policy.contains("(deny default)"));
        assert!(policy.contains("Dynamic rules"));
        assert!(policy.contains("/Users/jfb/Projects/test"));
    }
}
```

**Step 2: Verify**

Run: `cargo test -p tftio-gator -- sandbox -v`

**Step 3: Commit**

```
feat(gator): implement SBPL policy assembly
```

---

### Task 6: Implement prompter integration

**Files:**
- Modify: `crates/gator/src/prompt.rs`

**Step 1: Write implementation and tests**

```rust
//! Prompter library integration.
//!
//! Calls the prompter crate's public API to compose profiles into a
//! prompt string for injection into agent commands.

/// Base profiles always prepended to user-specified profiles.
const BASE_PROFILES: &[&str] = &["core.baseline", "core.agent", "core.git"];

/// Validate that all profile names exist in prompter's configuration.
///
/// # Errors
/// Returns an error listing unknown profile names.
pub fn validate_profiles(user_profiles: &[String]) -> Result<(), String> {
    let available = prompter::available_profiles(None)?;

    let unknown: Vec<&str> = user_profiles
        .iter()
        .filter(|p| !available.contains(p))
        .map(String::as_str)
        .collect();

    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown profiles: {}", unknown.join(", ")))
    }
}

/// Build the full profile list (base + user-specified).
pub fn build_profile_list(user_profiles: &[String]) -> Vec<String> {
    let mut profiles: Vec<String> = BASE_PROFILES.iter().map(|&s| s.to_owned()).collect();
    profiles.extend_from_slice(user_profiles);
    profiles
}

/// Compose profiles into a prompt string.
///
/// Prepends base profiles, validates all names, then renders via the
/// prompter library.
///
/// # Errors
/// Returns an error if profile validation or rendering fails.
pub fn compose_prompt(user_profiles: &[String]) -> Result<String, String> {
    validate_profiles(user_profiles)?;
    let all_profiles = build_profile_list(user_profiles);

    let bytes = prompter::render_to_vec(&all_profiles, None)?;
    String::from_utf8(bytes).map_err(|e| format!("prompt contains invalid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_profile_list_prepends_base() {
        let user = vec!["rust.full".to_owned()];
        let all = build_profile_list(&user);
        assert_eq!(all[0], "core.baseline");
        assert_eq!(all[1], "core.agent");
        assert_eq!(all[2], "core.git");
        assert_eq!(all[3], "rust.full");
    }

    #[test]
    fn build_profile_list_empty_user() {
        let all = build_profile_list(&[]);
        assert_eq!(all.len(), 3);
    }
}
```

**Step 2: Verify**

Run: `cargo test -p tftio-gator -- prompt -v`

**Step 3: Commit**

```
feat(gator): implement prompter profile composition
```

---

### Task 7: Implement agent dispatch and `exec`

**Files:**
- Modify: `crates/gator/src/agent.rs`
- Modify: `crates/gator/src/lib.rs`
- Modify: `crates/gator/src/main.rs`

**Step 1: Implement agent dispatch**

`crates/gator/src/agent.rs`:

```rust
//! Agent-specific command building and execution.

use crate::cli::Agent;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

/// Write prompt text to a temp file, returning the path.
///
/// The caller must keep the `NamedTempFile` alive until exec to prevent
/// cleanup. Since we exec, the file handle is inherited by the child.
///
/// # Errors
/// Returns an error if the temp file cannot be written.
pub fn write_prompt_tempfile(prompt: &str) -> Result<NamedTempFile, io::Error> {
    use std::io::Write;
    let mut f = NamedTempFile::new()?;
    f.write_all(prompt.as_bytes())?;
    f.flush()?;
    Ok(f)
}

/// Build the full command for sandbox-exec, including prompt injection.
///
/// Returns the `Command` ready for exec (does not execute it).
/// Also returns any temp files that must be kept alive until exec.
///
/// # Errors
/// Returns an error if prompt tempfile creation fails.
pub fn build_command(
    agent: &Agent,
    policy_path: &Path,
    prompt: Option<&str>,
    agent_args: &[String],
) -> Result<(Command, Vec<NamedTempFile>), io::Error> {
    let mut cmd = Command::new("sandbox-exec");
    cmd.arg("-f").arg(policy_path).arg("--");

    let mut tempfiles: Vec<NamedTempFile> = Vec::new();

    // Agent binary name
    cmd.arg(agent.to_string());

    // Inject prompt per agent's mechanism
    if let Some(prompt_text) = prompt {
        match agent {
            Agent::Claude => {
                cmd.arg("--append-system-prompt").arg(prompt_text);
            }
            Agent::Codex => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.arg("-c")
                    .arg(format!("experimental_instructions_file={}", path.display()));
                tempfiles.push(f);
            }
            Agent::Gemini => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.env("GEMINI_SYSTEM_MD", path);
                tempfiles.push(f);
            }
        }
    }

    // Forward remaining agent args
    cmd.args(agent_args);

    Ok((cmd, tempfiles))
}

/// Execute the sandboxed agent command (replaces current process).
///
/// # Errors
/// Returns the OS error if exec fails.
pub fn exec_command(mut cmd: Command) -> io::Error {
    cmd.exec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_claude_no_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Claude,
            tmp.path(),
            None,
            &["--help".to_owned()],
        )
        .unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.contains(&"claude".to_owned()));
        assert!(args.contains(&"--help".to_owned()));
        assert!(temps.is_empty());
    }

    #[test]
    fn build_command_claude_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Claude,
            tmp.path(),
            Some("system prompt"),
            &[],
        )
        .unwrap();
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.contains(&"--append-system-prompt".to_owned()));
        assert!(args.contains(&"system prompt".to_owned()));
        assert!(temps.is_empty()); // claude uses inline arg, no tempfile
    }

    #[test]
    fn build_command_codex_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Codex,
            tmp.path(),
            Some("codex prompt"),
            &[],
        )
        .unwrap();
        assert_eq!(temps.len(), 1);
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.iter().any(|a| a.starts_with("experimental_instructions_file=")));
    }

    #[test]
    fn build_command_gemini_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Gemini,
            tmp.path(),
            Some("gemini prompt"),
            &[],
        )
        .unwrap();
        assert_eq!(temps.len(), 1);
        // Env var set on Command
        let envs: Vec<_> = cmd.get_envs().collect();
        assert!(envs.iter().any(|(k, _)| k == "GEMINI_SYSTEM_MD"));
    }
}
```

**Step 2: Implement top-level orchestration in `lib.rs`**

```rust
//! Gator — agent sandbox harness.
//!
//! Wraps coding agents (Claude, Codex, Gemini) with macOS sandbox-exec
//! integration and prompter-based system prompt composition.

pub mod agent;
pub mod cli;
pub mod config;
pub mod prompt;
pub mod sandbox;
pub mod worktree;

use cli::Cli;
use std::path::PathBuf;

/// Run the gator harness. Returns an exit code.
///
/// # Errors
/// Returns an error string if any step fails.
pub fn run(cli: Cli) -> Result<(), String> {
    // 1. Prepend clankers to PATH
    if let Some(home) = dirs::home_dir() {
        let clankers = home.join(".local/clankers/bin");
        if clankers.is_dir() {
            let path = std::env::var("PATH").unwrap_or_default();
            std::env::set_var("PATH", format!("{}:{path}", clankers.display()));
        }
    }

    // 2. Resolve workdir
    let workdir = config::resolve_workdir(cli.workdir.as_deref())?;

    // 3. Load .safehouse + merge CLI extra dirs
    let safehouse_extras = config::load_safehouse_config(&workdir);
    let extras = config::merge_extra_dirs(safehouse_extras, &cli.add_dirs, &cli.add_dirs_ro);

    // 4. Detect worktrees
    let wt_info = worktree::detect_worktrees(&workdir);

    // 5. Assemble sandbox policy
    let policy = sandbox::assemble_policy(&workdir, &wt_info, &extras)
        .map_err(|e| format!("policy assembly failed: {e}"))?;

    // 6. Dry-run: print policy and exit
    if cli.dry_run {
        eprint!("{policy}");
        return Ok(());
    }

    // 7. Compose prompt (if profiles given and not --no-prompt)
    let prompt = if !cli.no_prompt && !cli.profiles.is_empty() {
        Some(prompt::compose_prompt(&cli.profiles)?)
    } else if !cli.no_prompt && cli.profiles.is_empty() {
        // No user profiles, but still compose base profiles
        Some(prompt::compose_prompt(&[])?)
    } else {
        None
    };

    // 8. Write policy to temp file
    let policy_file = tempfile::Builder::new()
        .prefix("gator-policy-")
        .suffix(".sb")
        .tempfile()
        .map_err(|e| format!("cannot create policy tempfile: {e}"))?;

    std::fs::write(policy_file.path(), &policy)
        .map_err(|e| format!("cannot write policy: {e}"))?;

    // 9. Build and exec command
    let (cmd, _tempfiles) = agent::build_command(
        &cli.agent,
        policy_file.path(),
        prompt.as_deref(),
        &cli.agent_args,
    )
    .map_err(|e| format!("cannot build command: {e}"))?;

    // exec replaces the process — this only returns on error
    let err = agent::exec_command(cmd);
    Err(format!("exec failed: {err}"))
}
```

**Step 3: Update `main.rs`**

```rust
use clap::Parser;
use gator::cli::Cli;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    if let Err(e) = gator::run(cli) {
        if json {
            eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
        } else {
            eprintln!("gator: {e}");
        }
        std::process::exit(1);
    }
}
```

**Step 4: Verify**

Run: `cargo test -p tftio-gator -v`
Run: `cargo clippy -p tftio-gator`
Run: `cargo build -p tftio-gator`

**Step 5: Commit**

```
feat(gator): implement agent dispatch and top-level orchestration

Wires together config, worktree, sandbox, prompt, and agent modules
into the main run() function. Execs sandbox-exec with assembled policy.
```

---

### Task 8: Update shell wrappers and verify end-to-end

**Files:**
- Modify: `~/.zshrc`

**Step 1: Replace wrapper functions**

Replace the entire `__prompter_collect_profiles_zsh`, `__sandbox_split_args`, `sandbox-claude`, `sandbox-codex`, `sandbox-gemini` block with:

```zsh
# Sandboxed AI agent wrappers — thin shims over gator
sandbox-claude() { gator claude "$@"; }
sandbox-codex()  { gator codex "$@"; }
sandbox-gemini() { gator gemini "$@"; }
```

The old `__prompter_collect_profiles_zsh` and `__sandbox_split_args` functions are deleted — gator handles all arg parsing internally.

**Step 2: Install gator binary**

Run: `cargo install --path crates/gator`

Or for development: ensure `~/.cargo/bin` is on PATH and run `cargo build -p tftio-gator`, then symlink or alias.

**Step 3: End-to-end verification**

```bash
# 1. Syntax check — gator assembles policy, sandbox-exec validates
gator claude --dry-run 2>/tmp/gator-policy.sb && sandbox-exec -f /tmp/gator-policy.sb /usr/bin/true

# 2. Help output
gator --help
gator claude --help

# 3. Dry-run from this repo (should detect worktrees)
(cd /Users/jfb/Projects/tools/feature/gator && gator claude --dry-run 2>&1 | grep -c "worktree")

# 4. Extra dirs
gator claude --add-dirs=/Users/jfb/Documents --dry-run 2>&1 | grep "Extra RW"

# 5. .safehouse config (create temp, verify)
echo 'add-dirs-ro=/Users/jfb/Data/reference' > /tmp/test-safehouse-dir/.safehouse
gator claude --workdir=/tmp/test-safehouse-dir --dry-run 2>&1 | grep "Extra RO"

# 6. Profile composition (requires prompter configured)
gator claude rust.full --dry-run

# 7. Shell wrappers
source ~/.zshrc && sandbox-claude --help
```

**Step 4: Commit**

```
feat(gator): replace shell wrappers with gator aliases

The zshrc sandbox-claude/codex/gemini functions are now thin shims
that delegate all logic to the gator binary.
```

---

## Dependency Summary

New workspace dependency addition:
```toml
tftio-prompter = { path = "crates/prompter", version = "2.1.0" }
```

Gator crate dependencies (all already in `[workspace.dependencies]`):
- `clap` — CLI parsing
- `dirs` — home directory resolution
- `git2` — worktree detection
- `tempfile` — policy and prompt temp files
- `thiserror` — error types
- `tftio-cli-common` — shared utilities
- `tftio-prompter` — prompt composition

No new external dependencies introduced.

## Files Changed Summary

| File | Action | Task |
|------|--------|------|
| `Cargo.toml` (root) | Modify — add workspace dep + member | 0, 1 |
| `release-please-config.json` | Modify — add gator entry | 1 |
| `.release-please-manifest.json` | Modify — add gator entry | 1 |
| `crates/prompter/src/lib.rs` | Modify — add `render_to_vec`, `available_profiles` | 0 |
| `crates/gator/Cargo.toml` | Create | 1 |
| `crates/gator/src/lib.rs` | Create | 1, 7 |
| `crates/gator/src/main.rs` | Create | 1, 7 |
| `crates/gator/src/cli.rs` | Create | 2 |
| `crates/gator/src/config.rs` | Create | 3 |
| `crates/gator/src/worktree.rs` | Create | 4 |
| `crates/gator/src/sandbox.rs` | Create | 5 |
| `crates/gator/src/prompt.rs` | Create | 6 |
| `crates/gator/src/agent.rs` | Create | 7 |
| `~/.zshrc` | Modify — collapse wrappers | 8 |
