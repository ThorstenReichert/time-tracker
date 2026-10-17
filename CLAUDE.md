# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:6cd5cc61 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->


## Development process: AI-native SDLC

Changes follow the six-stage AI-native SDLC (Plan → Design → Build → Test → Deploy → Maintain),
encoded as repo-local skills in `.claude/skills/`. Start with `ai-sdlc` (overview, artifact chain,
how to tell which stage a change is in). Then use the stage skills: `ai-sdlc-1-intent` (intent.md),
`ai-sdlc-2-spec` (spec.md), `ai-sdlc-3-build` (plan.md + implementation), `ai-sdlc-4-test`,
`ai-sdlc-5-deploy` and `ai-sdlc-6-maintain`. When the user names a stage, load its skill and
follow it.

## Build & Test

## Architecture Overview

Single-user Windows tray app: Rust (Tauri 2) + Vue 3/TS + SQLite. Full design in `spec.md` §2;
`R<n>` refers to its requirements. Style: functional core / imperative shell (Bernhardt), the core
shaped as a Decider (Chassaing), drivers as ports & adapters (Cockburn), and determinism in the
spirit of TigerBeetle. Goal: the app can be driven equally by the UI, the hotkey, the CLI and tests.

### Layout and dependency direction

```
crates/tt-core   pure domain: types, Decider (decide/evolve), invariants, rounding, read models
crates/tt-store  SQLite: migrations, load State, apply Events in one transaction, backup/export
crates/tt-app    the single write path: owns State + Store, executes Commands, publishes Events
crates/tt-cli    headless driver for scripts, scenario tests and verification
src-tauri/       GUI driver: Tauri commands, tray, global hotkey, autostart, clipboard
ui/              Vue 3 + TS; calls Tauri commands only, never touches the DB
```

Dependencies point inward only: `src-tauri, tt-cli → tt-app → tt-store → tt-core`.
`tt-core` depends on no IO crate (no Tauri, no rusqlite, no `std::fs`/`std::net`).

### tt-core: deterministic functional core

- **Decider shape.** `decide(&State, &Ctx, &Command) -> Result<Vec<Event>, DomainError>` and
  `evolve(State, &Event) -> State`. Every state change is an Event; there is no other path.
- **Everything is an input.** `Ctx { now, tz }` carries the current instant (UTC seconds) and
  the time zone. IDs are assigned by the core from counters in `State`. Forbidden in tt-core:
  reading the system clock or OS time zone, randomness, env vars, file/network IO, threads,
  global mutable state. Same inputs → identical outputs.
- **Whole world in memory.** `State` holds all clients, projects, tasks, entries, favorites and
  settings, loaded once at startup. The core enforces the invariants itself (R1, R2, R4, R12);
  the SQLite constraints are only a backstop.
- **Plain values at the boundary.** `Command`, `Event` and `State` are plain `serde` data:
  printable, comparable and serialisable.
- **Reads are pure functions** over `State` (+ date/range + tz): daily overview, statistics,
  billing error (R14–R21). They don't go through `decide`, and they are never stored.
- **Faux-O style.** Newtypes with validating constructors (parse, don't validate). Make illegal
  states unrepresentable (e.g. a rounding unit can only be a divisor of 60). Methods return new
  values. Mutation never leaks out of a function.
- **Assert invariants.** `State::check_invariants()` (at most one running entry, no overlaps,
  parent references valid) runs after every `evolve` in debug builds and tests.

### tt-app: the imperative shell

- `execute(Command) -> Result<Vec<Event>, AppError>`: stamp `Ctx` (the real clock and tz are
  read **only** here) → `decide` → `store.apply(events)` in one transaction → only after commit,
  `evolve` the in-memory State → publish the events (`src-tauri` turns them into
  `data-changed`/`timer-changed`). If persisting fails, State stays unchanged.
- One process per DB file (exclusive lock). `tt-cli` refuses while the app is running; use
  `--db <path>` for scratch databases.
- **No business rules outside tt-core.** If tt-app, a driver or the UI needs a business-rule
  `if`, move it into tt-core.

### Drivers

`src-tauri` and `tt-cli` only translate input → `Command` and result → output. Every feature
must be reachable without the UI (through `tt-app`, and so from tests and `tt-cli`).

### Testing seams (details: stage 4)

- tt-core: given State / when Command / then Events-or-error tests. Model-based property
  tests: random Command sequences with a simulated clock, invariants checked after every step.
- tt-store + tt-app: the same sequences against a temp SQLite DB; State reloaded from disk must
  equal the in-memory State.
- tt-cli: end-to-end scenario scripts against a temp DB.

## Conventions & Patterns
