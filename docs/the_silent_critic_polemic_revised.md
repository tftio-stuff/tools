# Your Code Review Process Is Theatre

## This Ritual Is Rotten

Your code review process is rotten, and deep down everyone involved knows it.

Not because engineers are lazy; not because managers are stupid. Not because GitHub is evil, and not because Capitalism. It is rotten because it is a path-dependent morass of inherited ritual, orthogonal compliance incentives, and institutional self-deception that has been mistaken for rigour for so long that we as an industry can no longer tell the difference.

You open the PR. You skim the diff. You leave a comment so there is visible evidence that you were there. You click Approve. Everyone keeps moving. The green checkmark appears. The ritual is complete. Nothing meaningful has been established except that the ceremony was performed.

And still we go on pretending. We pretend the approval means understanding. We pretend the queue means discipline. We pretend the branch protection rule means quality control. We pretend that because the process is tedious, it must therefore be serious.

That pretence was *already* corrosive when humans were the only ones producing code. In the presence of LLM agents, in a world where software is becoming effectively free to produce, it is catastrophic. A workflow that already produces weak assurance at human speed will collapse under machine speed. It will drown in output, routinize false confidence, and consume exactly the attention that should have been reserved for actual supervision.

Pull requests are not the only failure in software engineering. They are not even the deepest one. But they may be the most critical one, because they are load-bearing. They sit at the junction of quality claims, team coordination, compliance posture, managerial oversight, and release process. And they are not fit for purpose. They have never been fit for purpose as a code-quality mechanism. We built too much of the institution on top of them anyway. Now the bill is coming due.

At this point you may think I am arguing for nihilism. You may think I am saying code review does not matter, that standards do not matter, that software quality is fake, that everyone should just merge to `main` and let God sort it out. That there’s some exogenous force that will discipline us back into producing high quality software if we all just surrender to YOLO.

**I am not arguing that.**

Code matters. Code review matters. Code quality matters. Shared ownership matters. Social cohesion, the act of working together with each other to pull the groaning machine closer to perfection — it *all* matters. I am arguing against a specific thing: the mandatory, approval-centric pull-request workflow that many internal software teams now call "code review." The one where every change needs a green checkmark from another engineer before it can merge. The one that is defended as rigour, measured as compliance, and experienced by almost everyone involved as kabuki.

That workflow is not failing accidentally. It is failing predictably; in a sense, it is failing *by design*. It was never designed to bear the epistemic weight we now put on it. It was adopted because it was available, legible, and easy to institutionalize. Then we promoted it from contribution workflow to quality regime and started acting as though the upgrade had happened on purpose.

It asks a human being to infer correctness, safety, intent, and system impact from a textual diff, usually under time pressure, usually with partial context, usually after the important decisions have already been made. Then it records whatever happened as proof that review occurred. The artifact is legible. The quality signal is not.

The result is a process optimized for audit-ability rather than understanding. It produces a trace somebody can point to later. It does not reliably produce confidence.

And because it consumes the time and authority of senior engineers while providing much less verification than it advertises, it does not merely fail to protect code quality. In many organizations, it actively crowds out the work that would.

I am angry about this because I have spent more than three decades writing software for money. I know how much of a miracle this craft can be. I was here before the 486. Before Fast Ethernet. Before C++ was anything more than elaborate m4 macros draped over truly dire C compilers. I have watched computing become one of the strangest and most powerful creative media human beings have ever stumbled into: a place where people can build worlds made of abstraction, precision, logic, play, and structure, and make those worlds do things that would be unthinkable in the world of atoms.

And we are shitting all over our bed.

We are degrading this astonishing human medium because the economic incentives are bad, because the institutions are lazy, because inherited rituals are easier to preserve than better ones are to invent, and because legible process is easier to sell than real rigour is to build. What enrages me about the current review regime is not merely that it is inefficient. It is that it is a path-dependent morass of bad incentives and dead habit that is being stress-tested to failure by a new abundance of code it was never designed to govern.

Software development was never pure. Its standards were already compromised, its incentives were already warped, and its rituals were already doing damage long before agents arrived. I am not mourning the loss of a golden age. I am railing against the dying of an already dim light. I want the direction to change while there is still something worth saving.

We are not owed permanence. We are dust in the long run, and so is every system we build. Software engineering will not escape drift, decay, or eventual ruin any more than anything else does -- thermodynamics ensures as much. But that is not an argument for cynicism. It is an argument for stewardship. All we can hope to do is make things better while we are here: more honest, more rigorous, more humane, less stupidly wasteful, less captive to bad incentives and inherited ritual. I am angry, but I am not hopeless. Hope is precisely what makes me want to intervene.

## The Approval Is the Product

