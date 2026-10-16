---
name: ai-sdlc-5-deploy
description: AI-native SDLC Stage 5 (Deploy) - get a verified change reviewed and shipped. Claude reviews PRs against REVIEW.md, spec.md and plan.md (bugs, security, compliance passes; Important vs Nit), runs the fix loop, and the human code owner approves. Human approval gates are encoded as hooks (allow/ask/deny, managed settings), and CI/CD runs claude -p in a sandbox with per-environment MCP deploy tools and rehearsed rollback. Use when the user says "stage 5", "open a PR", "review this PR", "REVIEW.md", "approval gate", "release hook", "deploy", or "CI/CD with Claude".
---

# Stage 5 - Deploy

Three plays. The first is the per-change flow. The other two are set up once per repo or
organization.

| Play | Kind | Reference |
|---|---|---|
| **AI in the PR review loop** | per change, plus a one-time `REVIEW.md` | this file + [pr-review.md](pr-review.md) |
| **Hooks as approval gates** | one-time setup, then enforced on every action | [approval-gates.md](approval-gates.md) |
| **CI/CD integration and deployment** | one-time setup, then runs on every push or release | [cicd.md](cicd.md) |

Governing principle: **the agent may act up to the production gate and cannot pass it.**
Humans approve, based on the findings and evidence the agent provides.

## Per-change flow

### Entry criteria

- The implementation is complete for the plan's scope, and the Stage 4 definition of done is
  met (with evidence).
- `intent.md`, `spec.md` and `plan.md` are committed on the branch (or on main).

### Steps

1. **Open the PR** (only with the user's go-ahead, because it is outward-facing). Its
   description contains:
   - links to `intent.md`, `spec.md` and `plan.md`
   - the requirements covered (`R<n>`), and anything deferred
   - a summary of the plan deviations
   - the verification evidence (build, test and lint output excerpts; screenshots)
2. **Run the review.** Use the managed Code Review, claude-code-action in CI, or locally the
   `/code-review` skill or a fresh-context reviewer subagent. The review follows `REVIEW.md`
   and reads `spec.md` and `plan.md`, not only the diff. If there is no `REVIEW.md` yet, offer
   to create one ([pr-review.md](pr-review.md)).
3. **Fix loop.** Address every Important finding. Answer review comments (`@claude` on a
   comment makes Claude fix it and push, with the request and the fix recorded in the
   thread). Re-run the checks and the review until only code owner approval is left.
4. **Feed back.** A mistake flagged a second time goes into `CLAUDE.md` in this PR (two-strike
   rule). Also flag changes that leave `CLAUDE.md`, `spec.md` or `plan.md` stale.
5. **Human approval.** The code owner approves through branch protection. **The authoring agent
   never approves its own work**, and review findings never approve or block a PR on their
   own.
6. **Merge and release** through the pipeline and gates ([cicd.md](cicd.md),
   [approval-gates.md](approval-gates.md)). Production needs the release manager's
   authorization. Record what was released, the status after the release, and any rollback,
   in `release-notes.md` or the release record.
7. **Close out.** Close the tracker items and mark the change as released in the plan or
   intent status. The change now enters Stage 6.

### Exit criteria

- [ ] The PR links intent, spec and plan, and contains the evidence
- [ ] There are no open Important findings, and nits are handled or consciously skipped
- [ ] A human code owner approved it
- [ ] Merged, and released through the gates (or the release is scheduled with an owner)
- [ ] The release outcome is recorded, and the tracker is updated

## Small or solo repos

When there is no CI, branch protection or deploy targets yet, keep the **decision points** and
make them explicit: run a fresh-context review against `REVIEW.md`/spec/plan, ask the owner
for an explicit "approved to merge", and ask for an explicit "approved to release". Record the
gaps (no CI, no gates) as follow-up issues, and don't pretend the controls exist.

## Anti-patterns

- Opening PRs without spec and plan, which hides compliance issues the diff alone can't show
- Self-approval by the authoring agent, or findings that auto-block or auto-approve
- Uncapped nits, or reviewing generated code and things CI already enforces
- Pushing directly to main, or releasing to production without a release manager
- Repeated review findings that never make it into `CLAUDE.md`

## Hand-off

After the release, go to **Stage 6 - Maintain** (`ai-sdlc-6-maintain`).
