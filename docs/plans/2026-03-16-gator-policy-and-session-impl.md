# Gator Policy Profiles and Session Integration — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add named policy profiles (`--policy=<name>`) and silent-critic session integration (`--session=<id>`) to gator, with corresponding contract sandbox declarations in silent-critic.

**Architecture:** Two independent tracks. Track A adds policy loading, deny support, and session mode to gator. Track B adds sandbox declarations to silent-critic's contract model and a `session sandbox` subcommand. The two tools communicate via JSON over stdout — gator shells out to `silent-critic session sandbox <id> --format json` and parses the response. Policy resolution logic lives in gator's library crate so silent-critic can import it for template resolution.

**Tech Stack:** Rust, clap (CLI), serde/serde_json (JSON), git2, rusqlite (silent-critic DB), toml (policy files), dirs (path resolution).

---

## Track A: Gator Changes

### Task A1: Policy file format and loading

Add a `PolicyConfig` type and two-level file resolution to `config.rs`. Policy files are TOML with `[grants]` and `[denies]` sections.

**Files:**
- Modify: `crates/gator/src/config.rs`

**Step 1: Add the `PolicyConfig` struct and parsing**

Add to `crates/gator/src/config.rs`:

```rust
/// Sandbox policy profile loaded from a TOML file.
///
/// Policy files live at `<workdir>/.gator/policies/<name>.toml`
/// (project-level) or `~/.config/gator/policies/<name>.toml`
/// (user-global). Project-level overrides user-global.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyConfig {
    /// Directory grants.
    #[serde(default)]
    pub grants: PolicyGrants,
    /// Directory denies.
    #[serde(default)]
    pub denies: PolicyDenies,
}

/// Grant section of a policy file.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyGrants {
    /// Read-write directory grants.
    #[serde(default)]
    pub rw: Vec<String>,
    /// Read-only directory grants.
    #[serde(default)]
    pub ro: Vec<String>,
}

/// Deny section of a policy file.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyDenies {
    /// Paths to deny all file access.
    #[serde(default)]
    pub paths: Vec<String>,
}
```

Add `serde` to gator's `Cargo.toml` dependencies: `serde.workspace = true` and `toml.workspace = true`.

**Step 2: Add tilde expansion helper**

```rust
/// Expand `~` prefix to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}
```

**Step 3: Add policy resolution function**

```rust
/// Resolve and load a named policy profile.
///
/// Resolution order:
/// 1. `<workdir>/.gator/policies/<name>.toml` (project-level)
/// 2. `~/.config/gator/policies/<name>.toml` (user-global)
///
/// First match wins.
///
/// # Errors
/// Returns an error if the policy name is not found at either level,
/// or if the file cannot be parsed.
pub fn load_policy(name: &str, workdir: &Path) -> Result<PolicyConfig, String> {
    // Project-level
    let project_path = workdir.join(".gator/policies").join(format!("{name}.toml"));
    if project_path.is_file() {
        let content = fs::read_to_string(&project_path)
            .map_err(|e| format!("cannot read policy {}: {e}", project_path.display()))?;
        return toml::from_str(&content)
            .map_err(|e| format!("invalid policy {}: {e}", project_path.display()));
    }

    // User-global
    if let Some(home) = dirs::home_dir() {
        let global_path = home
            .join(".config/gator/policies")
            .join(format!("{name}.toml"));
        if global_path.is_file() {
            let content = fs::read_to_string(&global_path)
                .map_err(|e| format!("cannot read policy {}: {e}", global_path.display()))?;
            return toml::from_str(&content)
                .map_err(|e| format!("invalid policy {}: {e}", global_path.display()));
        }
    }

    Err(format!("policy not found: {name}"))
}

/// Load and merge multiple named policies.
///
/// Policies merge additively: grants union, denies union.
/// Returns the merged extra dirs and deny paths.
///
/// # Errors
/// Returns an error if any policy cannot be loaded.
pub fn load_policies(
    names: &[String],
    workdir: &Path,
) -> Result<(ExtraDirs, Vec<PathBuf>), String> {
    let mut extras = ExtraDirs::default();
    let mut denies = Vec::new();

    for name in names {
        let policy = load_policy(name, workdir)?;
        for p in &policy.grants.rw {
            extras.rw.push(expand_tilde(p));
        }
        for p in &policy.grants.ro {
            extras.ro.push(expand_tilde(p));
        }
        for p in &policy.denies.paths {
            denies.push(expand_tilde(p));
        }
    }

    Ok((extras, denies))
}
```