If you want to understand what a process is for, ignore what people say about it and look at what the process reliably produces.

Your PR workflow does not reliably produce deep understanding of the code. It does not reliably produce defect detection. It does not reliably produce architectural coherence. What it reliably produces is an approval record.

That approval record is useful to managers, auditors, procurement questionnaires, compliance frameworks, and engineering dashboards. It is much less useful to the people deciding whether a change is safe to run in production.

The checkmark is legible. That is the point. Scott's phrase was that modern power tries to "make a society legible."[^scott] Your organization is doing the same thing to engineering work. The checkmark is easy to count, easy to require, easy to display in a control report, and easy to describe in a sales conversation. "We have a rigorous review process" is a simpler sentence than "we have a layered verification regime with uneven human escalation." The first sentence fits neatly into governance. The second requires you to know what your process is actually doing.

This is why so many teams drift toward the same equilibrium.

- Thorough review is expensive.
- Superficial review is cheap.
- Blocking a PR creates friction.
- Approving a PR clears the queue.
- Almost nobody measures review quality directly.
- Almost everybody measures whether review happened.

Under those conditions, rubber-stamping is not a moral failure. It is the rational response to a badly designed system.

Engineers are not stupid. They know the difference between a process that generates confidence and a process that generates paperwork. When you force them through the second kind while describing it as the first, they adapt accordingly. They optimize for the visible output. They leave a comment, skim the diff, click Approve, and move on to work that might actually change the system's behavior.

If that description feels unfair, ask a narrower question: when was the last time your review process distinguished, in a measurable way, between "someone looked at this" and "someone understood the risk of this change"? Most teams cannot answer. The workflow does not even try. The GitHub pull-request workflow cannot distinguish those cases. It is fundamentally incapable of doing so.

## The Diff Is the Wrong Interface

A diff is a useful artifact. It is not a sufficient epistemic instrument.

It tells you what text changed. It does not directly tell you whether the change is correct, whether the tests are meaningful, whether the behaviour matches the requirement, whether the surrounding assumptions still hold, or whether the new abstraction will rot the architecture six months from now.

Yet the standard PR workflow asks reviewers to answer exactly those questions from the diff view.

That is the category mistake at the centre of the whole ritual.

You are asking people to reconstruct semantics from patch text. Sometimes they can. For small, local, familiar changes, diff review can be useful. A tiny bug fix in well-known code. A narrow refactor with strong tests. A risky migration reviewed by the one person who actually owns the subsystem. Those are real cases.

But that is not what the process is built around. The process is built around universality. Everything becomes a PR. Every PR needs approval. The same ceremony is applied to the trivial change, the architectural change, the generated change, the dependency update, and the AI-generated thousand-line slab. The workflow has one hammer and an unlimited appetite for nails.

That is why the conversation around review quality is so often unserious. People keep treating diligence as the missing variable when the deeper problem is mismatch. A better reviewer staring harder at the wrong artifact is still staring at the wrong artifact.

## You Are Using a Stranger-Danger Workflow on Your Own Team

The modern PR workflow inherits assumptions from a world where code arrived from people you did not know and could not trust.

That made sense. A maintainer reviewing a patch from a stranger is doing gatekeeping in the literal sense. The patch is a request for admission into a shared codebase. The review is adversarial because it has to be.

But that is not the context for many internal teams.

You are using the same workflow for Dana, who has owned this service for two years, for a new hire changing copy text, for a generated dependency update, and for an AI agent that produced two thousand lines in minutes. These are not the same problem. They should not receive the same default treatment. Yet the platform presents them as variations on one ritual: inspect the diff, leave remarks if needed, approve the patch.

For trusted teammates, mandatory approval often degenerates into symbolic supervision. For genuinely risky changes, it is nowhere near enough.

That is the perverse middle state many teams now inhabit: too ceremonial to be efficient, too shallow to be reliable.

## What This Process Actually Crowds Out

Defenders of the status quo usually point to the secondary benefits of review: knowledge sharing, mentoring, shared ownership, architectural consistency.

Those benefits are *real*. They are, in fact critical. And they are weak arguments for the current ritual, because the ritual is a fabulously poor delivery mechanism for all of them.

Knowledge sharing is better served by concise change summaries, design notes, demos, and ownership rotation than by forcing teammates to reverse-engineer intent from red and green line noise.

Mentoring is better served by explicit teaching and pairing than by drive-by comments on variable names.

Architectural coherence is better served by design review before implementation than by post hoc objections once the author has already written the code and the schedule is already loaded into the branch.

Shared ownership is better served by operating systems together, rotating on-call, and maintaining documentation than by pretending that approval implies comprehension.

Meanwhile, the current process burns exactly the resource you can least afford to waste: informed technical attention.

