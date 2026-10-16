---
name: ai-sdlc-1-intent
description: AI-native SDLC Stage 1 (Plan) - capture an idea, ticket or incident as a committed intent.md (problem, proposed outcome, affected users and systems, constraints, out of scope, open questions) by brainstorming with the originator. Use when the user says "stage 1", "capture intent", "write an intent.md", "I have an idea for…", or brings a new feature request, ticket or incident that has no intent yet.
---

# Stage 1 - Plan: capture intent as `intent.md`

## Goal

Turn an idea into a short, version-controlled **proto-spec** that the originator wrote in
their own words, with Claude's help. It records **what** is wanted, **why**, and **under which
constraints**. It does **not** record how to build it. This replaces the backlog → user story →
refinement handoffs, where meaning drifts at each step.

## Entry routes

- A person's idea (the most common case)
- A filed ticket or bug report
- An incident or metric breach from Stage 6 (Claude writes the intent itself, see the
  "Agent-generated intents" section below)

The same review steps apply to all of them.

## Entry criteria

- None. The originator doesn't need technical skills.

## Steps

1. **Let the originator describe the problem in their own words.** Ask them for:
   - what they can't do today, or what is painful
   - who is affected
   - what "better" looks like
   - what is explicitly out of scope
2. **Brainstorm until the idea is concrete.** Ask questions the way an analyst would, a few at
   a time (use `AskUserQuestion` for choices), about:
   - scope: what is in, what is out, and what is a later version
   - users: who they are, how many, and their context
   - systems: what this touches, replaces or feeds
   - constraints: data and privacy, platform, compliance, budget, time, technology that is
     fixed or deliberately left open
   - success: how the originator will know it worked (observable, ideally measurable)

   Stop asking when you could explain the problem and the outcome to a stranger. Questions that
   can't be answered yet become **open questions**. Don't block on them.
3. **Write `intent.md`** using [template.md](template.md). Rules:
   - Stay in the originator's voice and terms. Use first person if that is how they talk.
   - Stay at the level of *what and why*. Don't choose technology, architecture or data
     models unless the originator imposes it as a constraint. Write "deliberately left open
     for design" instead.
   - Constraints must be checkable. Stage 2 maps each one to how it is met.
   - Out of scope is a list of concrete items, not "everything else".
   - Open questions are real decisions the spec must answer, phrased as questions, with the
     current assumption if there is one.
4. **The originator corrects misunderstandings.** Show the draft and ask explicitly what is
   wrong or missing. Iterate.
5. **Set `Status: proposed`** and save it in the change folder (see `ai-sdlc`, "Change folder
   convention"). Commit it if the repo's git policy allows, otherwise propose the commit. Git
   records the author and timestamp.
6. **The product owner decides.** Accepting the intent (merge, or `Status: approved`) moves it
   to Stage 2. Rejecting it (a closed PR, or `Status: rejected` with a reason) is a valid outcome
   too. If the originator is also the PO, ask for the decision explicitly anyway.

## Exit criteria

- [ ] `intent.md` exists in the change folder and follows the template
- [ ] The originator confirmed it reflects what they mean
- [ ] Every constraint is concrete enough to verify later
- [ ] Out of scope and open questions are written down
- [ ] It contains no implementation decisions that the originator didn't impose
- [ ] It is committed, or the commit is proposed
- [ ] PO decision recorded: `Status: approved` (→ Stage 2, `ai-sdlc-2-spec`) or `rejected`

## Agent-generated intents (from Stage 6, tickets, chat)

When Claude writes the intent from a signal and not from a conversation, add these sections
from the template:

- **Evidence:** metric values, the baseline, timestamps, log lines, links
- **Actions taken:** for example, the pre-approved rollback that was run and the recovery check

Set `Author:` to the agent run or trigger. The human triage (fix now / schedule / dismiss)
replaces the originator review.

## Anti-patterns

- Writing a spec instead of an intent (requirements lists, architecture, tech stack)
- Claude's words replacing the originator's, or Claude inventing requirements nobody asked for
- Vague constraints ("must be fast", "secure")
- Waiting until every open question is answered. Carry them forward to Stage 2.
- Leaving the intent uncommitted or in a chat. It has to be in Git to count.

## Metrics

- Leading: time from the first conversation to the committed `intent.md` (from Git history).
  The target is hours, not weeks.
- Lagging: the survival rate (intents accepted into Stage 2), and the number of edits to
  `intent.md` after the first `spec.md` commit. Many edits mean the intent was unclear.

## Hand-off

Next: **Stage 2 - Design** (`ai-sdlc-2-spec`). It needs the approved `intent.md`.