**Step 4: Write tests**

```rust
#[test]
fn expand_tilde_with_home() {
    let expanded = expand_tilde("~/foo/bar");
    assert!(expanded.to_string_lossy().ends_with("/foo/bar"));
    assert!(!expanded.to_string_lossy().starts_with("~"));
}

#[test]
fn expand_tilde_absolute_unchanged() {
    let expanded = expand_tilde("/absolute/path");
    assert_eq!(expanded, PathBuf::from("/absolute/path"));
}

#[test]
fn load_policy_project_level() {
    let tmp = TempDir::new().unwrap();
    let policy_dir = tmp.path().join(".gator/policies");
    fs::create_dir_all(&policy_dir).unwrap();
    fs::write(
        policy_dir.join("test.toml"),
        "[grants]\nrw = [\"/a\"]\nro = [\"/b\"]\n\n[denies]\npaths = [\"~/.aws\"]\n",
    )
    .unwrap();

    let policy = load_policy("test", tmp.path()).unwrap();
    assert_eq!(policy.grants.rw, vec!["/a"]);
    assert_eq!(policy.grants.ro, vec!["/b"]);
    assert_eq!(policy.denies.paths, vec!["~/.aws"]);
}

#[test]
fn load_policy_not_found() {
    let tmp = TempDir::new().unwrap();
    let result = load_policy("nonexistent", tmp.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn load_policies_merges_additively() {
    let tmp = TempDir::new().unwrap();
    let policy_dir = tmp.path().join(".gator/policies");
    fs::create_dir_all(&policy_dir).unwrap();
    fs::write(
        policy_dir.join("a.toml"),
        "[grants]\nrw = [\"/x\"]\n",
    )
    .unwrap();
    fs::write(
        policy_dir.join("b.toml"),
        "[grants]\nro = [\"/y\"]\n\n[denies]\npaths = [\"/z\"]\n",
    )
    .unwrap();

    let (extras, denies) = load_policies(
        &["a".to_owned(), "b".to_owned()],
        tmp.path(),
    )
    .unwrap();
    assert_eq!(extras.rw, vec![PathBuf::from("/x")]);
    assert_eq!(extras.ro, vec![PathBuf::from("/y")]);
    assert_eq!(denies, vec![PathBuf::from("/z")]);
}
```

**Step 5: Verify**

Run: `cargo test -p tftio-gator -- config -v`
Run: `cargo clippy -p tftio-gator`

**Step 6: Commit**

```
feat(gator): add named policy profile loading with two-level resolution

Adds PolicyConfig TOML format, project-level and user-global resolution,
tilde expansion, and additive merge for multiple policies.
```

---

### Task A2: Add deny support to sandbox policy generation

The sandbox module currently only emits `(allow ...)` rules. Add support for emitting `(deny file-read* file-write* ...)` rules from policies.

**Files:**
- Modify: `crates/gator/src/sandbox.rs`

**Step 1: Add `emit_deny` function and update `assemble_policy` signature**

```rust
/// Emit an SBPL deny rule for a path.
fn emit_deny(path: &Path, out: &mut String) {
    writeln!(
        out,
        "(deny file-read* file-write* (subpath \"{}\"))",
        path.display()
    )
    .unwrap();
}
```

Update `assemble_policy` to accept deny paths:

```rust
pub fn assemble_policy(
    workdir: &Path,
    worktree_info: &WorktreeInfo,
    extra_dirs: &ExtraDirs,
    deny_paths: &[PathBuf],
) -> Result<String, io::Error> {
    // ... existing code ...

    // After extra dir grants, before closing:

    // Policy denies
    if !deny_paths.is_empty() {
        writeln!(policy, "\n;; Policy denies").unwrap();
        for dir in deny_paths {
            writeln!(policy, ";; Deny: {}", dir.display()).unwrap();
            emit_deny(dir, &mut policy);
        }
    }

    Ok(policy)
}
```

**Step 2: Write tests**

