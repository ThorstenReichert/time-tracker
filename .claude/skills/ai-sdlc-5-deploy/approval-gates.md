# Play: hooks as approval gates

## Goal

Keep the required human approvals in place when an agent acts. Each gate is a hook that
returns **allow**, **ask** (pause for a person) or **deny**. The play is filed under Deploy, but
gates apply wherever Claude acts. Examples: during Build, block migration or infrastructure
edits that have no change ticket. During Test, block test edits in a fix task.

## Entry criteria

- A written list of the approvals the change process requires.

## Steps

1. **Define the gates.** Engineering leadership, change management and compliance list the
   approvals that must survive: change sign-off, release authorization, edits to protected
   paths, and so on.
2. **Encode each gate as a hook.** A script that runs **before** Claude acts (`PreToolUse`)
   and returns a decision.
3. **Place the hooks by owner:**
   - Team gates go in `.claude/settings.json`, in Git.
   - **Non-negotiable gates go in managed settings**, which the platform or IT team owns and
     engineers can't disable. Note: with `allowManagedHooksOnly`, project hooks don't run, so
     the gate must live in the managed file's own `hooks` block.
4. **Make blocks explain themselves.** A deny states the reason **and the route to approval**.
   Deny reasons go to Claude, so Claude can relay them. Ask reasons go to the person.

## Hook configuration (example)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "mcp__deploy-.*__release",
        "hooks": [{ "type": "command", "command": "\"$CLAUDE_PROJECT_DIR\"/scripts/release-gate.sh" }]
      }
    ]
  }
}
```

The gate script reads the tool call JSON from stdin and prints its decision:

```json
{ "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Production release needs change ticket CHG-#### approved for version <sha>. Request approval at <route>."
} }
```

Example gate logic:

- **dev:** allow.
- **staging:** allow for pipeline releases from `main`, and ask otherwise.
- **prod:** require a ticket in the right format and a version that looks like a commit SHA,
  then query the change system. Allow only if the ticket is approved **for that exact
  version**, and log the approver. Otherwise deny, with the route to approval.
- **any other tool:** deny by default ("add a rule first").

## Design points

- Claude supplies inputs such as `version` and `ticket`. The gate is only sound if the agent
  **can't write to the approval records** the gate reads.
- In non-interactive runs nobody can answer `ask`, so it becomes a deny. That is why there is a
  pipeline-on-`main` exemption.
- Gates must **fail closed**: if the hook errors, the action is denied.
- Known limit: environment variables an engineer sets in their own session (for example
  `GITHUB_REF`) can bypass only the prompt that engineer would have answered themselves.

## Managed settings baseline (regulated environments)

Use this as a starting point and adjust it to the data classification:

- Permissions: deny reads of `.env` and secrets directories, and deny `WebFetch`, `curl` and
  `wget`. Allow git and the `make build/test/lint` targets, so the deny list doesn't cause
  prompt fatigue.
- Lock permissions: disable bypass mode, and apply only the managed permission rules.
- Sandbox: required, refuse to start without it, no unsandboxed retries. Network egress goes
  only to an allowlist (internal git, the package registry).
- Credentials: deny `~/.ssh` and the cloud credential files, and strip tokens from sandboxed
  environments.
- `allowManagedHooksOnly`, plugins only from an approved marketplace, managed MCP servers
  only, and a minimum Claude Code version.

## Exit criteria

- [ ] Every gate on the approval list is expressed as a hook
- [ ] Non-negotiable gates live in managed settings
- [ ] Blocks explain the route to approval, and unknown tools are denied by default
- [ ] Gates fail closed, and decisions are logged with timestamps
- [ ] The managed controls (sandbox, version floor) are verified in effect

## Anti-patterns

- Non-negotiable gates only in project settings (managed-only mode silently disables them)
- An agent that can write to the records its gate reads
- `ask` in unattended pipelines
- Deny messages without a route to approval
- Unknown tools allowed by default, or hooks that fail open
- Deny rules so broad that engineers get prompt fatigue

## Metrics

- Leading: the wait time at each gate (hook decisions with timestamps, through OpenTelemetry).
- Lagging: gate violations that reach production, before and after hooks.
