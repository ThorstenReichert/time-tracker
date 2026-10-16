# Play: continuous evals in CI

## Goal

Evals are the AI-native version of stage-gate QA. A suite of real tasks runs whenever the
**agent's configuration** changes (`CLAUDE.md`, `.claude/**` skills, hooks, agents, the model, a
prompt rewrite). It proves that the agent still meets the same standard. That configuration
steers the agent, so it needs the same regression testing as code. The suite is **live**: as
models improve, some cases stop telling good output from bad and get replaced by new ones from
monitoring. A fixed nightly or offline schedule works too.

## Entry criteria

- A `CLAUDE.md` and the feedback loop (the commands the checks reuse)
- CI that can run Claude Code non-interactively (`claude -p`)
- An API key or cloud model access with a budget for eval runs

## Steps

1. **Collect 20 to 50 real tasks** from recent work (merged PRs, fixed bugs), each with its
   accepted outcome. For a small project, start with five to ten.
2. **Turn each one into an eval:** a prompt plus checks that define an acceptable result
   (tests pass, lint is clean, behaviour is unchanged, policy is followed).
3. **Run the suite in CI** on a schedule (nightly) and on every PR that touches `CLAUDE.md`,
   `.claude/**` or `evals/**`.
4. **Gate configuration changes on the results.** A change that lowers the pass rate is
   reviewed before it merges. The pass threshold is a merge check.
5. **Turn every production incident into an eval,** written by the team that owned the
   incident. It stays in the suite as a regression test.
6. **Refresh the suite.** Retire cases that every configuration passes trivially, and add new
   ones from Stage 6.

## Artifacts

### `evals/<name>.json`

```json
{
  "name": "status-no-pii-in-logs",
  "source": "PR #123 / incident INC-42",
  "from": "<commit SHA of the code state before the task>",
  "prompt": "Add debug logging to the claim status endpoint.",
  "may_change": ["src/claims/"],
  "checks": [
    "make test",
    "make lint",
    "scripts/no-removed-test-lines.sh",
    "python evals/log_bodies.py"
  ]
}
```

### `evals/check.sh`, the grader

- It fails the run if the Claude run errored, if files **outside `may_change`** changed, or if
  any check fails.
- It flags changes made **by the checks themselves**.
- It runs from a copy made **before** the agent runs, so the agent can't alter the grader or
  the checks.

### Behavioural checks

Test the real risk, not code style. For example, call the endpoint with a made-up record,
capture all log output, and fail if a log line contains sensitive data. Tag-based policy
scripts miss free-text fields, so combine several checks.

### `.github/workflows/agent-evals.yml` (outline)

- Triggers: `pull_request` on paths `CLAUDE.md`, `.claude/**`, `evals/**`, plus a nightly
  `schedule`.
- For each eval:
  1. Create a scratch checkout at the eval's `from` commit.
  2. Remove `evals/` and `.claude/` from that checkout.
  3. Copy in the branch's `CLAUDE.md` and `.claude/` (the configuration under test).
  4. Run `claude -p "<prompt>"` with a restricted `--allowedTools` list.
  5. Grade the result with the pre-copied `check.sh`.
- Report the result per eval (`FAIL <name> (4/5 checks)` with the offending evidence) and the
  overall count (`0 eval(s) failed`).
- **Never use `--bare`.** It skips `CLAUDE.md`, skills and hooks, which are exactly what is
  under test.

## Exit criteria

- [ ] The suite runs on schedule and on every configuration change
- [ ] The pass threshold is enforced as a merge check (default: all pass)
- [ ] Every production incident has a matching eval
- [ ] Cases that stop discriminating are replaced

## Anti-patterns

- Using `--bare` in eval runs
- Letting the agent touch the grader or the checks
- Relying only on tag-based policy scripts
- A static eval set
- Testing only code changes and never the agent configuration

## Metrics

- Leading: the pass rate over time, and the time from an incident to its permanent eval.
- Lagging: regressions caught in CI compared with regressions found in production.