While senior engineers are skimming diffs to satisfy branch protection, they are not clarifying invariants, tightening interfaces, deleting bad abstractions, improving tests, or specifying the boundaries that would let machines verify more and humans guess less. The ceremony consumes the people who might otherwise improve the system.

That is why calling the process harmless bureaucracy understates the problem. It is not just overhead. It is displacement.

## How We Ended Up Here

Part of the durability of this workflow comes from the fact that it did not begin as a theory of good internal engineering. It is an inheritance.

Start with `diff`. Bell Labs described it in 1976 with almost brutal clarity: it reports file differences as "a minimal list of line changes" required to bring one file into agreement with another.[^diff] That is a beautiful description of what `diff` is, and an equally beautiful description of what it is not. It is a textual change description. It is not a semantic model of program behaviour.

Then `patch`: software whose own manual defines it as a tool to "apply a diff file to an original" and to take a "difference listing produced by the diff program" and apply it to source files.[^patch] Again: a precise solution to a real problem. If code changes are moving around as textual deltas, you need a reliable way to apply textual deltas.

Then move into the world of emailed patches, maintainers, and upstreams. Git still carries this lineage in its own `git request-pull` command, which is defined as a way to "generate a request asking your upstream project to pull changes into their tree."[^request-pull] The Linux kernel documentation still describes pull requests in exactly this idiom: signed tags, diffstats, shortlogs, explanatory messages, and an emailed `[GIT PULL]` request to the maintainer.[^kernel-prs] In that world the patch is not just a code artifact. It is a trust boundary.

Then GitHub takes this model, wraps it in a web interface, and normalizes it for everyone. Its own 2010 announcement says GitHub "launched with a simple pull request system on day one" and described pull requests as "our take on code review."[^github-prs] A workflow shaped by patch transport and stranger-trust negotiation becomes the default interface for every software team, including small internal teams working in a single repository on a single product.

That is a remarkable historical sleight of hand. A workflow designed around text transport, repository gating, and anonymous contribution gets rebranded as the universal default for software quality.

At no point in that lineage was the central question, "What is the best way for a trusted internal team to develop, verify, and govern changes to a complex software system?" The tools evolved to solve adjacent problems. File comparison. Patch distribution. Gatekeeping at scale. Legible contribution history.

And because the resulting workflow was available, convenient, and bundled with the platform, it became "best practice" long before most organizations had reasoned clearly about whether it was actually best.

This is the heart of the indictment: as a code review and quality mechanism, the pull request was never fit for purpose. It was useful for other things. It solved other problems. Then institutions treated its ubiquity as proof of adequacy and built release, compliance, and management structures on top of it. That mistake is about to become extremely expensive.

This is how you get engineering cargo cults. A toolchain with a real history and a real purpose outlives the conditions that made it sensible, but the ritual remains. The forms are preserved. The explanation is lost. The surviving structure is mistaken for timeless wisdom. Eventually, the cargo planes will return.

## The Machines Are Better at the Parts You Pretend Humans Are Doing

There was a time when diff inspection was one of the best tools available. That time is gone.

You now have type systems, static analysis, property-based testing, contract tests, coverage analysis, linters, schema validation, fuzzing, symbolic execution in some domains, increasingly capable code reasoning tools, and CI infrastructure that can run far more verification than most teams bother to wire together.

And yet the decisive gate in many organizations is still a human approval on a web page.

That should be embarrassing.

Humans are good at intent, tradeoffs, product judgment, domain modelling, and spotting when the whole shape of a change feels wrong. Machines are good at exhaustive checking, consistency, speed, and *repeatability*.

The standard PR process assigns too much machine-friendly work to humans and then congratulates itself for keeping a human in the loop.

If your review policy can be satisfied by somebody glancing at a diff on her phone between meetings, then what you have built is not a quality gate. It is a waiting room.

## What Comes After the Ritual

This is the point where people who like the current ritual become theatrically relieved and say, fine, so what is your alternative? Here it is.

The alternative is not no review. It is a different supervision interface.

It is also not the abolition of the pull request. The pull request remains useful as a social artifact: a place for visibility, coordination, accountability, and shared memory. What has to go is the fiction that pull-request approval is the primary quality gate.

This matters because the task is no longer to sand down the edges of the pull-request workflow. The task is to get out of the box it trapped us in. Once a load-bearing institution is not fit for purpose, incremental piety is not a strategy. You need an exit.

A serious engineering organization would separate concerns instead of collapsing them into one ritual. It would move design review earlier, before code exists and before authors become attached to local implementation details.

It would require authors and tools to explain the change in plain language: what changed, why it changed, what was verified mechanically, what remains uncertain, and what operational risks still exist.

