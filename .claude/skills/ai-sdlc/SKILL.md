---
name: ai-sdlc
description: Overview and router for the AI-native SDLC (six stages Plan, Design, Build, Test, Deploy, Maintain, with the artifact chain intent.md → spec.md → plan.md → code + evidence → PR → release → metrics → new intent.md). Use when the user mentions the "AI-native SDLC", "the SDLC playbook", "stage N", asks which stage a change is in, what comes next, or where to start. It routes to the matching stage skill (ai-sdlc-1-intent … ai-sdlc-6-maintain).
---

# AI-native SDLC: overview and router

Based on the Claude Academy course *The AI-native SDLC playbook*. Software moves from idea to
production through six stages. Each stage leaves a versioned artifact in Git that the next
stage reads. Humans stay accountable for every judgment call, and Claude does the drafting,
building, checking and monitoring.

## The six stages

| # | Stage | Main question | Primary artifact | Stage skill |
|---|---|---|---|---|
| 1 | Plan | What is wanted, why, under which constraints? | `intent.md` | `ai-sdlc-1-intent` |
| 2 | Design | What exactly do we build, and how? | `spec.md` (next to `intent.md`) | `ai-sdlc-2-spec` |
| 3 | Build | In which order, touching which files, proven how? Then build it. | `plan.md`, code, `CLAUDE.md`, skills, subagents | `ai-sdlc-3-build` |
| 4 | Test | How does Claude check its own work, and does the agent setup stay good? | verification commands, failing-first tests, evals | `ai-sdlc-4-test` |
| 5 | Deploy | Who reviews, who approves, how does it ship? | `REVIEW.md`, hooks/gates, pipeline, release notes | `ai-sdlc-5-deploy` |
| 6 | Maintain | Is it healthy in production? What should change next? | `bands.yaml`, detection script, new `intent.md` | `ai-sdlc-6-maintain` |

The stages form a loop, not a waterfall. Stage 6 writes a new `intent.md`, which re-enters
Stage 1. Any stage can also send work back: a spec concern can reopen the intent, and a plan
risk can reopen the spec.

## Artifact chain

```
idea / ticket / alert
   │  Stage 1 ── intent.md      (originator's words: problem, outcome, users, constraints, open questions)
   ▼
   │  Stage 2 ── spec.md        (numbered requirements R<n>, design, constraint traceability, OQ-<n>, AC-<n>)
   ▼
   │  Stage 3 ── plan.md        (files, order of work, risks, proof; every step cites R<n>)
   │             code + tests   (built from plan.md; CLAUDE.md and skills steer the build)
   ▼
   │  Stage 4 ── evidence       (pasted test, build and lint output; failing-first tests; evals for the agent setup)
   ▼
   │  Stage 5 ── PR + review    (checked against spec.md and plan.md; human code owner approves; gated release)
   ▼
   │  Stage 6 ── metrics        (deterministic detection → diagnosis → new intent.md + regression eval)
   └──────────────► back to Stage 1
```

## How to act when the user names a stage

1. Load the stage skill from the table above and follow it step by step.
2. Find the **change folder** (see below) and read every earlier artifact of the chain before
   writing the next one. If an earlier artifact is missing or not approved, say so and offer
   to run that stage first. Don't silently skip it.
3. Check the stage's **entry criteria**. Produce the artifact, then check the **exit
   criteria** together with the user.
4. Record the stage transition wherever the repo tracks work (for example its issue tracker
   or the artifact's `Status:` line). Follow the repo's git policy: commit only when the user
   or the repo instructions allow it, and otherwise propose the commit.
5. At the end, name the next stage and what it needs.

If the user only says "continue with the SDLC" or "what's next", figure out the stage from
the artifacts that exist:

| Found | Next step |
|---|---|
| nothing | Stage 1 |
| `intent.md` with `Status: draft` | finish Stage 1 (owner review) |
| `intent.md` approved, no `spec.md` | Stage 2 |
| `spec.md` not yet approved | finish Stage 2 (concerns, sign-off) |
| `spec.md` approved, no `plan.md` | Stage 3 (plan mode) |
| `plan.md` approved, work not done | Stage 3 (implement) with the Stage 4 feedback loop |
| implementation done, no PR or release | Stage 5 |
| released | Stage 6, or a new intent |

## Change folder convention

One change = one set of `intent.md` + `spec.md` + `plan.md` that are kept together.

- If the repo already uses a location (root files, `intent/`, `docs/changes/…`), keep using it.
- Otherwise: the first, founding change of a new project may live at the repo root. Put each
  later change in `changes/<YYYY-MM-DD>-<slug>/` with the same three file names.
- Artifacts reference each other with relative links (`Source: [intent.md](intent.md)`).

## Roles

The course assumes an enterprise. In small teams or solo projects one person can hold several
roles, but the **decision points stay the same**. Make each one explicit, even if you are
asking the same person every time.

| Role | Decides |
|---|---|
| Originator | writes the intent and corrects Claude's draft |
| Product owner (PO) | accepts or rejects the intent (→ Stage 2) and the spec (→ Stage 3) |
| Policy owners (security, compliance, brand, UX) | resolve spec concerns; sign off on policy skills |
| Engineer | interrogates and approves `plan.md`; steers sessions |
| Tech lead / architect | approves high-risk plans; owns `REVIEW.md` |
| Code owner | approves the PR (the authoring agent never approves its own work) |
| Release manager | authorizes production releases |
| On-call / service owner | triages Stage 6 findings |

## Guiding principles

- **Human accountability:** a person owns every decision that needs judgment.
- **Governance as code:** controls (hooks, permissions, CI checks) apply while the agent
  acts, not only in later reviews.
- **Knowledge as versioned artifacts:** standards and context live in Git in a form a
  machine can read (`CLAUDE.md`, skills, `REVIEW.md`).
- **Continuous evaluation:** checks run throughout the work, not only at stage boundaries.
- **Tiered review:** agents review everything. Humans focus on intent, risk and critical code.
- **Closed loop:** production signals flow back into new intents.
- **Measure:** every play names a leading and a lagging indicator. Most of them can be read
  from Git history.

## Where to start (play dependencies)

Plays with no prerequisites, which are good entry points: capture intent, plan mode,
CLAUDE.md, skills, feedback loop, hooks.

```
Requirements & design ← capture intent + skills
Subagents             ← CLAUDE.md (feedback loop helps)
Evals in CI           ← CLAUDE.md + feedback loop
PR review             ← evals + subagents (skills help)
CI/CD                 ← PR review + hooks
Closing the loop      ← CI/CD + capture intent
```

## Where knowledge belongs

| Kind of knowledge | Put it in |
|---|---|
| Day-one context: commands, conventions, architecture, recurring mistakes | `CLAUDE.md` |
| Organization policy that must be applied the same way every time | a skill in `.claude/skills/` |
| A rule that must **always** hold | a hook or CI check (deterministic), backing up the skill |
| Guidance for one task only | the prompt |
| A recurring job inside a session (verify, simplify, research) | a subagent in `.claude/agents/` |
| Review policy | `REVIEW.md` |
