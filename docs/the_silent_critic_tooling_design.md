# The Silent Critic Tooling Design

**Status:** Working design document
**Purpose:** Implementation-oriented companion to the system specification
**Scope:** Tool behavior, role surfaces, execution model, record model, and v1 integration behavior for `the-silent-critic`

This document sits below [the-silent-critic-system-spec.md](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic-system-spec.md) and alongside [the_silent_critic_formal_appendix.md](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_formal_appendix.md). The system spec defines what The Silent Critic must be. The formal appendix defines the semantics. This document describes how `the-silent-critic` can realize those requirements in a practical first implementation.

---

## 1. Core Implementation Decision

`the-silent-critic` is the trust boundary.

Not the worker. Not the prompt. Not a policy file the worker can read. Not a checklist embedded in orchestration glue.

The tool is responsible for:

- holding the task contract
- separating visible and hidden criteria
- observing the work product
- recording tool-authored evidence
- recording human decisions
- producing the canonical decision log
- exporting a PR-facing rendering of that decision log

If these functions live primarily in the worker's context, the system is no longer implementing The Silent Critic as specified.

## 2. Single Binary, Multiple Role Surfaces

A single executable, `the-silent-critic`, presents different functional surfaces to different callers.

The visible surface is determined by exogenous authorization context, such as:

- environment variables set outside the worker
- signed tokens verified by the tool
- socket or process identity
- filesystem-backed credentials the worker cannot mint

The mechanism is secondary. The required properties are:

1. exogenous to the worker
2. opaque at the mechanism level
3. non-forgeable through normal tool use

The minimum role surfaces are:

- **worker**
  Sees the task goal, visible criteria, and permitted execution/reporting commands.

- **auditor**
  Sees the full contract, evidence record, effective classifications, and hidden criteria.

- **operator**
  Sees everything the auditor sees and can compose contracts, approve hidden criteria, record decisions, and publish outcomes.

The worker-facing surface must not disclose hidden criteria or higher-role operations through normal interaction.

## 3. Self-Description Interface

The worker should not be configured with a static out-of-band API contract. The tool should describe its own visible surface at runtime.

At session start, the caller asks `the-silent-critic` what it can do. The tool returns a role-scoped manifest that includes:

- available commands
- required arguments
- output semantics
- command constraints

The manifest excludes:

- commands not available to the caller
- mention of capabilities not granted to the caller
- implementation details of the authorization mechanism

This makes the visible surface explicit while preserving the hidden acceptance surface.

## 4. Tool as Notary

The worker does not author the structured quality claim.

That is a hard implementation rule.

Worker output may contain summaries, intentions, and explanations, but the durable supervision record is authored by `the-silent-critic` from direct observation and recorded evaluation. Concretely:

- the worker may indicate which files or artifacts it believes are relevant
- the tool inspects the actual artifacts
- the tool records content identity, source-control state, and verification results
- the tool binds those observations to contract criteria

This distinction is not cosmetic. Worker narration is context. Tool-authored evidence is the acceptance substrate.

## 5. Durable State

Agents are treated as replaceable executors. `the-silent-critic` is the system of record.

The tool owns durable state for:

- task contracts
- criterion library entries
- evidence records
- effective classifications
- human decisions
- canonical decision logs
- PR-facing exports

The internal storage format is an implementation choice. SQLite-backed structured records are a plausible v1 default, but the core requirement is that the durable record be tool-authored and queryable.

## 6. Execution Model

### 6.1 Fire-and-Forget Workers

The preferred v1 worker lifecycle is fire-and-forget:

1. a worker is launched for one task
2. it sees the goal and visible contract surface
3. it performs work
4. it invokes `the-silent-critic` commands as needed
5. it terminates

If the outcome is insufficient, a new worker is launched with a revised contract or revised scope.

This model is desirable because it:

- limits context accumulation
- simplifies containment
- reduces long-session leakage risk
- makes workers cheaply replaceable

### 6.2 The Omission Gap

The tool can attest to what happened when it was invoked. It cannot, by itself, guarantee it was invoked every time it should have been.

That omission gap is real.

Therefore v1 needs some combination of:

- session checkpoints
- completeness checks against observed changes
- source-control integration that refuses uncovered work at integration boundaries

The tool provides attestation. The surrounding workflow provides completeness pressure. Neither is sufficient alone.

## 7. Session Flow

A minimal session model looks like:

1. `the-silent-critic session start --task <id> --role worker`
2. worker receives role-scoped manifest and visible contract surface
3. worker performs changes and invokes tool commands
4. `the-silent-critic submit ...`
5. `the-silent-critic session end --task <id>`

Session end performs completeness-oriented checks such as:

- were expected submissions made
- do observed changed files map to the task
- is there uncovered changed scope
- does the contract have unresolved residuals requiring adjudication

If gaps remain, the session is flagged for auditor/operator handling rather than silently accepted.

## 8. Contract Realization

The formal appendix defines the contract grammar. The tooling implication is that `the-silent-critic` must materialize contracts as first-class records with:

- a plain-language goal
- visible criteria
- hidden criteria
- criterion parameters
- declared mediations
- authority requirements

The contract record is the source from which all role-specific views are projected.

