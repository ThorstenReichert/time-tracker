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

### Beads ↔ SDLC

`plan.md` is the approved design of the work; beads is its live status. They don't overlap:

- **`plan.md` holds no status.** No checkboxes, no "done" marks. Progress lives only in beads,
  so `plan.md` is not a "markdown TODO list" in the sense of the beads rules.
- **Beads come after approval.** Plan mode is read-only, so create no beads while planning. Once
  `plan.md` is approved (stage 3, step 8), create one bead per plan step or slice, with `bd dep`
  dependencies matching the plan's order. Each bead names its plan step and the `R<n>` it covers.
  Then implement through `bd ready` / `bd update --claim` / `bd close`.
- **Deviations update both.** If the work changes scope, order or approach, update `plan.md`
  (with a "Deviations" note) and create, change or close the matching beads together. Changes
  below plan-step level only touch beads.
- Follow-up work found while building goes into beads, not into `plan.md` or code comments.

## Build & Test

Prerequisites (Windows): Rust from `rust-toolchain.toml` (1.96.1, MSVC), VS Build Tools with
the C++ workload, Node 24 + npm, WebView2, and the Tauri CLI
(`cargo install tauri-cli --version "^2.12" --locked`). Run everything from the repo root; run
`npm --prefix ui ci` once after cloning.

**One command, before every commit:** `pwsh scripts/check.ps1`. It runs `cargo fmt --all
--check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
`npm --prefix ui run typecheck` and `npm --prefix ui test`. It stops at the first failure.
Healthy output ends with:

```
==> ui test
...
ALL CHECKS PASSED
```

**In Claude Code, the `gatekeeper` subagent runs the checks** (`.claude/agents/gatekeeper.md`,
on Haiku). Delegate to it after each meaningful step and before every commit, instead of
running the checks in the main context. Give it a scope for a subset (for example "`cargo
test -p tt-core --test read_models`"), or none for the whole gate. Paste its report as the
evidence. It never edits, fixes or commits anything, and it doesn't re-run a failure to get
green. Fixing stays with the main session. Other agents run `pwsh scripts/check.ps1`
themselves.

| Task | Command | Healthy result |
|---|---|---|
| Rust tests, one crate | `cargo test -p tt-core` (or `tt-store`, `tt-app`, `tt-cli`, `timetracker-app`) | `test result: ok. N passed; 0 failed` for every test binary |
| Regenerate TS bindings | `UPDATE_BINDINGS=1 cargo test -p timetracker-app bindings` | rewrites `ui/src/bindings.ts`; commit it. A stale file fails `bindings_up_to_date` |
| Run the app (dev) | `cargo tauri dev --config src-tauri/tauri.dev.conf.json` | the window "ThorSoft TimeTracker" of the `target\debug` process (product name "ThorSoft TimeTracker (dev)"); hotkey `Ctrl+Alt+Shift+Space`; runs next to the installed app |
| Installer | `cargo tauri build` | `Finished 1 bundle at: ...\target\release\bundle\nsis\ThorSoft TimeTracker_0.1.0_x64-setup.exe` |

Gotchas:
- Always pass `--config src-tauri/tauri.dev.conf.json` to `cargo tauri dev`. Without it the
  dev build has the installed app's identity: single-instance forwards to the installed app,
  and the dev build exits.
- `src-tauri/build.rs` links `windows-app-manifest.xml` into every target. Without it,
  `cargo test -p timetracker-app` fails with `STATUS_ENTRYPOINT_NOT_FOUND`.
- Release and dev profiles use `panic = "abort"` (crash-only, plan.md). Tests still unwind.
- `tt-cli` needs `--db <scratch path>` and must never be pointed at the live database.

## Architecture Overview

Single-user Windows tray app: Rust (Tauri 2) + Vue 3/TS + SQLite. Full design in `spec.md` §2;
`R<n>` refers to its requirements. Style: functional core / imperative shell (Bernhardt), the core
shaped as a Decider (Chassaing), determinism in the spirit of TigerBeetle. Goal: the app can be
driven equally by the UI, the hotkey and automated tests. The decisions behind this design are
recorded as OQ-3 to OQ-9 in `spec.md` §4; check them before changing the code they affect.

### Layout and dependency direction

```
crates/tt-core   pure domain: types, Decider (decide/evolve), invariants, rounding, read models
                 (incl. quick-switch matching)
crates/tt-store  SQLite: migrations, load State, apply Events in one transaction, backup/export
crates/tt-app    the single write path: State + Store behind one lock, executes Commands,
                 publishes Events
