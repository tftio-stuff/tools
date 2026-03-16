# Silent Critic Skills Design

## Context

The silent-critic v1 thin slice is implemented as a CLI binary (`silent-critic`). The current `session compose` command uses `dialoguer` for interactive contract composition via terminal menus. This design replaces that with LLM-mediated skills for Claude Code, making the compose and adjudicate phases conversational rather than menu-driven.

The worker phase is not a skill. It receives a generated prompt containing the visible contract surface and exact shell commands for submitting evidence. The worker runs autonomously in a sandboxed environment.

## Architecture

Three pieces, two of which are skills:

```
Operator (human) + Claude Code
        |
        v
  /critic-compose  (skill)
        |
        | calls silent-critic CLI
        v
  Contract in DB + Worker Prompt on stdout
        |
        | operator feeds prompt to sandboxed worker agent
        v
  Worker (autonomous, sandboxed, no skill)
        |
        | calls silent-critic session submit via shell commands
        v
  Evidence in DB
        |
        | operator invokes after worker completes
        v
  /critic-adjudicate  (skill)
        |
        | calls silent-critic CLI
        v
  Decision in DB + Decision Log
```

## Skill 1: `/critic-compose`

### Trigger

Operator invokes `/critic-compose` or says "set up a silent critic session", "compose a contract", etc.

### Prerequisites

- `silent-critic` binary in PATH
- Project initialized (`silent-critic project init`)
- A worktree path for the task (operator provides or skill infers from cwd)

### Flow

1. **Initialize session.** Skill calls:
   ```bash
   silent-critic session new --worktree <path> --json
   ```

2. **Run discovery.** Skill calls:
   ```bash
   silent-critic session discover --json [--doc <extra-paths>...]
   ```

3. **Read discovery context.** Skill calls:
   ```bash
   silent-critic --json session status
   ```
   Plus reads the discovery contexts from the JSON output to understand what was found (build systems, docs, CI config, git history).

4. **Reason and propose.** The LLM reads the discovery output and the operator's stated goal, then presents a single structured proposal:

   ```
   Goal: Implement feature X

   Proposed criteria:

   1. [visible, must] cargo-test
      Claim: All cargo tests pass
      Check: cargo test --workspace
      Rationale: Cargo.toml detected, standard test gate

   2. [visible, should] clippy-clean
      Claim: No clippy warnings
      Check: cargo clippy --workspace -- -D warnings
      Rationale: Workspace uses clippy pedantic

   3. [hidden, should] no-dead-code
      Claim: No unused functions or imports introduced
      Check: cargo clippy --workspace -- -W dead-code
      Rationale: Hidden to avoid worker gaming removal of code
   ```

   The proposal includes rationale for each criterion and for visibility choices. Hidden criteria have explicit justification for why they should not be visible to the worker.

5. **Operator review.** Operator says "looks good", or requests changes: "make clippy-clean a must", "add a criterion for docs", "don't hide no-dead-code". The skill adjusts and re-presents if needed.

6. **Create contract.** Skill calls:
   ```bash
   silent-critic session compose-from --json-input <<'EOF'
   {
     "goal": "Implement feature X",
     "criteria": [
       {
         "namespace": "testing",
         "name": "cargo-test",
         "claim": "All cargo tests pass",
         "evaluator_type": "automated",
         "check_spec": "cargo test --workspace",
         "visibility": "visible",
         "tier": "must"
       },
       ...
     ]
   }
   EOF
   ```

7. **Generate worker prompt.** Skill calls:
   ```bash
   silent-critic session go --prompt-only --json
   ```
   Presents the worker prompt to the operator for review before launching the worker.

### Error handling

- If project not initialized: skill tells operator to run `silent-critic project init`
- If session already active: skill shows status and asks if operator wants to continue or start fresh
- If discovery finds nothing: skill reports and asks operator for guidance

## Non-interactive contract creation: `session compose-from`

### Command

```
silent-critic session compose-from --json-input
```

Reads a JSON document from stdin defining the contract. Replaces the dialoguer-based `session compose`.

### Input format

```json
{
  "goal": "string, required",
  "criteria": [
    {
      "namespace": "string, required",
      "name": "string, required",
      "claim": "string, required",
      "evaluator_type": "automated | human_judgment | agent_evaluated",
      "check_spec": "string, shell command (empty for human judgment)",
      "visibility": "visible | hidden",
      "tier": "must | should | nice",
      "parameter_schema": "optional JSON schema string",
      "residual_claim": "optional string"
    }
  ]
}
```

### Behavior

1. Validates session is in `composing` state
2. For each criterion:
   - Looks up existing criterion by namespace + name
   - If found and not deprecated: reuses it (updates claim/check_spec if different)
   - If not found: creates new criterion in library
3. Creates contract with goal, binds all criteria with visibility/tier
4. Updates session with contract ID
5. Transitions session to `ready`
6. Returns contract ID and summary as JSON