The worker should never read raw contract storage directly. All access should pass through the tool surface.

## 9. Criterion Library

Criteria are reusable definitions of what to check and how to check it.

A criterion definition contains at minimum:

- identifier
- claim
- evaluator type
- check specification
- parameter schema

Tier and independence are not intrinsic to the criterion definition. Those are assigned at contract composition time and adjusted through mediation.

### 9.1 Evaluator Types

The implementation must support three evaluator classes:

- **automated**
  `the-silent-critic` runs a command or built-in check and interprets the result.

- **agent-evaluated**
  `the-silent-critic` launches or delegates to an evaluator agent with the appropriate visibility surface.

- **human-evaluated**
  `the-silent-critic` records that a claim requires explicit human judgment.

### 9.2 Library Management

The library should be managed through the tool, not direct file edits.

Minimum v1 operations:

- create criterion
- update criterion
- deprecate criterion
- list criteria
- show criterion

Worker access to the library must be projection-based: only task-relevant visible criteria are exposed.

## 10. Evidence and Classification Pipeline

For each criterion bound into a task contract, the tool must be able to record:

- the raw evaluation result
- the effective epistemic tier
- the effective independence level
- the mediation chain that produced those values

This pipeline should be mechanical:

1. start from base tier and base independence in the contract
2. apply observable mediations
3. apply declared mediations
4. record the effective classification

The tool should treat the formal appendix as the semantic source for this pipeline.

## 11. Residual Uncertainty Surface

The tool should not dump only raw check results. It should compute the unresolved surface that remains after evidence is applied.

That surface includes:

- partial criteria residuals
- contradictory signals
- uncovered changed scope
- claims requiring explicit human judgment
- risk or blast-radius concerns elevated by policy

This residual surface is what should be shown to the auditor or operator as the primary review interface.

## 12. Human Decisions

The tool must support explicit recording of human decisions, not merely final accept/reject state.

At minimum, the tool should support recording:

- accept
- reject
- accept residual
- insufficient evidence
- waive criterion
- rescope
- require rework

Each decision record should capture:

- actor
- timestamp
- evidence basis
- affected claims or residuals
- resulting state transition

Human decisions are first-class structured records, not comments floating beside the system.

## 13. Canonical Decision Log

Every adjudicated task should produce a canonical decision log.

This is the primary human-facing output of the tool, even if the immediate rendering target is a pull request comment or attachment.

The canonical log should contain:

- goal
- work performed
- criteria applied
- evidence
- human decisions
- residual uncertainty
- acceptance outcome

The canonical log is the data model. All human renderings are projections of it.

## 14. Pull Request Export

V1 is explicitly designed to live inside an existing pull-request workflow.

Therefore `the-silent-critic` must export a PR-facing rendering of the decision log.

That export should be:

- human-readable
- concise by default
- expandable into detail
- attributable to a task and change set
- suitable for discussion among humans

The default rendering should be a structured narrative, not a raw machine dump.

### 14.1 Post-Adjudication Disclosure

Criteria hidden from the worker during execution should be disclosed in the PR-facing decision record by default after adjudication.

This is essential to human buy-in. Hidden criteria are a supervision device, not an enduring secrecy model.

### 14.2 Diff References

The export must be able to point back into the world of diffs.

That means the PR-facing record should support references to:

- commits
- files
- changed regions or hunks where feasible

The diff remains supporting context. The decision log remains primary.

## 15. Audit Surface

The audit surface is the append-only operational record underlying the canonical decision log.

It should capture:

- session events
- contract versions
- evidence records
- criterion evaluations
- mediation applications
- residual computations
- human decisions
- published exports

The audit surface should be queryable by:

- task
- session
- criterion
- decision state
- change set

Workers, auditors, and operators should each see only the projection appropriate to their role.

## 16. Source-Control Integration

V1 should integrate with source control in a way that supports, but does not yet replace, the existing pull-request workflow.

Useful enforcement points include:

- pre-commit or pre-push checks for uncovered changed files
- branch-state capture for attested evidence
- commit and diff references embedded in the decision log

The goal is not to make the diff sovereign again. The goal is to ensure the supervision record can point cleanly into the repository state teams already use.

## 17. Deferred Hardening

The following hardening steps are desirable but not required for v1:

- container or kernel isolation beneath the tool
- stronger mitigation of temporal and cross-tool side channels
- batch review interfaces
- speculative execution
- alternate UIs for navigating the decision log

These are downstream improvements. They do not define the core identity of The Silent Critic.

## 18. Open Implementation Questions

The main open implementation questions are:

- What is the minimum useful criterion library for v1?
- What is the smallest viable contract-composition interface?
- Which evidence should be captured automatically versus by explicit invocation?
- What is the minimum useful PR export format?
- Which source-control enforcement point gives the best leverage with the least workflow disruption?

## 19. Summary

In implementation terms, The Silent Critic requires `the-silent-critic` to behave as:

- trust boundary
- contract store
- evidence notary
- classification engine
- residual-uncertainty router
- human-decision recorder
- canonical decision-log producer
- PR-export generator

If the tool can do those things inside an existing pull-request workflow, v1 is on the right track.
