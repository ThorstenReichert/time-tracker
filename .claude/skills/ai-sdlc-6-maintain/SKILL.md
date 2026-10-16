---
name: ai-sdlc-6-maintain
description: AI-native SDLC Stage 6 (Maintain) - close the loop on production metrics. Deterministic detection (rolling mean/σ + drift rules) with response tiers in bands.yaml (1σ log, 2σ read-only diagnosis, 3σ PR or pre-approved runbook), a headless Claude run that writes its diagnosis as a new intent.md (Stage 1 format with Evidence and Actions taken), human triage, and a regression eval per fixed incident. Use when the user says "stage 6", "maintain", "monitor", "metric breach", "incident", "close the loop", or wants an alert turned into an intent.
---

# Stage 6 - Maintain: closing the loop on metrics

## Goal

Claude runs headless to close the SDLC loop. A trigger (a metric breach, a ticket, a chat
message, a schedule) starts a run, and the run writes an `intent.md` that re-enters Stage 1.
Deterministic gates decide whether work continues automatically or goes to a human. People
**triage and review** the output instead of starting the process themselves.

## Entry criteria

- `intent.md` as a structured format the loop can restart from (Stage 1 template)
- Claude-assisted PR review (Stage 5), and hooks as the action boundary
- For the highest tier: a rehearsed rollback (Stage 5 CI/CD)
- Infrastructure: a metrics store (Prometheus, a CI API, …), repo read access, and
  non-interactive Claude Code in CI (or the Agent SDK for webhook-driven services)

## Steps

1. **Pick one metric** with a stable rolling baseline: the CI test failure rate, the
   post-deploy 5xx rate, or the PR cycle time. Trust one metric end to end before you add more.
2. **Write a deterministic detection script.** Use the rolling mean and standard deviation plus
   drift rules (Western Electric, for example 4 of 5 readings above 1σ). It is version
   controlled, unit tested, and uses **no model**. About 30 lines of standard-library Python
   that print the highest tier reached are enough.
3. **Define the response tiers in `bands.yaml`** (version controlled):

   ```yaml
   metric: post_deploy_5xx_rate      # the only line that changes to monitor another metric
   tiers:
     - at: 1sigma
       action: log                    # noise trips 1σ several times a day, so only log
     - at: 2sigma                     # drift rules also count as 2σ
       action: diagnose
       tools: [read-only]             # Claude diagnoses, no changes
     - at: 3sigma
       action: remediate
       routes: [open-pr, runbook:rollback]   # PR through normal review, or a pre-approved runbook only
   ```
4. **Pick a trigger:** a scheduled workflow, a monitoring webhook, or an in-network cron job.
   Claude runs statelessly, with tools limited to its tier. Claude never picks its own tier.
   The script decides.
5. **The agent writes its diagnosis as `intent.md`** in the Stage 1 format (template in
   `ai-sdlc-1-intent`), including **Evidence** (values, baseline, tier, timestamps, log lines)
   and **Actions taken** (for example "ran pre-approved rollback at 14:01, recovery confirmed
   14:04"). Open questions capture the product decisions a human has to make.
6. **Human triage.** The on-call engineer or service owner works the queue: *fix now* (→ Stage 2/3),
   *schedule*, or *dismiss*. Product-facing findings go to the product owner. Use the
   dismissals to tune the bands.
7. **After a fix ships, add an eval** for the incident (`ai-sdlc-4-test` → evals). It runs from
   the commit before the faulty release, so the same failure is caught next time.

## When Claude is invoked on an alert or incident by hand

1. Confirm the tier from the detection output (don't judge severity yourself).
2. Diagnose read-only: recent releases and diffs, logs, metrics, the plan/spec of the suspect
   change.
3. Act only within the tier's allowed routes. Anything beyond them needs explicit human
   approval.
4. Verify recovery if you took an action. Then write the `intent.md` in a new change folder and
   hand it to triage.

## Exit criteria

- [ ] One metric runs end to end: deterministic detection → tiered response → `intent.md` in
      the triage queue, with timestamps
- [ ] Tier boundaries are enforced by config and permissions (no production access beyond the
      pre-approved runbooks)
- [ ] Every shipped fix has an eval that would fail if the issue came back
- [ ] The bands are tuned from dismissals until the noise is acceptable
- [ ] Leading and lagging metrics are tracked

## Chat-based first responder (optional)

With Claude in Slack (Claude Tag), Claude can join incident channels under its own identity,
verify recovery through MCP, post in the thread, record a post-mortem in a versioned lessons
file, and triage tickets (small fixes become PRs, and larger work becomes an `intent.md`).

## Anti-patterns

- Using a model for detection, or letting Claude choose its own tier
- Acting on every 1σ blip
- Giving the agent production access or broad tools, or letting it bypass PR review
- Running without a rollback path or pre-approved runbooks
- Skipping the eval after a fix, so the incident recurs
- Ignoring dismissals instead of tuning the bands
- Config outside version control, or detection without tests
- Scaling to many metrics before one is trusted

## Metrics

- Leading: the time from a breach to a triaged `intent.md` (compared with the old time from
  incident to post-mortem action).
- Lagging: the share of findings that become merged fixes, and repeat incidents of the same
  class (should fall as evals are added).

## Hand-off

The new `intent.md` goes back to **Stage 1** (`ai-sdlc-1-intent`) for triage or PO acceptance,
and the loop continues.
