# Gator: Named Policy Profiles and Silent-Critic Session Integration

**Goal:** Add two independent features to gator: (1) named policy profiles for per-context sandbox customization, and (2) silent-critic session integration where the contract is the explicit authority on sandbox boundaries.

**Architecture:** Named policies are TOML files resolved from project-level or user-global directories, layered on top of `.safehouse` and CLI flags. Session integration shells out to `silent-critic session sandbox <id> --format json` to get the complete sandbox spec, bypassing all implicit resolution. The two modes are mutually exclusive.

**Principle:** The silent-critic exists to make implicit context explicit. When a session drives the sandbox, the contract must declare every grant. No auto-detection, no fallback.

---

## Feature 1: Named Policy Profiles

### CLI

```
gator claude --policy=audit rust.full
gator codex --policy=feature --policy=extra-data
```

`--policy=<name>` is repeatable. Multiple policies merge additively.

### File Resolution

For each `--policy=<name>`:

1. `<workdir>/.gator/policies/<name>.toml` (project-level)
2. `~/.config/gator/policies/<name>.toml` (user-global)

First match wins. No merging between levels for the same name.

### File Format

```toml
# ~/.config/gator/policies/audit.toml

[grants]
rw = ["/Users/jfb/Projects/tools"]
ro = ["/Users/jfb/Work/shared-data"]

[denies]
paths = ["~/.ssh", "~/.aws"]
```

All fields are optional. Paths support `~` expansion.

### Grant Resolution Order (Non-Session Mode)

Each layer is additive:

1. `.safehouse` (project root)
2. `--policy` files (in order specified)
3. `--add-dirs` / `--add-dirs-ro` CLI flags

Worktree detection runs independently and adds its own grants.

Denies from policies are appended as `(deny file-read* file-write* ...)` rules in the assembled SBPL, after the static base profile's denies.

---

## Feature 2: Silent-Critic Session Integration

### Contract Sandbox Declaration

The silent-critic contract gains a `sandbox` section, composed during `session compose` alongside criteria:

```
sandbox:
  workdir: /Users/jfb/Projects/tools/feature/gator
  grants:
    rw:
      - /Users/jfb/Projects/tools/feature/gator
      - /Users/jfb/Projects/tools/.git
    ro:
      - /Users/jfb/Projects/tools/main
  denies:
    - ~/.aws
    - ~/Desktop
    - ~/Downloads
  policy: audit
```

Properties:

- **Explicit and complete.** The contract is the sole authority on sandbox grants when `--session` is used. No implicit workdir detection, no `.safehouse`, no worktree auto-discovery.
- **`policy` field is optional.** If present, the named policy template is loaded as a baseline, then the contract's explicit grants/denies are applied on top.
- **Composed during the dialectic.** The `session compose` flow suggests defaults based on worktree detection, but the human confirms or adjusts. Suggestions make it easy; the final declaration is explicit.
- **Auditable.** The sandbox declaration is visible in the decision log. The adjudicator sees what access was granted and whether constraints were intentional.

### Compose Flow

After the criteria dialectic, `session compose` adds a sandbox section:

```
--- Sandbox Configuration ---

Detected worktree: /Users/jfb/Projects/tools/feature/gator
  Git common dir: /Users/jfb/Projects/tools/.git
  Sibling: /Users/jfb/Projects/tools/main

Suggested grants:
  RW: /Users/jfb/Projects/tools/feature/gator
  RW: /Users/jfb/Projects/tools/.git
  RO: /Users/jfb/Projects/tools/main

Use a named policy as baseline? [none/audit/feature/...]: audit
Additional RW directories? [enter to skip]:
Additional RO directories? [enter to skip]:
Additional denies? [enter to skip]: ~/.aws

Sandbox configuration:
  Policy: audit
  RW: /Users/jfb/Projects/tools/feature/gator, /Users/jfb/Projects/tools/.git
  RO: /Users/jfb/Projects/tools/main
  Deny: ~/.aws

Accept? [y/n]:
```

Available named policies are listed from both `~/.config/gator/policies/` and `<workdir>/.gator/policies/`.

### Gator `--session` Flag

```
gator claude --session=<id> -- --model opus
```

Gator shells out to silent-critic:

```
silent-critic session sandbox <id> --format json
```

Silent-critic resolves the contract's sandbox section (loading the named policy template if specified, applying explicit grants/denies) and returns self-contained JSON:

```json
{
  "workdir": "/Users/jfb/Projects/tools/feature/gator",
  "grants": {
    "rw": [
      "/Users/jfb/Projects/tools/feature/gator",
      "/Users/jfb/Projects/tools/.git"
    ],
    "ro": [
      "/Users/jfb/Projects/tools/main"
    ]
  },
  "denies": [
    "/Users/jfb/.aws",
    "/Users/jfb/Desktop",
    "/Users/jfb/Downloads"
  ]
}
```

Gator parses the JSON and uses it as the complete sandbox spec. No workdir resolution, no `.safehouse`, no worktree detection, no `--policy` processing.

### Mutual Exclusivity

`--session` is incompatible with `--workdir`, `--add-dirs`, `--add-dirs-ro`, and `--policy`. Gator errors if both are provided.

If `silent-critic` is not installed or the session ID is invalid, gator errors immediately. No fallback.

### Resolution Responsibility

If the contract references `policy: audit`, silent-critic resolves that template and folds it into the JSON output. Gator never interprets the `policy` field in session mode.

---

## Mode Comparison

|                  | Non-session mode              | Session mode              |
|------------------|-------------------------------|---------------------------|
| Workdir          | Auto-detected or `--workdir`  | From contract             |
| `.safehouse`     | Loaded                        | Ignored                   |
| `--policy`       | Loaded                        | Incompatible (error)      |
| `--add-dirs`     | Applied                       | Incompatible (error)      |
| Worktrees        | Auto-detected                 | From contract             |
| Denies           | From `agent.sb` + policy      | From contract + `agent.sb`|
| Authority        | Implicit (gator decides)      | Explicit (contract decides)|

The static base profile (`agent.sb`) provides the security floor in both modes. Its deny-default posture, launchd listener denies, and container socket denies are always applied. The contract can add denies but cannot remove base ones.

---

## Changes Required

### Gator crate

- `cli.rs`: Add `--policy` (repeatable) and `--session` flags, with mutual exclusivity validation
- `config.rs`: Add `load_policy(name, workdir)` with two-level resolution, `PolicyConfig` struct with grants/denies, tilde expansion
- `sandbox.rs`: Accept denies from policies/session, emit `(deny ...)` rules in dynamic section
- `lib.rs`: Branch on session mode vs non-session mode in `run()`
- `session.rs` (new): Shell out to `silent-critic session sandbox`, parse JSON response

### Silent-critic crate

- Contract data model: Add `sandbox` section (workdir, grants, denies, optional policy reference)
- `session compose`: Add sandbox configuration step to the dialectic
- `session sandbox` (new subcommand): Resolve contract sandbox section, output JSON
- Database schema: Store sandbox section in contract table

### Shared

- Policy file format (TOML) is read by both gator (directly) and silent-critic (for template resolution)
- Policy resolution logic could live in gator's library and be called by silent-critic, or duplicated