It would invest in mechanical verification that produces signals with known meaning: type checks, invariant checks, static analysis, targeted tests, regression detection, interface validation, and explicit classification of what evidence does and does not exist.

It would classify those signals by epistemic authority and independence instead of flattening them all into the same merge ritual. Some checks close claims decisively. Some close them only partially. Some are routing signals. Some verify nothing at all. A serious system would know the difference and make that difference visible.

It would reserve human review for the places where human judgment is actually irreplaceable: architecture, product semantics, risk acceptance, domain intent, suspicious changes flagged by the evidence, and residual uncertainty machines cannot close.

It would stop pretending that every change deserves the same review shape. Small trusted changes with strong verification should flow differently from high-risk migrations, generated code, security-sensitive paths, or giant machine-produced patches. A process that cannot distinguish among these cases is not rigorous. It is merely uniform.

Most importantly, it would stop presenting the raw diff as the primary interface to supervision. The human should not be asked, first and by default, to inspect artifacts. She should be shown the residual uncertainty after machine verification has done everything it can. The question is not "please read these files." The question is "here are the claims we could not close mechanically; which of them require your judgment?"

That is the system I am arguing for. Review becomes uncertainty adjudication, not textual archaeology. The supervised object becomes a bounded unit of work with explicit verification claims, not a pile of changed lines. The diff remains available, just as the assembly listing remains available. It is no longer mistaken for the primary instrument of assurance. The resulting decision record can then be socialized through the pull request instead of reverse-engineered from it.

## Enough With the Checkmark

The green checkmark survives because it is legible. It is easy to require, easy to count, easy to explain, and easy to defend.

It is also a dangerous substitute for thought.

Across the industry, teams are spending enormous amounts of skilled attention performing a ceremony whose output is clearer than its value. They call this discipline. They call this review culture. They call this quality control.

Very often, it is none of those things.

It is a theatre of assurance built around a thin artifact, a weak signal, and an approval record that flatters the organization more than it protects the code.

At its worst, it becomes cargo cult engineering: branch protections, reviewer counts, template checkboxes, and approval gates layered into an ornate ritual whose visible structure is mistaken for real control. The tower is elaborate. The planes still do not land.

If your process cannot tell the difference between code that was understood and code that was merely approved, then your process does not know what it is claiming to know.

Stop defending that as rigour.

Stop forcing engineers to act out confidence they do not have.

Stop calling a compliance-friendly queue-management ritual "code review" and then wondering why it produces so little review.

I am not arguing for collapse. I am arguing against it. I want the craft of software engineering to become better than it has been, even under contact with code generation that can outpace human review by orders of magnitude. I want the scarce attention of good engineers spent where it protects systems rather than where it decorates process. I want us to build better forms of supervision before the current ones fail under load. Nothing will save us automatically. If this gets better, it will get better because people decide to build something better together.

And I do not mean just people like me. I do not want only the battle-hardened principal engineer to get leverage from better tools and better supervision. I want everyone who wants to play to be able to play. I want more people to be able to build serious things, understand what they are doing, and participate in this absurd, beautiful, abstract craft without being crushed by bad process or locked out by institutional sludge.

If you think this means I am arguing for less rigour, you have completely missed my point. I am arguing for a supervision system stricter than PR theatre and more honest than approval rituals. I call it The Silent Critic.

[^scott]: James C. Scott, *Seeing Like a State: How Certain Schemes to Improve the Human Condition Have Failed* (Yale University Press, 1998). In the book's introduction, Scott describes the state's effort to "make a society legible." An online excerpt reproducing that passage is available via The Anarchist Library.

[^diff]: J. W. Hunt and M. D. McIlroy, "An Algorithm for Differential File Comparison," Bell Laboratories Computing Science Technical Report #41, July 1976, [cs.dartmouth.edu/~doug/diff.pdf](https://www.cs.dartmouth.edu/~doug/diff.pdf).

[^patch]: `patch(1)` man page, Debian Bookworm edition, [manpages.debian.org/bookworm/patch/patch.1.en.html](https://manpages.debian.org/bookworm/patch/patch.1.en.html).

[^request-pull]: `git-request-pull(1)` manual page, kernel.org, [kernel.org/pub/software/scm/git/docs/git-request-pull.html](https://www.kernel.org/pub/software/scm/git/docs/git-request-pull.html).

[^kernel-prs]: "Creating Pull Requests," *The Linux Kernel documentation*, [docs.kernel.org/maintainer/pull-requests.html](https://docs.kernel.org/maintainer/pull-requests.html).

[^github-prs]: Ryan Tomayko, "Pull Requests 2.0," *The GitHub Blog*, August 31, 2010, [github.blog/2010-08-31-pull-requests-2-0](https://github.blog/2010-08-31-pull-requests-2-0/).