```rust
#[test]
fn emit_deny_format() {
    let mut out = String::new();
    emit_deny(Path::new("/Users/jfb/.aws"), &mut out);
    assert_eq!(
        out,
        "(deny file-read* file-write* (subpath \"/Users/jfb/.aws\"))\n"
    );
}

#[test]
fn assemble_policy_with_denies() {
    let base = dirs::home_dir().unwrap().join(DEFAULT_PROFILE_PATH);
    if !base.exists() {
        return;
    }

    let workdir = Path::new("/Users/jfb/Projects/test");
    let wt = WorktreeInfo::default();
    let extras = ExtraDirs::default();
    let denies = vec![PathBuf::from("/Users/jfb/.secret")];

    let policy = assemble_policy(workdir, &wt, &extras, &denies).unwrap();
    assert!(policy.contains("Policy denies"));
    assert!(policy.contains("(deny file-read* file-write* (subpath \"/Users/jfb/.secret\"))"));
}
```

**Step 3: Update all call sites**

Update `lib.rs` to pass `&[]` for deny_paths at the existing call site (preserves current behavior):

```rust
let policy = sandbox::assemble_policy(&workdir, &wt_info, &extras, &[])
```

**Step 4: Verify**

Run: `cargo test -p tftio-gator -v`
Run: `cargo clippy -p tftio-gator`

**Step 5: Commit**

```
feat(gator): add deny rule support to sandbox policy generation
```

---

### Task A3: Add `--policy` and `--session` CLI flags

**Files:**
- Modify: `crates/gator/src/cli.rs`

**Step 1: Add new fields to `Cli` struct**

```rust
    /// Named policy profile (repeatable). Loaded from
    /// <workdir>/.gator/policies/<name>.toml or ~/.config/gator/policies/<name>.toml
    #[arg(long = "policy", value_name = "NAME")]
    pub policies: Vec<String>,

    /// Silent-critic session ID. When set, the contract is the sole
    /// authority on sandbox grants. Incompatible with --workdir,
    /// --add-dirs, --add-dirs-ro, and --policy.
    #[arg(long, value_name = "ID")]
    pub session: Option<String>,
```

**Step 2: Add validation function**

```rust
impl Cli {
    /// Validate mutual exclusivity of session mode vs non-session flags.
    ///
    /// # Errors
    /// Returns an error if `--session` is combined with incompatible flags.
    pub fn validate(&self) -> Result<(), String> {
        if self.session.is_some() {
            let mut conflicts = Vec::new();
            if self.workdir.is_some() {
                conflicts.push("--workdir");
            }
            if !self.add_dirs.is_empty() {
                conflicts.push("--add-dirs");
            }
            if !self.add_dirs_ro.is_empty() {
                conflicts.push("--add-dirs-ro");
            }
            if !self.policies.is_empty() {
                conflicts.push("--policy");
            }
            if !conflicts.is_empty() {
                return Err(format!(
                    "--session is incompatible with: {}",
                    conflicts.join(", ")
                ));
            }
        }
        Ok(())
    }
}
```

**Step 3: Write tests**

```rust
#[test]
fn parse_with_policy() {
    let cli = Cli::parse_from([
        "gator", "claude", "--policy=audit", "--policy=extra",
    ]);
    assert_eq!(cli.policies, vec!["audit", "extra"]);
}

#[test]
fn parse_with_session() {
    let cli = Cli::parse_from([
        "gator", "claude", "--session=abc-123",
    ]);
    assert_eq!(cli.session, Some("abc-123".to_owned()));
}

#[test]
fn validate_session_exclusive() {
    let cli = Cli::parse_from([
        "gator", "claude", "--session=abc", "--workdir=/tmp",
    ]);
    assert!(cli.validate().is_err());
}

#[test]
fn validate_session_with_policy_exclusive() {
    let cli = Cli::parse_from([
        "gator", "claude", "--session=abc", "--policy=audit",
    ]);
    assert!(cli.validate().is_err());
}

#[test]
fn validate_session_alone_ok() {
    let cli = Cli::parse_from([
        "gator", "claude", "--session=abc",
    ]);
    assert!(cli.validate().is_ok());
}

#[test]
fn validate_no_session_ok() {
    let cli = Cli::parse_from(["gator", "claude", "--policy=audit"]);
    assert!(cli.validate().is_ok());
}
```

**Step 4: Call `validate()` from `main.rs`**

In `main.rs`, after `Cli::parse()`, add:

