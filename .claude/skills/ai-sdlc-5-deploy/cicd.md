# Play: CI/CD integration and deployment

## Goal

Run Claude Code non-interactively in the pipeline for the steps that need judgment (triaging
failures, summarizing flaky tests, drafting changelogs). Sandbox agent jobs, expose deploy,
status and rollback as **per-environment MCP tools**, and rehearse rollback before you need it.
The agent may prepare and ship changes, but the organization's gates control production.

## Entry criteria

- The PR review loop and the approval gates are in place.
- A CI platform with claude-code-action, or any runner that can call `claude -p`.
- Model access (API, Bedrock, Vertex AI or Foundry).
- An MCP server for each deployment target.
- A sandbox profile for agent jobs, with **no standing production credentials**.

## Steps (roll out in this order)

1. **Read-only first:** `claude -p` jobs for failure triage, flaky-test summaries and changelog
   drafts. Example: a job triggered by a failure reads the build log and appends a three-line
   summary to `triage.md`.
2. **Writes behind existing gates:** lint fixes, generated docs, responses to `@claude`. All of
   the output arrives as a PR through branch protection. **There is no push path to main.**
3. **Sandbox execution:** containers, network policies, and short-lived scoped tokens.
4. **Deployment through MCP:** one server per environment (`deploy-dev`, `deploy-staging`,
   `deploy-prod`), each with its own token. Allowlists match the server and tool name, not the
   arguments, so separate servers are what make staging and prod distinguishable. All servers
   offer the same tools:
   - `release(version[, ticket in prod])` → the release ID
   - `status()` → waits after the release, then reports the error rate per endpoint
   - `rollback()` → the live version
5. **Tier autonomy by environment:**

   | Env | `release` | `status` | `rollback` |
   |---|---|---|---|
   | dev | allowed | allowed | allowed |
   | staging | allowed; the hook asks, or allows from the pipeline on `main` | allowed | allowed |
   | prod | **not allowlisted**: only a hook `allow` (approved ticket from the release manager) gets through, and it fails closed | allowed | allowlisted (a pre-approved runbook) |

6. **Rehearse rollback.** It is a single command the agent can run, exercised regularly in
   staging. Stage 6 relies on it.

## Example staging job (outline)

On a push to `main`, run
`claude -p "<release, check status, roll back if any endpoint's 5xx > 1%>" --allowedTools "mcp__deploy-staging__release,mcp__deploy-staging__status,mcp__deploy-staging__rollback" --permission-mode dontAsk`
with only the staging token in the environment. Write the outcome to `release-notes.md`: what
was released, what the status reported, and whether it rolled back. Lock the configuration
with the managed settings `allowManagedHooksOnly` and `allowManagedMcpServersOnly`.

## Governance

- Branch protection routes every agent change through a PR.
- The production deploy hook requires a named release manager's authorization.
- Non-interactive runs use the **agent's own identity**, so logs separate agent actions from
  the engineer who triggered them.
- Per-environment permission tiers limit the agent's scope before it reaches the gate.

## Exit criteria

- [ ] Read-only jobs run reliably before any write jobs are enabled
- [ ] Agent writes reach main only through reviewed PRs
- [ ] Agent jobs use short-lived scoped tokens, with no standing prod credentials
- [ ] Deploy tools are per-environment MCP servers with allowlists, and the prod hook fails
      closed
- [ ] Rollback is one command, rehearsed in staging
- [ ] Triage and DORA metrics are tracked

## Anti-patterns

- Standing production credentials, or shell scripts with embedded secrets
- One MCP server shared across environments
- An agent that pushes to main, or auto-releases to production
- Hooks that fail open
- Rollback that is untested until the incident

## Metrics

- Leading: the share of pipeline failures triaged without paging a human.
- Lagging: the DORA metrics (deployment frequency, lead time, change failure rate, time to
  restore).
