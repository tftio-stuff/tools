# The Silent Critic Formal Appendix

**Status:** Working document
**Purpose:** Formal companion to the system specification. Defines the exact semantics of contracts, evidence, containment, residual uncertainty, and decision-log composition.

This appendix supports the companion system specification. The system spec is normative at the architectural level. This appendix makes key terms precise enough to implement, audit, and criticize.

---

## 1. Formal Scope

This appendix formalizes five things:

1. task contracts
2. evidence classification
3. containment properties
4. residual-uncertainty composition
5. decision-log composition

It does not attempt a full proof system. The goal is disciplined semantics, not mathematical completeness.

## 2. Independence

### Definition

Independence measures the degree to which an evidence-producing process is free from shared context with the worker process whose output it evaluates.

Let **W** be the context available to the worker and **E** be the context available to the evidence producer. Context includes prompts, prior conversation, repository visibility, implementation visibility, model family, optimization objective, and temporal proximity.

Independence is inversely related to the overlap between **W** and **E**.

At the extremes:

- **Minimum independence:** `W = E`. The same agent, in the same session, produces the artifact and the evidence. This is self-verification.
- **Maximum practical independence:** The evidence producer shares only the task-level specification or empirical environment, not the worker's reasoning, implementation context, or optimization incentives.

### Factors That Reduce Independence

| Factor | Effect |
|--------|--------|
| Same agent instance | Shared weights, prompt, and optimization objective |
| Same session | Shared intermediate reasoning and ambient context |
| Access to implementation | Evaluator can converge on the implementation rather than the intent |
| Shared model family | Shared training biases and likely failure modes |
| Immediate temporal proximity | Shared transient context and short-horizon reasoning |

### Practical Independence Levels

For classification purposes, the system uses an ordered lattice:

1. **minimum**
   Same agent, same session, same implementation context.
2. **low**
   Different agent instance or run, but same model family and implementation visibility.
3. **moderate**
   Different evaluator with reduced implementation visibility or specification-first access.
4. **high**
   Different model family, specification-oriented evaluation, or strong separation from the worker context.
5. **maximum**
   Empirical or specification-derived evaluation with minimal shared context.

### Composition

Independence does not compose additively.

For evidence signals `e1 ... en` addressing the same claim:

    I_effective(claim) ≈ max(I_e1, I_e2, ..., I_en)

Low-independence signals can reinforce routing confidence, but they do not accumulate into high-independence assurance.

## 3. Epistemic Tiers

### Definition

An epistemic tier classifies what an evidence signal is sufficient to do for a specific claim in a specific context.

### Order

The tier order is:

    decisive > partial > strong-evidence > weak-evidence > non-adjudicative

### Meanings

| Tier | Meaning | Sufficient for |
|------|---------|----------------|
| `decisive` | Closes the claim in context | Acceptance without additional review of that claim |
| `partial` | Closes only a defined subset of the claim | Residual narrowing |
| `strong-evidence` | High-value routing signal | Escalation and prioritization |
| `weak-evidence` | Exploratory or low-authority signal | Hypothesis generation |
| `non-adjudicative` | Context only | Never acceptance |

### Assignment

A signal's tier is a function of:

- the signal
- the claim
- the task context
- the signal's independence

The same check may occupy different tiers across different claims.

### Composition

Given multiple signals for one claim:

- same-tier signals do not promote into a higher tier
- lower-tier signals do not raise a higher-tier signal
- partial signals combine by shrinking the residual claim
- signals below a required independence threshold may be demoted by mediation

## 4. Containment Properties

### Definition

A containment property is an invariant that holds regardless of worker cooperation.

### Required Properties

**Visibility boundary**

Let `M(role)` be the command and data surface visible to a given role.

For roles `worker < auditor < operator`:

    M(worker) ⊂ M(auditor) ⊂ M(operator)

The worker-visible surface must not disclose hidden criteria or higher-role capabilities through normal interaction.

**Non-escalation**

The worker cannot change its own authorization context during a task.

**Execution-time opacity**

Criteria marked hidden are not visible to the worker during execution.

**Post-adjudication transparency**

By default, criteria hidden from the worker during execution become visible in the human decision record after adjudication.

This is a core property of The Silent Critic rather than an implementation convenience.

### Side Channels

This appendix acknowledges three side-channel classes:

- behavioral
- cross-tool
- temporal

V1 is not required to eliminate them entirely. It is required to keep the intended contract surface out of direct ordinary worker view.

## 5. Task Contract Grammar

### Purpose

A task contract is the formal acceptance surface for one bounded unit of work.

The contract is declarative. It specifies what must be evaluated, what is visible during execution, and how evidence is classified. It does not encode workflow control flow.

### Terminals

```text
TASK_ID        = string
IDENTIFIER     = namespace "." name
TIER           = "decisive" | "partial" | "strong-evidence" | "weak-evidence" | "non-adjudicative"
INDEPENDENCE   = "maximum" | "high" | "moderate" | "low" | "minimum"
VISIBILITY     = "visible" | "hidden"
PARAM_NAME     = [a-z][a-z0-9_-]*
PARAM_VALUE    = string | number | boolean
MEDIATION_ID   = namespace "." name
CLAIM_TEXT     = string
```

### Contract Structure

```text
contract       = "task" TASK_ID ":" goal-clause criterion-ref+
goal-clause    = "goal:" CLAIM_TEXT

criterion-ref  = IDENTIFIER
                 base-tier-clause
                 base-independence-clause
                 visibility-clause
                 [ declared-mediation* ]
                 [ param-clause* ]
                 [ residual-clause ]

base-tier-clause         = "tier:" TIER
base-independence-clause = "independence:" INDEPENDENCE
visibility-clause        = "visibility:" VISIBILITY
declared-mediation       = "mediation:" MEDIATION_ID
param-clause             = PARAM_NAME ":" PARAM_VALUE
residual-clause          = "covers:" CLAIM_TEXT "residual:" CLAIM_TEXT
```