```rust
    if let Err(e) = cli.validate() {
        // ... error handling same pattern as run() errors
    }
```

**Step 5: Verify**

Run: `cargo test -p tftio-gator -v`
Run: `cargo clippy -p tftio-gator`

**Step 6: Commit**

```
feat(gator): add --policy and --session CLI flags with mutual exclusivity
```

---

### Task A4: Implement session mode

Create a new `session.rs` module that shells out to `silent-critic session sandbox` and parses the JSON response. Wire it into `lib.rs` as an alternative execution path.

**Files:**
- Create: `crates/gator/src/session.rs`
- Modify: `crates/gator/src/lib.rs`

**Step 1: Add `serde_json` dependency**

Add to `crates/gator/Cargo.toml`: `serde_json.workspace = true`

**Step 2: Create session module**

```rust
//! Silent-critic session integration.
//!
//! Shells out to `silent-critic session sandbox <id> --format json`
//! to get the complete sandbox specification from the contract.

use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::config::ExtraDirs;

/// Sandbox specification returned by silent-critic.
#[derive(Debug, Deserialize)]
pub struct SessionSandbox {
    /// Working directory.
    pub workdir: PathBuf,
    /// Directory grants.
    pub grants: SessionGrants,
    /// Paths to deny.
    #[serde(default)]
    pub denies: Vec<String>,
}

/// Grant section from the session sandbox response.
#[derive(Debug, Default, Deserialize)]
pub struct SessionGrants {
    /// Read-write directories.
    #[serde(default)]
    pub rw: Vec<String>,
    /// Read-only directories.
    #[serde(default)]
    pub ro: Vec<String>,
}

/// Query silent-critic for the sandbox specification of a session.
///
/// Runs `silent-critic session sandbox <id> --format json` and parses
/// the JSON response.
///
/// # Errors
/// Returns an error if silent-critic is not installed, the session ID
/// is invalid, or the output cannot be parsed.
pub fn fetch_session_sandbox(session_id: &str) -> Result<SessionSandbox, String> {
    let output = Command::new("silent-critic")
        .args(["session", "sandbox", session_id, "--format", "json"])
        .output()
        .map_err(|e| format!("cannot run silent-critic: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "silent-critic session sandbox failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid UTF-8 from silent-critic: {e}"))?;

    serde_json::from_str(&stdout)
        .map_err(|e| format!("cannot parse silent-critic output: {e}"))
}

/// Convert a `SessionSandbox` into the types gator uses internally.
///
/// Expands tilde in all paths.
#[must_use]
pub fn into_sandbox_parts(sandbox: SessionSandbox) -> (PathBuf, ExtraDirs, Vec<PathBuf>) {
    let workdir = sandbox.workdir;

    let mut extras = ExtraDirs::default();
    for p in &sandbox.grants.rw {
        extras.rw.push(crate::config::expand_tilde(p));
    }
    for p in &sandbox.grants.ro {
        extras.ro.push(crate::config::expand_tilde(p));
    }

    let denies: Vec<PathBuf> = sandbox
        .denies
        .iter()
        .map(|p| crate::config::expand_tilde(p))
        .collect();

    (workdir, extras, denies)
}
```

Note: `expand_tilde` in `config.rs` needs to become `pub` (currently not exported). Change `fn expand_tilde` to `pub fn expand_tilde`.

**Step 3: Add `pub mod session;` to `lib.rs`**

**Step 4: Wire session mode into `run()`**

Replace the body of `run()` in `lib.rs` with a branch:

```rust
pub fn run(cli: &Cli) -> Result<(), String> {
    // 1. Prepend clankers to PATH
    if let Some(home) = dirs::home_dir() {
        let clankers = home.join(".local/clankers/bin");
        if clankers.is_dir() {
            let path = std::env::var("PATH").unwrap_or_default();
            unsafe {
                std::env::set_var("PATH", format!("{}:{path}", clankers.display()));
            }
        }
    }

    // Branch: session mode vs non-session mode
    let (workdir, extras, denies, wt_info) = if let Some(session_id) = &cli.session {
        // Session mode: contract is sole authority
        let sandbox = session::fetch_session_sandbox(session_id)?;
        let (workdir, extras, denies) = session::into_sandbox_parts(sandbox);
        let wt_info = worktree::WorktreeInfo::default(); // no auto-detection
        (workdir, extras, denies, wt_info)
    } else {
        // Non-session mode: implicit resolution
        let workdir = config::resolve_workdir(cli.workdir.as_deref())?;

        let mut safehouse_extras = config::load_safehouse_config(&workdir);
        let mut denies = Vec::new();

        // Load named policies
        if !cli.policies.is_empty() {
            let (policy_extras, policy_denies) =
                config::load_policies(&cli.policies, &workdir)?;
            safehouse_extras.rw.extend(policy_extras.rw);
            safehouse_extras.ro.extend(policy_extras.ro);
            denies = policy_denies;
        }

        let extras = config::merge_extra_dirs(safehouse_extras, &cli.add_dirs, &cli.add_dirs_ro);
        let wt_info = worktree::detect_worktrees(&workdir);
        (workdir, extras, denies, wt_info)
    };

    // 5. Assemble sandbox policy
    let policy = sandbox::assemble_policy(&workdir, &wt_info, &extras, &denies)
        .map_err(|e| format!("policy assembly failed: {e}"))?;

    // 6. Dry-run: print policy and exit
    if cli.dry_run {
        eprint!("{policy}");
        return Ok(());
    }

    // 7. Compose prompt (if not --no-prompt)
    let prompt = if cli.no_prompt {
        None
    } else {
        Some(prompt::compose_prompt(&cli.profiles)?)
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

    let err = agent::exec_command(cmd);
    Err(format!("exec failed: {err}"))
}
```

**Step 5: Write tests for session module**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_sandbox_json() {
        let json = r#"{
            "workdir": "/Users/jfb/Projects/test",
            "grants": {
                "rw": ["/Users/jfb/Projects/test"],
                "ro": ["/Users/jfb/Projects/other"]
            },
            "denies": ["~/.aws"]
        }"#;

        let sandbox: SessionSandbox = serde_json::from_str(json).unwrap();
        assert_eq!(sandbox.workdir, PathBuf::from("/Users/jfb/Projects/test"));
        assert_eq!(sandbox.grants.rw, vec!["/Users/jfb/Projects/test"]);
        assert_eq!(sandbox.grants.ro, vec!["/Users/jfb/Projects/other"]);
        assert_eq!(sandbox.denies, vec!["~/.aws"]);
    }

    #[test]
    fn into_sandbox_parts_expands_tilde() {
        let sandbox = SessionSandbox {
            workdir: PathBuf::from("/work"),
            grants: SessionGrants {
                rw: vec!["/a".to_owned()],
                ro: vec!["~/b".to_owned()],
            },
            denies: vec!["~/.aws".to_owned()],
        };

        let (workdir, extras, denies) = into_sandbox_parts(sandbox);
        assert_eq!(workdir, PathBuf::from("/work"));
        assert_eq!(extras.rw, vec![PathBuf::from("/a")]);
        assert!(!extras.ro[0].to_string_lossy().starts_with("~"));
        assert!(!denies[0].to_string_lossy().starts_with("~"));
    }

    #[test]
    fn parse_session_sandbox_minimal() {
        let json = r#"{"workdir": "/tmp", "grants": {}}"#;
        let sandbox: SessionSandbox = serde_json::from_str(json).unwrap();
        assert_eq!(sandbox.workdir, PathBuf::from("/tmp"));
        assert!(sandbox.grants.rw.is_empty());
        assert!(sandbox.denies.is_empty());
    }
}
```

**Step 6: Verify**

Run: `cargo test -p tftio-gator -v`
Run: `cargo clippy -p tftio-gator`
Run: `cargo build -p tftio-gator`

**Step 7: Commit**

```
feat(gator): implement session mode with silent-critic integration

