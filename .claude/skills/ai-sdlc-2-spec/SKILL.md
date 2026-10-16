---
name: ai-sdlc-2-spec
description: AI-native SDLC Stage 2 (Design) - turn an approved intent.md into spec.md (numbered requirements, design, a map from each intent constraint to how it is met, open questions, areas of concern) in one session, with the organization's policy skills applied. Use when the user says "stage 2", "requirements and design", "write the spec", "spec.md", or an intent.md has just been approved.
---

# Stage 2 - Design: requirements and design as `spec.md`

## Goal

Turn the approved `intent.md` into a `spec.md` that engineering can plan against, with
concerns flagged. Requirements and design happen in **one** AI-assisted session, not in two
phases. The product owner **reviews** the spec but doesn't write it. `intent.md` and `spec.md`
are committed together, so the pair records what was asked and what was decided.

## Entry criteria

- `intent.md` with `Status: approved`. If it isn't approved, stop and offer Stage 1.
- The policy skills that exist are available: security, compliance, brand, UX, API, and so on.
  Check `.claude/skills/` and the plugins. If there are none, write "Skills applied: none" in
  the spec header. Don't make up policies.

## Steps

1. **Read everything first:** `intent.md`, the relevant policy skills, `CLAUDE.md`, and, for
   existing code, the parts of the codebase the change touches. Use a subagent for wide code
   exploration.
2. **Do the core pass.** This is the prompt pattern from the course. Run it by hand first,
   then turn it into a slash command once it has stabilized:

   > Read the attached intent. Apply our <brand / security / UX / …> skills. Name every
   > constraint from the intent. Document the requirements and design fully. Flag conflicting
   > policies. Describe clearly any areas of concern.

3. **Resolve decisions with the owner.** The intent leaves some decisions open on purpose
   (technology, storage, UX details) and lists open questions. Present the real options with
   a recommendation and the trade-offs (use `AskUserQuestion`). Don't decide silently on
   anything the owner would care about. Record each answer in the spec.
4. **Write `spec.md`** using [template.md](template.md). Rules:
   - Number everything so later stages can cite it: requirements `R<n>`, carried-forward open
     questions `OQ-<n>`, areas of concern `AC-<n>`.
   - Requirements are testable statements of behaviour, including edge cases, defaults and
     limits. Put formulas and algorithms in full where precision matters.
   - The design covers architecture, components and the direction of their dependencies, the
     data model, interfaces, error handling, security and privacy, and the testing approach.
     It stays at design level and doesn't list file-by-file steps (that is `plan.md`).
   - **Constraint traceability:** a table that maps every constraint and success criterion
     from the intent to the R-numbers or design sections that satisfy it. A constraint with no
     row is a gap.
   - **Open questions:** each intent question is either *answered* (with a pointer to the R
     or design section) or *carried forward* as `OQ-<n>`, with when or how it will be resolved.
   - **Areas of concern:** risks, policy conflicts, unverified assumptions about other
     systems (rate limits, permissions, machine policies), and trade-offs that were accepted.
     Each one gets an **owner** and either a mitigation or "accepted, because …".
   - Exclusions: repeat the intent's out-of-scope items and add the ones decided here.
5. **Review against the original idea,** together with the owner:
   - Does the spec solve the problem stated in the intent?
   - Is every constraint mapped?
   - Is every open question answered or carried forward?
   - Did the spec add scope the intent didn't ask for? If so, remove it or get it confirmed.
6. **Resolve flagged concerns first.** Route each AC to its policy owner before engineering
   uses the spec. A concern that changes the *intent* goes back to Stage 1 (edit `intent.md`
   and note it).
7. **Commit `spec.md` next to `intent.md`** (or propose the commit, depending on the repo's
   git policy). Record which skills and versions were applied.
8. **Go / no-go:** the PO decides. For higher-risk work they consult a tech lead. On "go",
   set `Status: approved`. That starts Stage 3.

## Front-end work

The PO can mock up the UI from `intent.md` in a design tool (for example Claude Design),
iterate, and reference or export the result in the spec.

## Exit criteria

- [ ] `spec.md` follows the template, with numbered R / OQ / AC
- [ ] Every intent constraint and success criterion appears in the traceability table
- [ ] Every intent open question is answered or carried forward
- [ ] Every area of concern has an owner and a mitigation or acceptance
- [ ] The policy conflicts that were flagged are resolved with their policy owners
- [ ] Committed together with `intent.md` (or the commit is proposed)
- [ ] PO go decision: `Status: approved` (→ Stage 3, `ai-sdlc-3-build`)

## Automation (later)

Once the manual pass is stable, turn it into a slash command. After that, use a non-interactive
job (`claude -p`) that runs when an intent is accepted and opens a PR with `spec.md` for the
PO to review.

## Anti-patterns

- Generating the spec without the organization's skills loaded
- Skipping the PO's review of the spec
- Engineering starting while concerns are still unresolved
- Requirements that can't be tested, or that aren't numbered
- Implementation steps in the spec (they belong in `plan.md`)
- Quietly changing the intent in the spec. Changes to the intent go back to `intent.md`.

## Metrics

- Leading: time from the `intent.md` commit to the `spec.md` commit.
- Lagging: the number of `spec.md` commits after the first `plan.md` commit (rework after the
  build started).

## Hand-off

Next: **Stage 3 - Build** (`ai-sdlc-3-build`). It starts in plan mode with `intent.md` and
`spec.md`.