### Criterion reuse

Matching by namespace + name allows the criterion library to accumulate over time. The operator (via the skill) refines criteria across sessions. The compose-from command preserves this by reusing existing criteria rather than creating duplicates.

## Worker prompt generation: `session go --prompt-only`

### Command

```
silent-critic session go --prompt-only [--json]
```

Instead of spawning a worker process:

1. Generates worker token, stores in session
2. Transitions session to `executing`
3. Outputs a self-contained prompt to stdout

### Prompt content

The prompt is structured natural language containing:

**Goal section:**
```
You are working on: <goal>
```

**Criteria section (visible only):**
```
Your work will be evaluated against these criteria:

1. [must] cargo-test: All cargo tests pass
2. [should] clippy-clean: No clippy warnings
```

**Evidence submission section:**
```
When you believe you have satisfied a criterion, submit evidence by running:

  silent-critic session submit --criterion <criterion-id>

This executes the predefined check for that criterion and records the result.
Available criteria and their IDs:
  <id-1>  cargo-test
  <id-2>  clippy-clean
```

**Boundaries section:**
```
The SILENT_CRITIC_TOKEN environment variable is set. Do not modify or
inspect silent-critic's state beyond the submit command. Your role is
to do the work and submit evidence. You cannot see the full acceptance
surface -- there may be criteria you are not aware of.
```

The prompt does not contain:
- Hidden criteria
- The DB path
- The operator token
- Any information about the adjudication process

### Output format

Plain text by default (suitable for piping into an agent). With `--json`, wraps in the standard ok_response envelope with the prompt as a string field and the worker token as a separate field.

## Skill 2: `/critic-adjudicate`

### Trigger

Operator invokes `/critic-adjudicate` or says "review the session", "adjudicate", "how did the worker do", etc.

### Prerequisites

- Active session in `executing` or `awaiting_adjudication` state

### Flow

1. **End session (if still executing).** Skill calls:
   ```bash
   silent-critic session end --json
   ```
   This computes residuals and transitions to `awaiting_adjudication`.

2. **Load decision log.** Skill calls:
   ```bash
   silent-critic log <contract-id> --format json
   ```

3. **Present summary.** The LLM reads the full log and presents:

   - Goal restated
   - Each criterion with:
     - Claim
     - Whether it was visible or hidden (hidden criteria now disclosed)
     - Evidence: pass/fail, exit code, summary of stdout/stderr
   - Residuals: criteria with no evidence or failing evidence
   - LLM's assessment: what looks clean, what's concerning, what the evidence doesn't cover

4. **Operator decides.** Operator states their decision and reasoning. Types:
   - `accept` -- all criteria satisfied, residuals acceptable
   - `reject` -- criteria not met
   - `accept_residual` -- accepting despite residuals, with stated basis
   - `require_rework` -- send back for another attempt
   - `waive_criterion` -- specific criterion doesn't apply
   - `rescope` -- contract was wrong, not the worker

5. **Record decision.** Skill calls:
   ```bash
   silent-critic decide --contract <id> --type <type> --basis "<operator reasoning>"
   ```

6. **Export log.** Skill calls:
   ```bash
   silent-critic log <contract-id> --format markdown
   ```
   Presents the final decision log. This is the artifact suitable for PR description, review record, etc.

### LLM value-add

The LLM reads raw evidence (test output, clippy output) and summarizes it for the operator. The operator reviews the summary, not the raw output. The LLM can also identify gaps: "tests pass but there are no tests for the new module" -- that's residual uncertainty the check_spec can't catch.

## Changes to silent-critic binary

### Remove

- `dialoguer` dependency from Cargo.toml
- `interactive.rs` module
- `session compose` command (the interactive one)
- `worker.rs` module (process spawning)

### Add

- `session compose-from --json-input` command (stdin JSON -> contract)
- `--prompt-only` flag on `session go` (output prompt instead of spawning)

### Modify

- `session go` without `--prompt-only`: remove process spawning, require `--prompt-only` in v1 (spawning deferred to sandboxed launch, separate concern)

## Skill file locations

Both skills live in the `tftio-dev-tools` plugin:

```
skills/
  critic-compose/
    SKILL.md
  critic-adjudicate/
    SKILL.md
```

## Future considerations

- **Sandbox profiles:** `session go` may eventually generate macOS sandbox profiles alongside the worker prompt. Separate design.
- **Prompter integration:** The worker prompt could be generated as a prompter-compatible TOML profile rather than raw text, allowing composition with other profiles.
- **Criterion evolution:** As the library grows, the compose skill can reason about which existing criteria apply to a new task, reducing operator effort over time.
- **Multi-attempt:** Operator rejects, starts new session with refined criteria. The criterion library carries forward. Decision logs across sessions show the refinement history.