Adds --session=<id> flag that shells out to silent-critic to get the
contract's sandbox specification. In session mode, the contract is the
sole authority -- no implicit workdir, .safehouse, or worktree detection.
```

---

## Track B: Silent-Critic Changes

### Task B1: Add sandbox fields to contract data model

Add sandbox columns to the contracts table and update the Contract struct.

**Files:**
- Modify: `crates/silent-critic/src/models.rs`
- Modify: `crates/silent-critic/src/db.rs`

**Step 1: Add sandbox types to models.rs**

```rust
/// Sandbox configuration for a contract.
///
/// Declares the complete sandbox boundary for the worker.
/// When present, this is the sole authority on filesystem access.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractSandbox {
    /// Working directory for the worker.
    pub workdir: Option<String>,
    /// Named policy template to use as baseline.
    pub policy: Option<String>,
    /// Read-write directory grants.
    #[serde(default)]
    pub rw: Vec<String>,
    /// Read-only directory grants.
    #[serde(default)]
    pub ro: Vec<String>,
    /// Paths to explicitly deny.
    #[serde(default)]
    pub denies: Vec<String>,
}
```

**Step 2: Update Contract struct**

Add `sandbox` field to the `Contract` struct:

```rust
pub struct Contract {
    pub id: String,
    pub session_id: String,
    pub goal: String,
    pub created_at: String,
    pub sandbox: Option<ContractSandbox>,
}
```

**Step 3: Update database schema**

Add a `sandbox_json` TEXT column to the `contracts` table. Store the sandbox as serialized JSON (nullable). Update the `CREATE TABLE` statement:

```sql
CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    goal TEXT NOT NULL,
    sandbox_json TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id)
);
```

**Step 4: Update all DB functions that read/write contracts**

- Insert: serialize `sandbox` to JSON, store in `sandbox_json`
- Select: deserialize `sandbox_json` back to `Option<ContractSandbox>`
- Find all `INSERT INTO contracts` and `SELECT ... FROM contracts` statements in `db.rs` and update them

**Step 5: Write tests**

```rust
#[test]
fn contract_sandbox_roundtrip() {
    let sandbox = ContractSandbox {
        workdir: Some("/Users/jfb/Projects/test".to_owned()),
        policy: Some("audit".to_owned()),
        rw: vec!["/Users/jfb/Projects/test".to_owned()],
        ro: vec!["/Users/jfb/Projects/other".to_owned()],
        denies: vec!["~/.aws".to_owned()],
    };
    let json = serde_json::to_string(&sandbox).unwrap();
    let parsed: ContractSandbox = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.workdir, sandbox.workdir);
    assert_eq!(parsed.rw, sandbox.rw);
}

#[test]
fn contract_sandbox_none_roundtrip() {
    let sandbox: Option<ContractSandbox> = None;
    let json = serde_json::to_string(&sandbox).unwrap();
    assert_eq!(json, "null");
}
```

**Step 6: Verify**

Run: `cargo test -p tftio-silent-critic -v`
Run: `cargo clippy -p tftio-silent-critic`

**Step 7: Commit**

```
feat(silent-critic): add sandbox fields to contract data model

Contracts can now declare a sandbox section with workdir, grants,
denies, and an optional named policy template. Stored as JSON in the
contracts table.
```

---

### Task B2: Update compose flow to accept sandbox configuration

Update `ComposeFromInput` to include sandbox configuration, and update `run_compose_from()` to store it on the contract.

**Files:**
- Modify: `crates/silent-critic/src/commands/session.rs`

**Step 1: Add sandbox to `ComposeFromInput`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeFromInput {
    pub goal: String,
    pub criteria: Vec<ComposeFromCriterion>,
    #[serde(default)]
    pub sandbox: Option<ContractSandbox>,
}
```

**Step 2: Update `run_compose_from()` to pass sandbox through**

In the function where the contract is created, pass `input.sandbox` to the contract creation call. This depends on the exact DB insert function — update the call site to include the sandbox field.

**Step 3: Write tests**

```rust
#[test]
fn compose_from_input_with_sandbox() {
    let json = r#"{
        "goal": "test goal",
        "criteria": [],
        "sandbox": {
            "workdir": "/tmp/test",
            "rw": ["/tmp/test"],
            "denies": ["~/.aws"]
        }
    }"#;
    let input: ComposeFromInput = serde_json::from_str(json).unwrap();
    assert!(input.sandbox.is_some());
    let sb = input.sandbox.unwrap();
    assert_eq!(sb.workdir, Some("/tmp/test".to_owned()));
}

#[test]
fn compose_from_input_without_sandbox() {
    let json = r#"{"goal": "test goal", "criteria": []}"#;
    let input: ComposeFromInput = serde_json::from_str(json).unwrap();
    assert!(input.sandbox.is_none());
}
```

**Step 4: Verify**

Run: `cargo test -p tftio-silent-critic -v`
Run: `cargo clippy -p tftio-silent-critic`

