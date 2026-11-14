## Why

The one statusLine preset, `profile-indicator`, prints `Current profile: <name>` and nothing else, while a session's cost is set by its model and effort: Claude Code passes both to a statusLine command on stdin as `model.display_name` and `effort.level` ([statusline docs](https://code.claude.com/docs/en/statusline)). A shell prototype on Claude Code 2.1.311 rendered `Profile: personal · Model: Opus 5.5 (high)` from that JSON and `CLAUDE_CONFIG_DIR`, and the operator wants it, a bare variant and a cost-colored one shipped as presets.

Relatively simple: one rendering command over stdin JSON with `serde_json`, already a dependency, and three preset values; no new crate. Implemented on `develop`.

Builds on `develop` alone.

Subject: `feat: add compact, labeled and colored statusline presets`

Decisions:

- Rendering is a new command, `claude-shim statusline <preset>`, over the shell snippet with `jq` the prototype used: no `jq` on the host, and the logic is unit-tested. (operator)
- Presets are named `compact`, `labeled` and `colored`. (operator)
- The profile is the one the session runs under - `CLAUDE_CONFIG_DIR`, when it is a profile directory of the shim - over cwd resolution, which `profile-indicator` uses and which can name another profile than the session's; resolution is the fallback. (operator)
- `colored` honors `NO_COLOR` ([no-color.org](https://no-color.org)) by printing the `labeled` line. (operator)
- Colors rise with cost, model by family and effort by level, as the spec's table gives. (operator approved the prototype's palette)
- A segment with no data is left out rather than printed as a placeholder; stdin that is not a JSON object is read as empty, and the command exits 0, since Claude Code blanks the line on a non-zero exit. *Arguable.*
- `profile new --statusline` keeps installing `profile-indicator`: changing it would move the contract for no request. *Arguable.*

Assumption: a statusLine command inherits `CLAUDE_CONFIG_DIR` from Claude Code. The docs do not say so; the prototype showed the profile on 2.1.311. Task 1.1 checks it.

## What Changes

- New command `claude-shim statusline <compact|labeled|colored>`: reads Claude Code's statusLine JSON on stdin, prints one line, exits 0.
- `claude-shim profile statusline --preset` accepts `compact`, `labeled` and `colored` beside `profile-indicator`, writing `claude-shim statusline <preset>` as the `statusLine` command.
- Docs: `docs/guide/statusline.md` gains the presets.

Public contract: adds one command and three `--preset` values; nothing existing changes or breaks, and no stored state migrates. A profile switches with `claude-shim profile statusline --preset <preset> --force`.

## Capabilities

### New Capabilities

- `statusline`: the statusLine presets the shim installs and the line each renders from Claude Code's stdin JSON.

### Modified Capabilities

None.

## Impact

- `src/cli/mod.rs`, `src/lib.rs`: the `statusline` command.
- `src/profile/mod.rs`: three `StatusLinePreset` variants.
- New core module for rendering; its glue reads stdin and the environment.
- `e2e/`: the command and the presets; `docs/guide/statusline.md`.
