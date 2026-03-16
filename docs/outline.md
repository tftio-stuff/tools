## The Silent Critic: Argument Memo Outline

### 1. Opening: scarce human attention is the bottleneck

- State the fundamental resource constraint directly.
- Software generation is becoming abundant; human attention is not.
- The central problem in software development is increasingly not producing changes, but deciding what deserves real human judgment.
- The memo is about how to budget that attention well.

### 2. Automated systems are here to stay

- Do not frame this as a speculative future.
- Automated systems are already part of software development and will remain so.
- The practical question is not whether to trust them in the abstract, but how to build justified trust in them without wasting human cognition.
- That is the supervision problem.

### 3. Existing review practices spend attention badly

- Current code review practices are weak not because people are lazy, but because they ask humans to attend to the wrong object.
- Diff-centered review can help in narrow cases, but it is a poor general-purpose mechanism for evaluating correctness, safety, or alignment with intent.
- This problem predates agents; automation only makes it impossible to ignore.
- The failure is fundamentally attentional and epistemic, not merely procedural.

### 4. The real question: what should a human actually have to decide?

- Reframe review as an attentional-allocation problem.
- The goal is not that a human should "look at the change."
- The goal is that a human should intervene where their judgment is actually load-bearing.
- That means distinguishing between what can be closed mechanically and what still requires human adjudication.

### 5. Tacit intent and epistemic uncertainty

- Every real engineering task has explicit criteria and tacit criteria.
- Acceptance is never exhausted by the written brief.
- This is true regardless of whether the worker is a human, a friend, Jane, a contractor, or an agent.
- Therefore a serious supervision regime must expose epistemic uncertainty:
  what is known, what is verified, what remains unresolved, and why.

### 6. Core thesis: humans should review uncertainty, not artifacts

- The primary interface to review should not be a raw artifact like a diff.
- The human should be shown unresolved claims, residual uncertainty, verification gaps, blast radius, and decision points.
- Supporting artifacts remain available, but they are not the first-order epistemic surface.
- Review becomes uncertainty adjudication rather than textual archaeology.

### 7. The Silent Critic as the proposed answer

- Introduce The Silent Critic as a supervision framework for budgeting human attention and building justified trust in automated work.
- It formalizes the acceptance surface of a bounded unit of work.
- It separates what can be verified mechanically from what still requires judgment.
- It produces a decision record instead of asking reviewers to reconstruct one from artifacts.

### 8. Why hidden criteria matter

- Not because every worker is hostile.
- Because any worker optimizes against the regime it can see.
- Tacit intent is real, so visible checks cannot be assumed to exhaust what matters.
- Hidden criteria preserve the real acceptance surface during execution.
- After adjudication, those criteria and the resulting decisions can be disclosed to humans.

### 9. Trust, not theatre

- The point is not to preserve a ritual of oversight.
- The point is to create a stronger basis for justified trust in automated systems.
- Trust should come from explicit evidence, bounded uncertainty, and recorded human decisions.
- A system that merely produces approval signals without showing what remains uncertain does not deserve trust.

### 10. Human consequences

- Senior engineers should not be default artifact gatekeepers.
- Their attention should go to architecture, uncertainty, risk, and intent.
- Juniors should not be displaced by agents and replaced by senior supervisory clergy.
- A better system preserves an apprenticeship path by moving human learning upward, not by removing humans from the process.

### 11. Humane ambition

- This is not about protecting a mystical human remainder.
- It is about removing the work robots can do from the critical human path.
- The goal is more space for human judgment, creativity, responsibility, and craft.
- The larger hope is a richer, more humane computing culture under abundance.

### 12. Integration surfaces come later

- Existing workflow artifacts, including pull requests, still matter.
- They can carry the resulting decision record and remain useful social artifacts.
- But they are not the center of the argument.
- The center of the argument is attentional budgeting and justified trust.

## Compression Advice

- Sections 1-2 establish the world.
- Sections 3-6 are the conceptual core.
- Sections 7-9 are the framework claim.
- Sections 10-11 are the human stakes.
- Section 12 should be short and clearly secondary.