**Step 5: Commit**

```
feat(silent-critic): accept sandbox configuration in compose-from input
```

---

### Task B3: Add `session sandbox` subcommand

Add a new CLI subcommand that outputs the resolved sandbox specification as JSON.

**Files:**
- Modify: `crates/silent-critic/src/cli.rs`
- Modify: `crates/silent-critic/src/commands/session.rs`
- Modify: `crates/silent-critic/src/main.rs` (dispatch)

**Step 1: Add CLI subcommand**

In `cli.rs`, add to the `SessionCommand` enum:

```rust
    /// Output resolved sandbox specification for a session's contract.
    Sandbox {
        /// Session ID (defaults to current session)
        #[arg(value_name = "SESSION_ID")]
        session_id: Option<String>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },
```

**Step 2: Implement `run_session_sandbox()`**

In `session.rs`:

```rust
/// Output the resolved sandbox specification for a session's contract.
///
/// Reads the contract's sandbox section from the database. If the
/// contract references a named policy template, resolves it and merges
/// the grants/denies.
///
/// Output format is JSON matching gator's SessionSandbox struct.
pub fn run_session_sandbox(
    db: &Database,
    session_id: &str,
) -> Result<String, anyhow::Error> {
    let session = db.get_session(session_id)?;
    let contract_id = session
        .contract_id
        .ok_or_else(|| anyhow::anyhow!("session has no contract"))?;
    let contract = db.get_contract(&contract_id)?;
    let sandbox = contract
        .sandbox
        .ok_or_else(|| anyhow::anyhow!("contract has no sandbox configuration"))?;

    // Build the resolved output
    // If sandbox.policy is set, resolve the template and merge
    // For now, pass through directly (policy resolution will use gator's library)
    let output = serde_json::json!({
        "workdir": sandbox.workdir,
        "grants": {
            "rw": sandbox.rw,
            "ro": sandbox.ro,
        },
        "denies": sandbox.denies,
    });

    Ok(serde_json::to_string_pretty(&output)?)
}
```

**Step 3: Add dispatch in main.rs**

Wire `SessionCommand::Sandbox` to call `run_session_sandbox()`.

**Step 4: Write tests**

Test the JSON output format matches what gator expects:

```rust
#[test]
fn session_sandbox_output_format() {
    // Create in-memory DB, insert session + contract with sandbox
    // Call run_session_sandbox
    // Parse output as gator's SessionSandbox format
    // Verify fields match
}
```

**Step 5: Verify**

Run: `cargo test -p tftio-silent-critic -v`
Run: `cargo clippy -p tftio-silent-critic`
Run: `cargo run -p tftio-silent-critic -- session sandbox --help`

**Step 6: Commit**

```
feat(silent-critic): add session sandbox subcommand

Outputs the resolved sandbox specification for a session's contract
as JSON, for consumption by gator --session.
```

---

## Task Dependencies

```
A1 (policy loading)     ──┐
A2 (deny support)       ──┼── A4 (session mode + lib.rs wiring)
A3 (CLI flags)          ──┘         │
                                    │ depends on
B1 (contract model)     ──┐         │
B2 (compose sandbox)    ──┼── B3 (session sandbox cmd) ──┘
                          │
                          └── (B2 depends on B1)
```

Track A tasks 1-3 can proceed in parallel.
Track B tasks are sequential (B1 -> B2 -> B3).
Task A4 depends on A1-A3 and B3.

## Verification Checklist

After all tasks:

```bash
# Full test suite
cargo test --workspace -v

# Clippy
cargo clippy --workspace

# Build both crates
cargo build -p tftio-gator -p tftio-silent-critic

# Non-session mode with policy
mkdir -p /tmp/gator-test/.gator/policies
echo '[grants]\nrw = ["/tmp/extra"]\n\n[denies]\npaths = ["/tmp/secret"]' > /tmp/gator-test/.gator/policies/test.toml
gator claude --workdir=/tmp/gator-test --policy=test --dry-run 2>&1 | grep -E "Extra RW|Deny"

# Session mode (requires silent-critic setup)
# silent-critic project init --name test
# silent-critic session new --worktree /tmp/test
# silent-critic session compose-from < contract-with-sandbox.json
# silent-critic session sandbox <session-id> --format json
# gator claude --session=<session-id> --dry-run
```
