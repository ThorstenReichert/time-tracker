# Play: the CLAUDE.md

## Goal

Give Claude the context a new team member would need on day one: commands, conventions,
architecture, and the mistakes the team sees most often. Claude reads the file at the start of
every session. The whole team maintains it, and it changes whenever Claude makes a mistake
again.

## Steps

1. **Generate a draft.** Run `/init` (or explore the repo) to get a first version.
2. **Trim it** to what a new joiner needs on day one:
   - **Commands:** build, test (unit and integration, with prerequisites such as Docker),
     lint (and whether CI enforces it), and how to run each service. Include an example of
     healthy output (see Stage 4).
   - **Conventions:** the language and framework for each component, and rules such as "every
     endpoint has a test" or "dates cross the API as ISO 8601".
   - **Architecture:** what each directory holds, and the boundaries ("only the backend talks
     to system X"; "X allows 50 req/s, so cache and never poll").
   - **Things Claude gets wrong:** concrete do / don't rules, each with its reason ("don't log
     request or response bodies, because they may contain PII"; "don't bump dependency
     versions, because the platform team owns them").
   - **Definition of done / verification** (Stage 4): which commands must pass, and that their
     output must be shown.
3. **Commit it at the repo root.** It is shared by the team and reviewed like code. Code owners
   approve changes to it.
4. **Two-strike rule:** when Claude makes the same mistake twice, the correction goes into the
   file. This includes mistakes found in PR review (Stage 5).
5. **Keep it under about a page.** Claude reads all of it every session, so stale content wastes
   context and dilutes the rules that matter. Move long procedures into skills and point to
   them.

## Rules for editing

- Before you add something, ask whether a new joiner needs it on day one, and whether Claude
  would get it wrong without it. If not, leave it out.
- Prefer rules that come with a reason. The reason lets Claude handle edge cases.
- When the code changes a command, path or boundary, update `CLAUDE.md` in the same change.
- If the repo also has `AGENTS.md` or other agent instruction files, keep the substantive
  content in sync, or point one to the other.

## Exit criteria

- [ ] Commands, conventions, architecture and "things Claude gets wrong" are present and correct
- [ ] Every command was actually run once and works
- [ ] About a page or less, with no stale content
- [ ] Committed, and changes go through review

## Metrics

- Leading: how often Claude repeats a mistake that `CLAUDE.md` should have prevented.
- Lagging: the time from a new team member's start to their first merged PR.
