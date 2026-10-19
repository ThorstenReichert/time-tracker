# Review policy

Owner: Thorsten Reichert (tech lead). Applies to every change merged into `master`. The
reviewer runs in a fresh context: never the session that wrote the code. Findings never
approve or block on their own; the owner gives the explicit "approved to merge".

Read `spec.md`, `plan.md` (including its Deviations section) and `CLAUDE.md` before the diff.
Every finding is tagged with its pass and its severity, and names the file and line.

## Passes
1. **Bugs:** logic errors, edge cases (half-open intervals, the running entry, midnight,
   DST gap and fold), error handling, concurrency (the one `App` lock, the bridge and tray
   threads), resource leaks, and UI state that goes stale (late answers, missed
   `data-changed` reloads).
2. **Security:** the Tauri capabilities and CSP stay minimal (no permission a window doesn't
   use, scripts only from `'self'`). The input from the UI and the CLI is validated in
   tt-core. No SQL built from strings. No secrets or personal data in logs or errors. New
   dependencies are justified and pinned as `plan.md` says.
3. **Compliance:** the diff matches `spec.md` (R numbers), `plan.md` (files, order, proof)
   and the architecture in `CLAUDE.md`:
   - tt-core stays pure. No clock, time zone, randomness, env vars, file/network IO,
     threads or global mutable state. `BTreeMap`/`BTreeSet`, not hash maps. `#[allow]` on
     the purity lints only in `testing.rs`.
   - Dependencies point inward only: `src-tauri, tt-cli → tt-app → tt-store → tt-core`.
   - No business-rule `if` in tt-app, the drivers or the UI. Such rules belong in tt-core.
   - Every change to State is an Event, and events are self-contained: tt-store never reads
     State. `decide` reaches entries only through named queries (OQ-9), and read models take
     entries as input, never `State`.
   - The write path order holds: decide → evolve a candidate → one transaction → commit →
     swap → publish. On a store error, State is reloaded from the DB.
   - Startup order: lock → backup → migrations → load and check invariants.
   - The OQ-3 to OQ-9 decisions in `spec.md` §4 are respected.
   - Every feature can be reached without the UI (tt-app, tt-cli).
   - Each requirement has the proof `plan.md` names. New UI views have a `<Name>.test.ts`.
   - Every departure from `plan.md` is recorded in its Deviations section.

## Severity
- **Important:** behaviour breaks, data loss or corruption, a security weakness, a breach of
  the spec, the plan or the architecture rules above, or missing proof for a requirement.
- **Nit:** style, naming, minor readability. **At most 5 nits per review.** Summarize the rest
  as a count.

End every review with a tally that a script can read, e.g. `important=2 nit=3`.

## Skip
- Generated or binary files: `ui/src/bindings.ts` (checked by `bindings_up_to_date`),
  `Cargo.lock`, `ui/package-lock.json`, `src-tauri/icons/*`, `.beads/`.
- The process files: `.claude/`, `.agents/`.
- What `pwsh scripts/check.ps1` already enforces: rustfmt, clippy with `-D warnings`
  (including the tt-core purity lints), and the `vue-tsc` type check.

## Also flag
- Changes that leave `CLAUDE.md`, `AGENTS.md` (their architecture sections must stay
  identical), `spec.md` or `plan.md` stale.
- A mistake already flagged in an earlier review (two-strike rule: it goes into `CLAUDE.md`).
