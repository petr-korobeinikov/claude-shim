## Why

The `colored` preset shows what a session costs per call - model and effort - but not what it has spent: neither its tokens nor the plan's limits. Claude Code passes the limits on stdin as `rate_limits.five_hour` and `rate_limits.seven_day`, and the path of the session's transcript as `transcript_path` ([statusline docs](https://code.claude.com/docs/en/statusline)); stdin holds no session token total. The operator's shell prototype, `statusline-test.sh` in profile `personal`, renders `Profile: personal · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Week: 41% · Week resets: in 2d 5h` from them and runs today instead of a shipped preset.

Complex: two data sources beyond stdin - the transcript JSONL and the clock - and an undocumented transcript format. Implemented on `feature/add-full-statusline-presets`.

Builds on `develop` alone.

Subject: `feat: add full and full-week statusline presets`

Decisions:

- Two presets, `full` and `full-week`; `full-week` is `full` plus the countdown to the weekly limit's reset. (operator; names without `reset`)
- Every segment carries its own label - `Effort: high`, not `(high)` - joined by ` · `. (operator)
- `Session: n%` and `Week: n%` are part of `full`, not a preset of their own. (operator)
- The countdown reads `Week resets: in 2d 5h`. (operator)
- Both presets are colored with the `colored` palette and print plain under `NO_COLOR`; no plain twins. (operator)
- Tokens are summed per API call as input + cache writes + output, keyed by `message.id`, cache reads left out: with them the prototype read 1.3M for a 70k session, since every call re-reads the context. Shown with a tilde. (operator)
- A limit's segment is left out when its window is absent - the operator's plan has no 5-hour window - never printed as `0%`. (operator)
- Tokens, limits and the countdown carry no color of their own; labels and ` · ` are dim. *Arguable.*
- Under a minute left the countdown reads `in 0m`; the segment is left out once `resets_at` has passed. *Arguable.*
- Subagent transcripts are not counted, and `/clear`, which starts a new transcript, resets the count. *Arguable.*

Assumption: an assistant line of the transcript JSONL carries `message.id` and `message.usage` with `input_tokens`, `cache_creation_input_tokens` and `output_tokens`, and one call spans several lines sharing an id. The docs do not describe the format; the prototype relied on it. Task 1.1 checks it.

## What Changes

- `claude-shim statusline` accepts `full` and `full-week`.
- `claude-shim profile statusline --preset` accepts `full` and `full-week`, writing `claude-shim statusline <preset>`.
- `NO_COLOR` covers the new presets.
- Docs: `docs/guide/statusline.md` gains both presets and their segments.

Public contract: adds two `statusline` presets and two `--preset` values; nothing existing changes or breaks, and no stored state migrates. A profile switches with `claude-shim profile statusline --preset full --force`.

## Capabilities

### New Capabilities

- `session-usage`: what a session has spent - its token count from the transcript and the countdown to the weekly reset. Split from `statusline`, whose delta would run past its budget.

### Modified Capabilities

- `statusline`: the presets the command renders and installs, `NO_COLOR`, and the `full` and `full-week` lines with their limit segments.

## Impact

- `src/statusline/`: the two presets, stdin's new fields, the token count over the transcript, the countdown.
- `src/profile/mod.rs`: two `StatusLinePreset` variants.
- `e2e/statusline.bats`, `e2e/profile_statusline.bats`; `docs/guide/statusline.md`.
- No new crate.
