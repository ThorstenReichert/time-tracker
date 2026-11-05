# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project. It is
the single source: `CLAUDE.md` only imports it (`@AGENTS.md`), so edit this file, not that one.

## Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations to avoid hanging on confirmation prompts.

Shell commands like `cp`, `mv`, and `rm` may be aliased to include `-i` (interactive) mode on some systems, causing the agent to hang indefinitely waiting for y/n input.

**Use these forms instead:**
```bash
# Force overwrite without prompting
cp -f source dest           # NOT: cp source dest
mv -f source dest           # NOT: mv source dest
rm -f file                  # NOT: rm file

# For recursive operations
rm -rf directory            # NOT: rm -r directory
cp -rf source dest          # NOT: cp -r source dest
```

**Other commands that may prompt:**
- `scp` - use `-o BatchMode=yes` for non-interactive
- `ssh` - use `-o BatchMode=yes` to fail instead of prompting
- `apt-get` - use `-y` flag
- `brew` - use `HOMEBREW_NO_AUTO_UPDATE=1` env var

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

**Architecture in one line:** issues live in a local Dolt DB (`.beads/embeddeddolt/`, gitignored). It has no Dolt remote and isn't exported to git, so beads issues and memories exist only on this machine, not on GitHub (see "GitHub remote, local workflow" below).

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates and commit as part of session close. A current "do not commit" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git by active profile**: report `git status` and the proposed commit; wait for
   approval unless the current request authorizes the commit. Push only on the owner's
   explicit request (see "GitHub remote, local workflow" below).
5. **Hand off** - Summarize changes, validation, issue status, and any blocked commit

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit without clear authority from the active profile or the current user request.
- If a required commit is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->

## Development process: AI-native SDLC

Changes follow the six-stage AI-native SDLC (Plan → Design → Build → Test → Deploy → Maintain),
encoded as repo-local skills in `.claude/skills/`. Start with `ai-sdlc` (overview, artifact chain,
how to tell which stage a change is in). Then use the stage skills: `ai-sdlc-1-intent` (intent.md),
`ai-sdlc-2-spec` (spec.md), `ai-sdlc-3-build` (plan.md + implementation), `ai-sdlc-4-test`,
`ai-sdlc-5-deploy` and `ai-sdlc-6-maintain`. When the user names a stage, load its skill and
follow it.

### GitHub remote, local workflow

`origin` is the public repository https://github.com/ThorstenReichert/time-tracker. The local
`master` tracks `origin/main`. No pull requests: changes are merged locally into `master` and pushed to `main`.
Don't propose PRs or record their absence as a gap. Rulesets on `main` require signed commits
and forbid force pushes and deleting `main`. CI: the GitHub Actions
workflow `.github/workflows/check.yml` runs the local gate `pwsh scripts/check.ps1` on every
push to `main`. Its status check is `check`. The local checks stay the feedback loop and CI
is the backstop. Where the stage skills mention a PR, CI or branch protection, use the
equivalents below.
- **Branch.** Start each change from an up-to-date `master` (`git pull --ff-only`) on a local
  branch `feat/<slug>`, `fix/<slug>` or `docs/<slug>`. Small docs-only changes may be
  committed on `master` directly.
- **Signed commits.** The repository's git config signs every commit (`commit.gpgsign`, GnuPG)
  as `ThorstenReichert@users.noreply.github.com`. Never bypass signing or change that
  identity: `main` rejects a push with an unsigned commit.
- **The checks:** `pwsh scripts/check.ps1` (through the `gatekeeper` subagent) before every
  merge and every push.
- **Stage 5 review:** fresh-context reviewer subagents against `REVIEW.md`, `spec.md` and
  `plan.md`. Fixes go on the same branch as new commits. Docs-only changes need no reviewer
  subagents.
- **Approval and merge.** The owner gives the explicit "approved to merge", then a local
  `git merge --no-ff` of the branch into `master`. The merge commit message serves as the PR
  description: links to intent, spec and plan, the `R<n>` covered, deviations, and the check
  evidence. Then delete the merged branch (`git branch -d`).
- **Publishing.** Push only on the owner's explicit request, never as part of a merge or
  session close, because it is outward-facing: `git push origin master:main` (the branch
  names differ, so a bare `git push` fails). If CI's `check` turns red, fix it on `master`
  with a new commit; don't re-run a failed job just to get green. A release tag goes on the
  merge commit: `git tag -a v<x.y.z>`, pushed with `git push origin v<x.y.z>` on the owner's
  request. The release record (release notes, installer SHA-256) follows as a docs commit.
- Never force-push or rewrite pushed history. If `origin/main` has commits that `master`
  lacks, `git pull --ff-only` them in.
- The repository is public: commit no secrets, tokens, databases, exports or personal data.

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

**CI** (`.github/workflows/check.yml`, job `check`) runs the same `pwsh scripts/check.ps1` on
`windows-latest` after `npm --prefix ui ci`, from a clean checkout. It needs no secrets and no
`ui/dist`, because debug builds don't embed the frontend. Change the gate in `check.ps1`, not in
the workflow, so local runs and CI stay the same. The actions are pinned to commit SHAs.

| Task | Command | Healthy result |
|---|---|---|
| Rust tests, one crate | `cargo test -p tt-core` (or `tt-store`, `tt-app`, `tt-cli`, `timetracker-app`) | `test result: ok. N passed; 0 failed` for every test binary |
| Regenerate TS bindings | `UPDATE_BINDINGS=1 cargo test -p timetracker-app bindings` | rewrites `ui/src/bindings.ts`; commit it. A stale file fails `bindings_up_to_date` |
| Run the app (dev) | `cargo tauri dev --config src-tauri/tauri.dev.conf.json` | the window "ThorSoft TimeTracker" of the `target\debug` process (product name "ThorSoft TimeTracker (dev)"); hotkey `Ctrl+Alt+Shift+Space`; runs next to the installed app |
| Installer | `cargo tauri build` | `Finished 1 bundle at: ...\target\release\bundle\nsis\ThorSoft TimeTracker_<version>_x64-setup.exe` |

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

UI look (R25): follows Windows light/dark. The PrimeVue theme is `ThorSoftPreset` in
`ui/src/lib/brand.ts` (Aura with the logo colours as `primary`, `night` navy surfaces in dark).
Components use PrimeVue tokens or `--brand-text`/`--error-text` (`style.css`), not hex literals
or fixed steps like `--p-primary-800`, which vanish in one mode. The quick-switch popup has no
PrimeVue and mirrors the theme in its CSS (pinned by `brand.test.ts`). The logo is
`BrandLogo.vue`, with `thorsoft-logo-dark.svg` derived from the original (also pinned).

### Testing seams (details: stage 4)

- tt-core: given State / when Command / then Events-or-error tests. Model-based property tests:
  random Command sequences with a simulated clock and time zone, invariants checked after every
  step.
- tt-store + tt-app: the same sequences against a temporary SQLite DB. After a reload from disk,
  the serialised State must equal the in-memory State.
- tt-cli: end-to-end scenario scripts against a temporary DB.
- ui: component tests (Vitest, `@vue/test-utils`, happy-dom) next to each component, as
  `<Name>.test.ts`. The backend is faked with Tauri's `mockIPC` through `ui/src/test/backend.ts`
  (handlers per command, recorded calls, faked events); `test/mount.ts` mounts with PrimeVue
  and Pinia; `test/page.ts` finds dialogs and AutoComplete suggestions. Pure UI logic goes in
  `ui/src/lib/` with plain unit tests. No browser or native-shell tests yet (owner, 2026-10-18).
