# Play: skills as institutional knowledge

## Goal

Turn institutional knowledge that must be applied **the same way every time** (security rules,
API design, brand, compliance) into skills, so the policy is applied while the code is written,
not only found in review. Skills are versioned, shared, and updated centrally.

Not everything belongs in a skill. Guidance that belongs to one component goes in
`CLAUDE.md`, and guidance for one task goes in the prompt.

## Entry criteria

- One policy with a **named owner** and a written source of truth.

## Steps

1. **Pick one piece of knowledge that is enforced inconsistently today.**
2. **Write it as a skill:** a folder with a `SKILL.md`. An engineer drafts it from the owner's
   source, with Claude's help.
   - Frontmatter `name` and `description`. The description says **when** the skill applies,
     with concrete triggers ("creating or changing external endpoints, reviewing API code,
     generating OpenAPI specs").
   - Body: numbered, checkable rules. Add a **verification step** where possible ("run
     `<check script>` and include its output in the summary").
3. **Place it:**
   - `.claude/skills/<name>/` ships it with this repo.
   - A plugin distributes it across the whole organization.
4. **Test triggering.** Ask for the task in several different phrasings (in fresh sessions or
   subagents), and confirm the skill loads every time. If it doesn't, tune the description.
5. **Get the owner's sign-off.** When the policy changes, update the skill, and the owner
   reviews the change like code.
6. Engineers pick up the new version in their next session.

## Back up must-hold rules with hooks

A skill is advisory and makes violations rare. A hook or CI check is deterministic and makes
them nearly impossible. For any rule that must **always** hold:

- Add a `PostToolUse` hook on `Edit|Write`, scoped to the relevant paths, that runs the same
  check script the skill names. When the check fails, the hook exits with code **2**, so the
  message reaches Claude and Claude fixes it.
- Add a **required CI check** on PRs as well. It catches writes made through shell commands,
  which the edit hook doesn't see.

Example `.claude/settings.json` fragment:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{ "type": "command", "command": "\"$CLAUDE_PROJECT_DIR\"/scripts/check-api-policy.sh" }]
      }
    ]
  }
}
```

The script reads the hook's JSON from stdin (`tool_input.file_path`). It exits 0 if the file is
out of scope or passes. If the file fails, it writes the reason to stderr and exits 2.

Guidance for build-time hooks:

- Keep them **fast** and scoped to the changed file.
- Good uses: protecting paths (generated code, frozen packages), running formatters and
  linters, keeping secrets out of diffs, and enforcing must-hold policy.
- Run heavy checks (the full test suite) at commit or PR time, not on every edit.
- **No human approval prompts in build hooks.** Those belong at the Stage 5 gates. A person on
  the critical path blocks parallel sessions.

## Exit criteria

- [ ] The skill triggers reliably across varied phrasings
- [ ] The policy owner approved the skill
- [ ] It lives in the repo or a plugin, under version control
- [ ] Every must-hold rule has a deterministic backstop (a hook and/or a required CI check)

## Anti-patterns

- Relying on a skill alone for a rule that must always hold
- Skills for guidance that belongs in `CLAUDE.md` or a prompt
- Slow checks in per-edit hooks, or approval prompts in build hooks
- Skill text that has drifted from the official policy

## Metrics

- Leading: the time from a policy approval to the updated skill being merged.
- Lagging: PR review findings that cite the policy. These should fall toward zero. If they
  don't, the skill isn't triggering or its text has drifted.
