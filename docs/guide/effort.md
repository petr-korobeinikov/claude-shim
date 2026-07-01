# Effort level

A profile or a project binding can pin a Claude Code effort level.
Before exec'ing `claude`, the shim exports it as `CLAUDE_CODE_EFFORT_LEVEL`,
so a session starts at that effort without editing `settings.json`.

The levels are `low`, `medium`, `high`, `xhigh`, `max`, and `auto`.

## Pinning an effort

```sh
# A profile's default — the active profile, or a named one:
claude-shim profile effort high
claude-shim profile effort max --profile work

# Seed it when creating the profile:
claude-shim profile new personal --effort high

# This directory's binding — the project or workspace marker in effect:
claude-shim profile effort xhigh --local

# Or when first binding a directory to a profile:
claude-shim profile use work --effort max
```

`--local` and `--profile` are mutually exclusive.
`--local` rewrites the marker the shim resolves here
(the nearest `.claude/claude-shim.json` or `.claude-shim.json`),
keeping its profile name and printing the file it touched;
it fails when no binding is in scope.

## Precedence

The effort for a session is chosen highest-first:

1. `CLAUDE_CODE_EFFORT_LEVEL` already set in the shell — never overwritten.
2. The `effort` of the nearest project or workspace marker — only that marker;
   if it has none, the profile default is used, not a marker farther up.
3. The resolved profile's default (`profiles/<name>/claude-shim.json`).
4. Otherwise left unset, and Claude Code uses its own default.

::: tip
`profile effort <level>` without `--local` sets the profile default.
If the current directory has a project or workspace override,
that override wins — use `--local` to change the effort in effect here.
:::
