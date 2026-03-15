# The Silent Critic System Specification

**Status:** Working specification
**Purpose:** Normative description of The Silent Critic as a supervision system for agentic software development
**Scope:** Architecture, invariants, data model, and integration behavior for `the-silent-critic`

This document states the normative architecture. The companion formal appendix defines the exact semantics of contracts, criteria, evidence classification, containment properties, and decision-log composition.

## 1. Naming

**The Silent Critic** is the supervision framework.

`the-silent-critic` is the tool that implements the framework's trust boundary, verification surface, and decision record.

This distinction is normative throughout this document:

- **The Silent Critic** refers to the method, model, and supervision regime
- **`the-silent-critic`** refers to the executable and its persisted records

## 2. Problem Statement

The pull request is a useful social artifact and an inadequate quality mechanism.

This specification assumes:

- Pull requests remain valuable for coordination, explanation, accountability, and organizational memory
- Pull-request approval is not a reliable primary quality gate
- Software is becoming effectively free to produce
- Human attention is therefore the scarce resource
- Real engineering acceptance always includes both explicit and tacit criteria
- A supervision system that exposes the full acceptance surface to the worker invites optimization against visible checks rather than against actual intent

The Silent Critic exists to provide a stronger supervision boundary inside that environment without requiring immediate replacement of the pull request as the team's social artifact.

## 3. Core Design Goals

The system MUST satisfy the following goals:

1. Preserve the pull request as a human coordination artifact.
2. Replace diff-centered quality theatre with a stronger supervision mechanism.
3. Preserve tacit human intent as a real part of the acceptance regime.
4. Prevent the worker from seeing the full acceptance surface during execution.
5. Produce tool-authored evidence rather than relying on worker self-report.
6. Route human attention to residual uncertainty and acceptance decisions rather than routine artifact inspection.
7. Produce a shareable decision record that can be socialized through the pull request.
8. Support v1 operation inside an existing pull-request workflow.

## 4. Non-Goals

The system does not initially attempt to:

- Replace the pull request as the primary socialization surface
- Eliminate diffs from engineering workflows
- Prove program correctness in the formal sense
- Remove humans from the supervision loop
- Define a permanent boundary between machine-capable and human-only work
- Solve organization-wide orchestration beyond the scope needed to support the supervision boundary

## 5. Architectural Thesis

The Silent Critic is built on four claims:

1. **Acceptance exceeds explicit requirements.**
   Every non-trivial engineering task has explicit criteria and tacit criteria.

2. **Workers optimize against what they can see.**
   Any worker, human or machine, will tend to satisfy the visible regime if the visible regime diverges from actual intent.

3. **Supervision must not rely on worker cooperation.**
   The system's core guarantees must hold even if the worker is mistaken, optimizing narrowly, omitting context, or strategically satisfying visible checks.

4. **The decision record must remain social.**
   Hidden criteria may be hidden during execution, but the resulting acceptance record must be visible to humans after adjudication.

## 6. System Model

The Silent Critic models software work as bounded tasks subject to supervision.

Each task has:

- a stated goal
- a worker-visible contract surface
- a worker-hidden contract surface
- a verification record
- a set of human decisions
- a final decision log

The system has four operational phases:

1. **Contract composition**
   The acceptance surface for the task is defined.

2. **Execution**
   The worker performs the task against the visible contract surface.

3. **Adjudication**
   The system evaluates evidence, exposes hidden criteria to authorized humans, and routes residual uncertainty for human judgment.

4. **Publication**
   The decision record is exported into the pull request or equivalent social surface.

## 7. Trust Boundary

`the-silent-critic` is the trust boundary.

This means:

- The worker does not hold the full task contract.
- The worker does not author the structured quality claim.
- The worker does not control which criteria are binding.
- The worker does not determine which evidence is sufficient for acceptance.

Instead, `the-silent-critic` is responsible for:

- storing the task contract
- separating visible and hidden criteria
- observing the world state relevant to the task
- recording attested evidence
- classifying unresolved uncertainty
- recording human decisions
- publishing the final decision log

Any design that places these responsibilities primarily in the worker prompt, worker memory, or worker-authored output is out of scope for The Silent Critic.

## 8. Roles

The minimum role model is:

- **Worker**
  Performs the task against the visible contract surface.

