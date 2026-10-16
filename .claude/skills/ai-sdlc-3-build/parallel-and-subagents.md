# Play: parallel sessions and subagents

## Goal

One engineer runs several streams of work at once, and recurring jobs become reusable
subagents. The engineer's role shifts from typing to orchestrating and reviewing.

- **Parallel session:** a separate Claude Code instance working on one task in its own Git
  **worktree**. Sessions don't know about each other, and the engineer is the only link
  between them.
- **Subagent:** a scoped helper inside one session, with its own context window and a limited
  set of tools. It is best for jobs that recur across tasks (verifying, simplifying,
  researching).

## Entry criteria

- A `CLAUDE.md`, because every session reads it.
- A feedback loop (Stage 4), so sessions can check their own work and need less supervision.
- A Git repo (worktrees provide the isolation).
- Permission settings that pre-allow the commands the team considers safe (build, test, lint,
  git), so sessions don't stall on prompts. See the `fewer-permission-prompts` /
  `update-config` skills if they are available.

## Steps

1. **Split the work into tasks that touch different files.** Use the `plan.md` file table to
   find independent steps. Tasks that share files run one after another in a single session.
2. **Give each parallel task its own worktree** (`claude --worktree`, the `EnterWorktree` tool,
   or `git worktree add ../<repo>-<task> -b <branch>`). Each worktree has its own branch and
   its own PR.
3. **Start with two or three sessions.** Add more only while your review keeps pace.
4. **Turn repeated jobs into subagents.** Define each one in `.claude/agents/<name>.md` with:
   - `name`
   - `description`: when to use it
   - `tools`: the minimal set it needs
   - a body with instructions and the report format

   Check the files into Git so the team shares them. Typical ones are a verifier, a code
   simplifier, and a researcher.

## Example: verifier subagent

```markdown
---
name: verifier
description: Use after implementing a plan step or before reporting a task done. Starts the app, exercises the changed behaviour and nearby flows, and reports pass/fail against plan.md. Never edits code.
tools: Read, Grep, Glob, Bash
---
You verify; you do not fix.
1. Read plan.md (the step(s) just implemented and their Proof section) and the diff.
2. Run the build, test and lint commands from CLAUDE.md. Capture the literal output.
3. Start the app if relevant and exercise the changed behaviour and adjacent flows.
4. Report per proof item: PASS/FAIL, the evidence (output excerpt), and for FAIL the exact
   reproduction. Do not modify files.
```

## Governance

- Controls belong in the repo configuration (hooks, permissions), so they apply to every
  session automatically.
- Session activity is logged and attributed to the engineer who ran it (for example through
  the OpenTelemetry export).

## Anti-patterns

- Running tasks that touch the same files in parallel
- Adding sessions faster than review can keep up
- Subagents with broad tool access ("all tools") when they only need read and run
- A verifier that fixes code (that mixes up roles and hides failures)

## Metrics

- Leading: concurrent sessions per engineer while review quality holds, and the share of the
  day spent steering rather than waiting.
- Lagging: changes merged per engineer per week, read together with the rework rate.
