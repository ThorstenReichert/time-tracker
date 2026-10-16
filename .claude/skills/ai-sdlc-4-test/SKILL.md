---
name: ai-sdlc-4-test
description: AI-native SDLC Stage 4 (Test) - give Claude a feedback loop (one-command build/test/lint with healthy-output examples in CLAUDE.md, measurable targets, failing-first tests for bugs, screenshot loops for UI, verification in the definition of done, hooks protecting tests) and run continuous evals in CI whenever CLAUDE.md, skills or hooks change. Use when the user says "stage 4", "feedback loop", "definition of done", "verify your work", "fix this bug" (test-first), "evals", or "test the agent setup".
---

# Stage 4 - Test

Two plays:

| Play | Tests | When | Reference |
|---|---|---|---|
| **Feedback loop** | the *change*: Claude checks its own work while it works | every task, from the first line of code | this file |
| **Continuous evals in CI** | the *agent setup*: does Claude still behave well after `CLAUDE.md`, skills, hooks or the model change? | once the agent config carries real weight | [evals.md](evals.md) |

## Play: give Claude a feedback loop

### Goal

Claude verifies its own work with tests, builds and screenshot comparisons, and catches and
fixes mistakes **before** an engineer sees the output. The loop runs throughout the task, not
only at the end. A verifier subagent in a fresh context (see Stage 3) is a separate, optional
final check.

### Entry criteria

- A test suite and a build that each run locally with **one command**.
- For UI work: a way for Claude to see the result (a browser tool such as Claude in Chrome, or
  a screenshot utility through MCP).

### Steps (setting up the loop, once per repo)

1. **One command per check.** Wrap multi-step checks (`make test`, `npm test`, `cargo test
   --workspace`, …). Each command must exit non-zero on failure.
2. **List the commands in the `CLAUDE.md` Commands section**, each with an example of healthy
   output.
3. **Add verification to the definition of done in `CLAUDE.md`.** Use this block:

   ```markdown
   ## Definition of done
   Before reporting any task complete, run all three and paste the literal output:
   - Build: `<cmd>`, which must end with "<healthy marker>".
   - Test: `<cmd>`, all green. No skipped, deleted or weakened failing tests.
   - Lint: `<cmd>`, zero warnings.
   If a test fails, fix the code, not the test.
   ```
4. **Protect the loop.** During fix tasks, block edits to test files. Use a `PreToolUse` hook
   on `Edit|Write` that denies paths matching the test globs while a fix is in progress (for
   example when a marker file or an env var is set), or reject any fix diff that touches tests
   in review. Prefer the hook, and use review only as a fallback.

### Steps (each task)

1. **Define a measurable target** before you start: specific tests pass, a screenshot matches
   the mock, an endpoint returns the expected status and fields. Take it from the `plan.md`
   Proof section if one exists.
2. **Bug fixes go test-first:**
   1. Write a test that reproduces the bug.
   2. Run it and confirm it **fails for the expected reason**. Show the output.
   3. Commit the failing test (or propose the commit).
   4. Make it pass **without editing the test**.
3. **UI work:** implement → screenshot → compare with the mock → adjust. This usually takes
   two or three rounds.
4. **Run the loop after every meaningful step,** not only at the end.
5. **Report with evidence.** Paste the literal toolchain output (the relevant excerpt, with
   the pass/fail summary lines), not a summary like "tests pass".
6. Optionally, before reporting done, run a verifier subagent in a fresh context.

### Exit criteria (definition of done for any task)

- [ ] Build, test and lint pass, and their output is shown
- [ ] For bug fixes: the pre-existing failing test now passes, unmodified
- [ ] For UI: the screenshot matches the mock
- [ ] No tests were skipped, deleted or weakened to get green

### Anti-patterns

- Reporting done without running the checks, or summarizing instead of pasting output
- Editing or deleting failing tests to get green, or adding skip markers
- Writing the fix before a failing test exists
- Relying on late signals (CI minutes later, testers days later). That makes humans the
  bottleneck.
- Relying only on reviewers to spot test edits
- Treating the feedback loop and the verifier subagent as the same thing

### Metrics

- Leading: the first-pass CI success rate of agent-written changes.
- Lagging: review time per PR, and the change failure rate.

## Hand-off

When the work is done and verified, go to **Stage 5 - Deploy** (`ai-sdlc-5-deploy`). Each
production incident later (Stage 6) becomes a permanent eval ([evals.md](evals.md)).