- **Auditor**
  Can inspect the full contract and the evidence record, including criteria hidden from the worker.

- **Operator**
  Composes contracts, approves hidden criteria, adjudicates unresolved uncertainty, and accepts or rejects outcomes.

These roles may be implemented by humans, agents, or mixed human-agent procedures, but the visibility rules in this specification are normative regardless of implementation.

## 9. Execution-Time Visibility Rules

The following invariants apply during execution:

1. The worker MUST see the task goal and all criteria marked visible.
2. The worker MUST NOT see criteria marked hidden.
3. The worker MUST NOT be told whether zero, one, or many hidden criteria exist except insofar as that fact is unavoidable from the execution protocol.
4. The worker MUST NOT be able to alter its own authorization level or reveal hidden criteria by normal tool usage.
5. The worker MUST NOT be the authoritative source of the acceptance record.

This specification does not require perfect elimination of all side channels in v1. It does require that the system's intended acceptance boundary not be directly inspectable through ordinary worker interaction with `the-silent-critic`.

## 10. Post-Adjudication Transparency

The Silent Critic is not permitted to become a private oracle.

The following invariant is mandatory by default:

**Hidden during execution, visible after adjudication.**

This means:

- Criteria hidden from the worker during task execution MUST be disclosed to humans after adjudication by default
- The decision record MUST expose both visible and formerly hidden criteria by default
- The pull request MUST be able to carry the resulting acceptance record by default

Exceptions MAY exist for security-sensitive or otherwise restricted criteria, but such exceptions are deviations from the default and MUST be explicit.

The system MUST optimize for human buy-in, inspectability, and discussion after the task is complete.

## 11. Task Contract

Each task MUST have a contract.

At minimum, the contract contains:

- task identifier
- plain-language goal
- visible criteria
- hidden criteria
- verification procedures associated with criteria
- authority needed for final acceptance

The contract may be represented internally using richer formal machinery, including criterion libraries, mediation rules, epistemic tiers, and residual definitions. Those mechanisms are part of the formal model, but the existence of a contract is the primary architectural requirement.

The formal appendix defines:

- the contract grammar
- criterion visibility semantics
- independence and epistemic-tier semantics
- mediation and residual composition
- post-adjudication disclosure rules
- decision-log composition rules

The following invariant applies:

**A task without hidden criteria is not yet a full Silent Critic task.**

Such a task may still be useful, but it is closer to improved CI than to The Silent Critic's intended supervision model.

## 12. Verification and Attestation

The Silent Critic distinguishes between worker narration and tool-authored evidence.

### 12.1 Worker Narration

Any worker-authored statement about what changed, why it changed, or why it should be considered correct is non-authoritative context. It MAY be useful. It MUST NOT be the decisive basis for acceptance.

### 12.2 Tool-Authored Evidence

`the-silent-critic` MUST be able to record evidence it authors or attests to directly. This includes, at minimum:

- observed files or other changed artifacts
- source-control state sufficient to connect the work to a change set
- results of verification procedures
- the binding between evidence and task criteria

The exact implementation mechanism may vary, but the system MUST preserve the distinction between:

- what the worker says happened
- what `the-silent-critic` can attest happened

### 12.3 Verification Procedures

Verification procedures MAY be:

- automated
- agent-evaluated
- human-evaluated

The system SHOULD prefer stronger and more independent evidence where feasible, but this specification does not require a single implementation strategy for every criterion class.

## 13. Residual Uncertainty

The primary human review surface is residual uncertainty.

This means:

- The system SHOULD compress the supervision record into what remains unresolved
- Human attention SHOULD be routed to uncertainty, ambiguity, scope decisions, architectural concerns, and risk acceptance
- Humans SHOULD NOT be asked to begin by reconstructing the meaning of the work from raw diffs alone

Residual uncertainty MAY include:

- criteria only partially closed by available evidence
- conflicts between signals
- unverified blast-radius concerns
- ambiguous alignment with tacit intent
- changes requiring explicit risk acceptance

## 14. Human Decisions

The Silent Critic requires an explicit decision record for human judgment.

A **decision** is any human act that materially changes acceptance state, including:

- accepting or rejecting a task result
- accepting residual uncertainty
- re-scoping the task
- declaring evidence insufficient
- waiving a criterion
- adding or revising criteria for rework

