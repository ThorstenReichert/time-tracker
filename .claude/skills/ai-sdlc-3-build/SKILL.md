---
name: ai-sdlc-3-build
description: AI-native SDLC Stage 3 (Build) - start in plan mode from intent.md + spec.md, interrogate and commit an approved plan.md (files, order of work, risks, proof; each step cites R-numbers), then implement it with a feedback loop and keep plan.md in sync. Also covers the enabling plays - maintaining CLAUDE.md, encoding policy as skills (+ hook backstops), and parallel sessions/worktrees and subagents. Use when the user says "stage 3", "plan mode", "write plan.md", "implement the spec", "set up CLAUDE.md", "write a policy skill", "run this in parallel", or "create a subagent".
---

# Stage 3 - Build

Stage 3 has one main flow and three enabling plays:

| Play | When | Reference |
|---|---|---|
| **Plan mode → `plan.md` → implement** | every change | this file + [plan-template.md](plan-template.md) |
| The `CLAUDE.md` | a missing or stale CLAUDE.md, or a repeated mistake (two-strike rule) | [claude-md.md](claude-md.md) |
| Skills as institutional knowledge | a policy that is applied inconsistently | [policy-skills.md](policy-skills.md) |
| Parallel sessions and subagents | independent tasks, or recurring jobs inside a session | [parallel-and-subagents.md](parallel-and-subagents.md) |

## Main flow: plan mode as the default starting point

### Goal

Before any code is written, Claude produces a written plan in read-only plan mode. The engineer
interrogates and corrects it, and the approved version is committed as `plan.md`. Then Claude
implements it, often in one pass. Design review happens **before** code exists, and the Stage 5
PR review checks the diff against `plan.md`.

### Entry criteria

- `spec.md` with `Status: approved`, and its `intent.md`. For small changes without a spec,
  plan mode still applies. Use the intent or the task description and say that the spec is
  missing.
- Claude Code with access to the repo. A `CLAUDE.md` helps but isn't required. If there is
  none, suggest the CLAUDE.md play first.

### Steps

1. **Enter plan mode.** Use the `EnterPlanMode` tool, or ask the user to press Shift+Tab until
   plan mode is on. Plan mode is read-only and blocks edits until the plan is accepted.
2. **Read the inputs:** `intent.md`, `spec.md`, `CLAUDE.md`, and the code you will touch
   (explore with subagents if the area is wide). Check the spec's open questions and areas of
   concern. Any of them that block the plan have to be resolved now.
3. **Interview the engineer.** Ask about the choices the spec left to implementation: library
   picks, slicing, the testing tools, and what should come first. Also ask about the spec's
   mitigations that affect the order of work (for example "verify X early").
4. **Draft the plan** using [plan-template.md](plan-template.md):
   - the files that change (new, modified, deleted), each with its purpose
   - the order of work in small steps that can be verified. **Every step cites the
     requirements it implements (`R<n>`)**, so reviewers can check the plan against the spec
     line by line.
   - risks: what could break, the riskiest step and how it is de-risked, and the alternatives
     that were considered and rejected (with the reason)
   - proof: the tests, commands and visual checks that show each step and the whole change
     work (this feeds Stage 4)
   - a coverage check: every `R<n>` in scope appears in at least one step. Anything deferred
     is listed explicitly.
   - for large specs, plan the **thin vertical slice first** (the end-to-end skeleton), then
     the increments
5. **Interrogate the plan.** Proactively state what could break, which step is riskiest, which
   alternatives were rejected, and what is still assumed. Invite the engineer to push back.
   Iterate.
6. **Apply the stranger test.** Iterate until someone who never saw this conversation could
   implement the change from `plan.md` alone. Pin down names, paths, commands and versions.
7. **Get approval.** Present the plan (`ExitPlanMode`). Routine changes: the engineer approves.
   Higher-risk changes (security, data migrations, public APIs, infrastructure): ask the user
   to get tech lead or architect approval, and record who approved.
8. **Save and commit `plan.md`** in the change folder with `Status: approved` and the approver.
   Commit it if the git policy allows, otherwise propose the commit. If the repo tracks work
   in an issue tracker, create issues for the plan steps or slices, with dependencies.
9. **Implement.** Follow the plan step by step and keep the feedback loop on
   (`ai-sdlc-4-test`): run the proof for each step, and paste the real output. Use parallel
   sessions or subagents only for independent work
   ([parallel-and-subagents.md](parallel-and-subagents.md)).
10. **Keep the plan true.** If the implementation diverges (a different file, order or
    approach), update `plan.md` **in the same commit** as the code that diverges, and add a
    "Deviations" note. Optionally, enforce this with a hook or a CI check.
11. **Learn as you go.** If Claude makes the same mistake twice, add the correction to
    `CLAUDE.md` (two-strike rule). If a policy wasn't applied, consider a skill or a hook.

### Exit criteria

- [ ] `plan.md` approved by the right person, committed (or the commit is proposed), all R in
      scope covered
- [ ] The implementation is complete for the planned scope, and `plan.md` matches what was
      built (deviations are recorded)
- [ ] Every step's proof was run, and its output was shown (Stage 4 definition of done)
- [ ] Follow-up work is recorded in the tracker, not left in comments
- [ ] Ready for Stage 5 (`ai-sdlc-5-deploy`): a branch or PR exists with intent, spec and plan
      linked

### Auto mode

Once the guardrails are in place (a tuned `CLAUDE.md`, policy skills, hooks that block unsafe
actions, a test suite that runs), routine work can run with auto-accepted edits after the plan
is approved. Review then shifts from watching edits to reviewing artifacts and evidence after
longer sessions. Don't suggest auto mode while those guardrails are missing.

### Source of truth with existing tools

If Jira, a requirements tool, Figma or a change board already exists, name **one** source of
truth for each artifact:

- **Repo is truth:** the markdown is authoritative, and the other tool links to the commit.
- **Tool is truth:** the markdown is a working copy, and Claude reads and writes through MCP.
- **Linkage (the minimum):** the markdown cites the record ID, and the record cites the
  commit SHA.

### Anti-patterns

- Writing code before the plan is approved, or skipping plan mode "because it's small"
- Plan steps that don't cite requirements, so the plan can't be checked against the spec
- Vague proof ("test it"), when it should name the exact commands or tests
- A plan that drifts silently from the code
- Accepting the first draft without interrogating it

### Metrics

- Leading: the share of changes merged on the first implementation pass, and the time from
  plan approval to the merged PR.
- Lagging: rework cycles per change, and how often the merged diff still matches `plan.md`.