### Contract Invariants

For a contract to satisfy The Silent Critic model:

1. it MUST have a goal
2. it MUST contain at least one visible criterion
3. it SHOULD contain at least one hidden criterion
4. every criterion MUST have a base tier
5. every criterion MUST have a base independence
6. every criterion MUST declare execution-time visibility

If a contract has no hidden criteria, it may still be valid, but it is closer to improved CI than to a full Silent Critic task.

## 6. Mediation Rules

### Purpose

Mediations mechanically adjust a criterion's effective classification based on observed or declared context.

### Structure

```text
mediation-def  = MEDIATION_ID
                 condition-type
                 condition-spec
                 effect+

condition-type = "observable" | "declared"
effect         = tier-effect | independence-effect
tier-effect         = "tier:" ADJUSTMENT
independence-effect = "independence:" ADJUSTMENT
ADJUSTMENT          = "promote" | "demote" | "set:" TIER | "set:" INDEPENDENCE
```

### Examples

Observable mediations:

- `authorship.same-agent`
  Demote independence when the same session produced both artifact and test.
- `coverage.below-threshold`
  Demote tier when observed coverage falls below a declared threshold.
- `freshness.stale`
  Set tier to `weak-evidence` when evidence was gathered against stale content.

Declared mediations:

- `context.mock`
  Demote tier and independence for mock-backed checks.
- `context.shared-model-family`
  Demote independence for shared-model evaluation.
- `context.regulatory`
  Raise the contextual bar by demoting would-be decisive evidence.

### Effective Classification

The effective classification for a criterion is computed mechanically:

```text
1. Start with base tier and base independence
2. Apply matching observable mediations
3. Apply declared mediations
4. Record effective tier and effective independence
```

The audit record MUST preserve this derivation chain.

## 7. Contract Satisfaction Semantics

A contract is satisfied when all of the following hold:

1. every `decisive` criterion has passing evidence
2. every `partial` criterion has passing evidence and a declared residual
3. every residual is either:
   - closed by another criterion, or
   - routed to human adjudication
4. every `strong-evidence` and `weak-evidence` criterion has a recorded signal
5. hidden criteria were not visible to the worker during execution
6. post-adjudication disclosure rules have been applied to the final decision record unless an explicit exemption exists

If unresolved residuals remain and no human decision closes them, the contract is not satisfied.

## 8. Residual Uncertainty

### Definition

Residual uncertainty is the set of open claim fragments that remain after available evidence has been applied.

### Composition

Partial evidence does not promote to decisive by count. It narrows the unresolved surface.

If a criterion declares:

```text
covers:   "execution paths for modified functions"
residual: "edge cases, integration behavior"
```

then subsequent evidence applies to the residual, not the original claim as a whole.

### Routing Rule

Residuals that remain open after all available evidence must be routed into the human decision record.

Residual uncertainty is therefore both:

- a formal remainder of contract evaluation
- the primary human review surface

## 9. Decision Semantics

### Definition

A decision is a human act that changes the acceptance state of a task.

### Decision Types

The minimum decision types are:

- `accept`
- `reject`
- `accept-residual`
- `rescope`
- `insufficient-evidence`
- `waive-criterion`
- `require-rework`

### Decision Record

Each decision record should contain:

```text
decision:
  type: DECISION_TYPE
  actor: string
  timestamp: datetime
  basis: string
  evidence_refs: ref*
  resolves: claim-fragment*
  outcome: string
```

The system spec's "human decisions" section is the human-readable projection of this structure.

## 10. Decision Log Composition

### Canonical Model

A decision log is the canonical structured record for one adjudicated task.

It is not synonymous with the pull-request rendering. The PR rendering is one projection of the same canonical record.

### Required Sections

The canonical decision log contains:

1. `goal`
2. `work_performed`
3. `criteria_applied`
4. `evidence`
5. `human_decisions`
6. `residual_uncertainty`
7. `acceptance_outcome`

### Section Semantics

**goal**
The task objective in plain language.

**work_performed**
What `the-silent-critic` can attest changed.

**criteria_applied**
Both visible criteria and criteria hidden during execution, disclosed after adjudication unless explicitly exempted.

**evidence**
Signals produced by automated, agent-evaluated, or human-evaluated procedures, with effective classification.

**human_decisions**
Recorded acts of judgment that changed acceptance state.

**residual_uncertainty**
The remaining open claims, if any, and how they were handled.

**acceptance_outcome**
The final state and rationale.

## 11. Pull Request Rendering

The v1 pull-request artifact is a structured narrative rendering of the decision log.

The rendering must:

- be readable by humans
- preserve the decision order and rationale
- expose hidden criteria after adjudication by default
- link back to supporting evidence
- link back to changed commits, files, or diff regions where useful

## 12. Relationship to Diffs

The diff is not the primary acceptance record.

Formally:

- the decision log is primary
- the diff is supporting context
- claims in the decision log may carry references into the diff

Reviewer flow is therefore:

1. read the decision log
2. inspect evidence and residuals
3. descend into diffs only where needed

## 13. Summary

The formal model of The Silent Critic is built around:

- a contract with visible and hidden criteria
- mechanically classified evidence
- containment properties that do not assume worker cooperation
- residual uncertainty as the routed surface for human judgment
- post-adjudication transparency
- a canonical decision log that can be rendered into a pull request

That is the formal skeleton underlying the architectural spec.