Each recorded decision SHOULD include:

- who made the decision
- when it was made
- what evidence informed it
- what uncertainty or conflict it resolved
- what outcome followed from it

Human judgment is not an embarrassment in this system. It is a first-class part of the acceptance record.

## 15. Decision Log

Every adjudicated task MUST produce a decision log.

The decision log is the primary human-facing artifact generated by The Silent Critic.

The decision log has both a canonical structured form and one or more renderings. The pull-request export is the initial rendering, not the canonical record.

At minimum, the decision log MUST contain:

- **Goal**
  What the task was intended to accomplish.

- **Work Performed**
  What `the-silent-critic` can attest changed.

- **Criteria Applied**
  The criteria used to evaluate the task, including criteria hidden from the worker during execution unless explicitly exempted.

- **Evidence**
  What verification was run and what it showed.

- **Human Decisions**
  What judgment calls were made and why.

- **Residual Uncertainty**
  What remained unresolved and how it was handled.

- **Acceptance Outcome**
  The final acceptance state and rationale.

The decision log SHOULD be structured, concise by default, and expandable into detail.

## 16. Pull Request Export

The pull request remains the primary socialization surface in v1.

Therefore, `the-silent-critic` MUST support export of the decision log into a form that can be attached to, linked from, or embedded in a pull request.

The exported artifact MUST be:

- human-readable
- durable enough for later inspection
- attributable to a specific task and change set
- suitable for discussion among humans

The export SHOULD read as a structured narrative rather than a raw machine dump.

The default v1 sections are:

- Goal
- Work Performed
- Criteria Applied
- Evidence
- Human Decisions
- Residual Uncertainty
- Acceptance Outcome

## 17. Relationship to Diffs

The diff remains relevant, but it is demoted.

The following rules apply:

1. The diff MUST remain addressable from the decision log.
2. The decision log MUST be readable without beginning from the diff.
3. Major claims in the decision log SHOULD point back to relevant commits, files, or diff regions where useful.
4. The diff is supporting context, not the primary acceptance record.

This is a transitional requirement. The system must work in a world still organized around diffs without treating the diff as sovereign.

## 18. V1 Deployment Model

The v1 deployment target is insertion into an existing pull-request workflow.

This means:

- pull requests continue to exist
- branch protections may continue to exist
- existing review rituals may continue to exist
- `the-silent-critic` operates initially in an advisory or augmenting role

The purpose of v1 is not to win institutional permission to replace the pull request. The purpose of v1 is to produce a stronger quality signal inside the existing workflow and make the contrast legible in practice.

## 19. Humane Objective

The Silent Critic is not merely a containment architecture. It is also an attempt to preserve a humane software discipline under conditions of abundant machine labor.

The system SHOULD support:

- removal of low-authority routine work from the critical human path
- concentration of human attention on judgment, intent, and responsibility
- preservation of apprenticeship and growth paths for less experienced engineers
- reduction of senior-engineer gatekeeping as a default control surface

The intended outcome is not fewer humans. It is better use of human attention.

## 20. Deferred Interfaces

This specification requires a shareable decision record. It does not require that the pull request remain the final or best interface for that record.

Future interfaces MAY include:

- alternative change-management UIs
- decision visualizations
- evidence graphs
- uncertainty navigation tools

However, the structured decision record is the canonical model. PR text is the initial rendering, not the conceptual model.

## 21. Open Questions

The following questions remain open at the time of this specification:

- What is the minimum useful hidden-criteria set for v1?
- How should task contracts be authored and revised in practice?
- What side-channel risks are acceptable in the initial implementation?
- Which portions of the decision log should be generated automatically versus curated by humans?
- What is the right balance between compact PR export and deeper linked artifacts?

## 22. Summary

The Silent Critic is a supervision system for software work under conditions of effectively free code generation.

Its defining properties are:

- a trust boundary external to the worker
- a task contract containing visible and hidden criteria
- tool-authored evidence and attestation
- human review centered on residual uncertainty
- post-adjudication transparency
- a shareable decision record
- compatibility with existing pull-request workflows

If those properties are absent, the system may still be useful, but it is not yet The Silent Critic in the sense intended by this specification.
