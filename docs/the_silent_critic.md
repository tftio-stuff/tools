# The Silent Critic

**The Silent Critic** is a supervision framework for software development in a world where software is becoming effectively free to produce and human attention is the scarce resource.

It begins from a simple observation: the pull request remains a useful social artifact, but it was never a serious primary quality mechanism. Pull requests are good for coordination, explanation, accountability, and shared memory. They are not good enough, by themselves, to serve as the main supervision boundary for abundant machine-generated software.

That mismatch is now impossible to ignore. When software output becomes cheap, the old review ritual breaks down. Humans cannot keep pace by reading more diffs. The bottleneck is no longer producing code. It is deciding which changes are acceptable, which uncertainties matter, and where human judgment is actually required.

The Silent Critic responds by changing the object of review.

Instead of treating the diff as the primary epistemic surface, it treats software work as a bounded unit of change with an acceptance surface. That acceptance surface includes both explicit criteria and tacit intent. The worker is evaluated against that fuller acceptance regime, not merely against the visible checklist.

This leads to two practical consequences.

First, the worker cannot be shown the whole acceptance surface during execution. If the visible regime is narrower than actual intent, any worker will optimize against what it can see. A serious supervision system has to account for that reality rather than pretending it does not exist.

Second, human review moves upward. Humans are not asked, by default, to reconstruct meaning from raw diffs. They are asked to review residual uncertainty: what the available verification could not close, what judgment still matters, and why the work is acceptable anyway.

The goal is not to abolish the pull request. The goal is to stop asking it to do a job it was never fit to do. The pull request remains the social artifact. The quality claim moves to a stronger supervision substrate. The resulting decision record can then be socialized through the pull request rather than reconstructed from it.

That decision record is part of the framework. The Silent Critic should produce a shareable account of:

- the goal of the change
- the criteria that were applied
- the evidence that was gathered
- the human decisions that were made
- the uncertainty that remained
- the final acceptance outcome

The deeper ambition is not merely procedural. The Silent Critic is an attempt to preserve a humane software discipline under conditions of abundant machine labor. If routine implementation and routine checking become cheap, then human attention can be spent where it matters most: intent, architecture, uncertainty, responsibility, and the slow work by which junior engineers become senior ones.

This document is the shortest statement of the framework.

For the critique of the current process, read [the_silent_critic_polemic_revised.md](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_polemic_revised.md) and [the-silent-critic-argument-memo.md](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic-argument-memo.md).

For the formal model and implementation details, read [the-silent-critic-system-spec.md](/Users/jfb/Projects/tools/feature/gator/docs/the-silent-critic-system-spec.md), [the_silent_critic_formal_appendix.md](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_formal_appendix.md), and [the_silent_critic_tooling_design.md](/Users/jfb/Projects/tools/feature/gator/docs/the_silent_critic_tooling_design.md).
