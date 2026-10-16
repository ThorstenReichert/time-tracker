# Play: AI in the PR review loop

## Goal

Claude reviews every PR against the organization's policies with the same passes, ranks the
findings by severity, and addresses the comments on PRs it authored. Engineers focus on
whether the change matches the intent and whether its risk is acceptable. Review capacity no
longer depends on how much humans can read.

## Entry criteria

- An up-to-date `CLAUDE.md`. Skills, if the review is meant to enforce written policies.
  Defined subagents.
- A review integration: the managed **Code Review** service (an admin enables it per repo), or
  **claude-code-action** in your own CI (for pipeline control or routing through Bedrock,
  Vertex AI or Foundry).
- Branch protection that requires code owner approval.

## Steps (setup)

1. **Enable review.** Start with the managed service. Use the action when you need pipeline
   control.
2. **Write `REVIEW.md`** at the repo root (the tech lead owns it). Use the template below.
3. **Set the human threshold.** Findings never approve or block on their own, and code owner
   approval stays required. Optionally, gate merges on the severity counts published in the
   check run.
4. **Fix loop.** `@claude` on a comment → Claude fixes it and pushes. `@claude review` asks for a
   fresh review. For PRs Claude opened, a custom slash command can sweep the unresolved
   comments and failing checks, fix them and push, until the PR is green and only waits for
   code owner approval.
5. **Feed findings into `CLAUDE.md`.** Add a mistake the second time it is flagged.
6. **Tune monthly.** The tech lead rates findings, caps nits, and excludes generated paths and
   anything CI already enforces.

## `REVIEW.md` template

```markdown
# Review policy

Every finding is tagged with its pass and its severity.

## Passes
1. **Bugs:** logic errors, edge cases, error handling, concurrency, resource leaks.
2. **Security:** authn/z, input validation, secrets, PII in logs or errors, injection, dependency risk.
3. **Compliance:** the diff matches `spec.md` (R-numbers), `plan.md` (files, order, proof) and
   the design principles in `CLAUDE.md` and the skills. Flag any deviation that isn't recorded
   in plan.md's Deviations section.

## Severity
- **Important:** behaviour breaks, data leaks, policy or spec breaches, missing proof for a
  requirement.
- **Nit:** style, naming, minor readability. **At most 5 nits per review.** Summarize the rest
  as a count.

## Skip
- Generated files: `<paths>`
- Anything CI already enforces (formatting, lint rules): `<list>`

## Also flag
- Changes that leave `CLAUDE.md`, `spec.md` or `plan.md` stale.
```

## Output of a review

The check run (or the local report) is a list of findings ranked by severity, each one with its
pass, file and line, and the reason, plus a tally that a machine can read (for example
`important=2 nit=1`). Example finding: "Important (compliance): the panel fetches on every
render, but plan.md requires caching, because the backend allows 50 req/s." A reviewer that
looks only at the diff would miss this.

## Governance

- Separation of duties: the agent that wrote the code can't approve it.
- `REVIEW.md` applies to every PR. Findings, fixes, ratings and approvals stay in the PR
  history, which is the audit record.

## Metrics

- Leading: the time to first review (minutes), and the share of comments resolved without a
  human touching the branch.
- Lagging: defects and vulnerabilities caught before merge compared with those found in
  production.