crates/tt-cli    headless driver for scenario tests, on scratch databases only
src-tauri/       GUI driver: Tauri commands, tray, global hotkey, autostart, clipboard,
                 UI DTO types (specta)
ui/              Vue 3 + TS; calls Tauri commands only, never touches the DB
```

Dependencies point inward only: `src-tauri, tt-cli → tt-app → tt-store → tt-core`.
`tt-core` depends on no IO crate (no Tauri, no rusqlite, no `std::fs`/`std::net`).
No port traits: tests use a temporary SQLite database, not fakes.

### tt-core: deterministic functional core

- **Decider shape.** `decide(&State, &Ctx, &Command) -> Result<Vec<Event>, DomainError>` and
  `evolve(State, &Event) -> State`. Every change to State is an Event, except restore (see
  tt-app). Events are in-memory descriptions of change; the database stores the resulting
  State (OQ-3). Undo (R10) is an inverse command.
- **Everything is an input.** `Ctx { now, tz }` is built by the driver. `decide` uses `tz` only
  to convert typed local times (R10, R11) to UTC; an ambiguous or nonexistent time is rejected
  (OQ-6). IDs come from counters in `State`. Forbidden
  in tt-core: reading the system clock or OS time zone, randomness, env vars, file/network IO,
  threads, global mutable state. Use `BTreeMap` in State for stable serialisation. Same inputs →
  identical outputs.
- **Whole world in memory.** `State` holds everything, including the counters, and is loaded at
  startup. The core enforces the invariants (R1, R2, R4, R12); the SQLite constraints are a
  backstop. `check_invariants()` runs after every `evolve` in debug builds and tests, and on every
  load in all builds.
- **Keep a prefetch migration open (OQ-9).** `State` = `Catalog` (small, always fully loaded) +
  `Entries` (the only part that grows). `decide` reaches entries only through named queries SQL
  could answer with an index (running, overlapping a range, recent combos), never by iterating
  the map. Read models take the entries they need as input, not the whole `State`. Events are
  self-contained: tt-store applies them without reading `State`. No rule may need all entries.
- **Plain values at the boundary.** `Command`, `Event` and `State` are plain `serde` data. UI DTOs
  live in src-tauri, never in tt-core.
- **Reads are pure functions** over the `Catalog` and the entries they need, plus `now`, `tz` and
  the date, range or query: daily overview, statistics, billing error, quick-switch matching. A
  running entry counts up to `now`. They don't go through `decide` and are never stored.
- **Faux-O style.** Newtypes with validating constructors (parse, don't validate). Make illegal
  states unrepresentable (e.g. a rounding unit can only be a divisor of 60). Methods return new
  values. Mutation never leaks out of a function.

### tt-app: the imperative shell

- `execute(Ctx, Command) -> Result<Vec<Event>, AppError>`. The driver builds `Ctx`; tt-app never
  reads the clock or time zone. Under one lock: `decide` → `evolve` a copy into a candidate
  State → one transaction that applies the events in order → commit → swap in the candidate →
  publish the events. A panic in `evolve` happens before anything is written. The database is
  the authority: on any store error, discard the candidate and reload State from the database
  (a failed commit's outcome can be unclear); if the reload fails, accept no writes. Reads take
  the same lock.
- Switch order: stop the running entry first, then insert the new one (uniqueness is checked per
  statement).
- **Restore (R23)** is the one exception to "every change is an Event". Under the same lock it
  backs up the current database, replaces the file from the chosen backup, reloads State and
  checks its invariants.
- **Startup order:** take the exclusive lock → backup if a database exists → migrations → load
  State and check invariants → start the drivers. The backup must come before migrations.
- One process per database file (exclusive lock). `tt-cli` requires `--db <path>` and never opens
  the live database.
- **No business rules outside tt-core.** If tt-app, a driver or the UI needs a business-rule
  `if`, move it into tt-core.

### Drivers

`src-tauri` and `tt-cli` only translate input → `Command` and result → output. Every feature
must be reachable without the UI (through `tt-app`, and so from tests and `tt-cli`). Quick-switch
matching is a tt-core read model, not UI code.

### Testing seams (details: stage 4)

- tt-core: given State / when Command / then Events-or-error tests. Model-based property tests:
  random Command sequences with a simulated clock and time zone, invariants checked after every
  step.
- tt-store + tt-app: the same sequences against a temporary SQLite DB. After a reload from disk,
  the serialised State must equal the in-memory State.
- tt-cli: end-to-end scenario scripts against a temporary DB.

## Conventions & Patterns
