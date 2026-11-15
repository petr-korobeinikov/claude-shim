# statusLine indicator

Claude Code can show the profile a session runs under in its in-session status bar.
`claude-shim` writes a `statusLine` block into a profile's `settings.json`,
so every session under that profile renders the line its preset gives.

This is Claude Code's own status bar inside the session —
distinct from the shell [prompt indicator](prompt-indicator.md),
which marks your terminal prompt.

## Presets

| Preset | Line |
| --- | --- |
| `profile-indicator` | `Current profile: work` |
| `compact` | `work · Opus 5.5 · high` |
| `labeled` | `Profile: work · Model: Opus 5.5 (high)` |
| `colored` | the `labeled` line, colored by cost |
| `full` | `Profile: work · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Session: 24% · Week: 41%` |
| `full-week` | the `full` line, then `· Week resets: in 2d 5h` |

`compact`, `labeled`, `colored`, `full` and `full-week` take the model and the effort
from the JSON Claude Code passes the command on stdin —
`model.display_name` and `effort.level` —
and leave out a segment that has no data:
without an effort, `labeled` prints `Profile: work · Model: Haiku 5.5`.
The profile is the one the session runs under,
read from `CLAUDE_CONFIG_DIR`;
when that is not a profile directory of the shim,
it is the profile active in the current directory.

### Tokens and limits

`full` and `full-week` also show what the session has spent:

| Segment | Source |
| --- | --- |
| `Tokens: ~66.9k` | the transcript at `transcript_path` |
| `Session: 24%` | `rate_limits.five_hour.used_percentage`, rounded |
| `Week: 41%` | `rate_limits.seven_day.used_percentage`, rounded |
| `Week resets: in 2d 5h` | `rate_limits.seven_day.resets_at` minus now |

The token count is approximate:
it sums each API call's input, cache writes and output once,
and leaves cache reads out —
every call re-reads the whole context,
so with them a 70k session would read over a million.
It counts the transcript of the session itself, not its subagents',
and `/clear` starts a new transcript,
so the count starts over.
Without a transcript the segment is left out.

[Claude Code passes](https://code.claude.com/docs/en/statusline) `rate_limits` to Claude.ai subscribers only,
after the session's first response;
without it, or without one of its windows,
that segment is left out rather than printed as `0%`.
The countdown reads `<d>d <h>h`, `<h>h <m>m` or `<m>m`,
and is left out once the reset has passed.

### Colors

`colored`, `full` and `full-week` paint the model and the effort louder the more they cost;
labels, ` · ` and parentheses are dim,
and the tokens, limits and countdown plain.

| Model | Color | Effort | Color |
| --- | --- | --- | --- |
| Fable | bold orange | `max` | bold red |
| Opus | yellow | `xhigh` | yellow |
| Sonnet, unknown | plain | `high` | green |
| Haiku | dim | any other | dim |

The model's family is looked up in `model.id` first, then in `model.display_name`.
With [`NO_COLOR`](https://no-color.org) set and non-empty,
`colored` prints the `labeled` line,
and `full` and `full-week` print their lines without color.

## Enable it on a new profile

Pass `--statusline` to `profile new`;
it installs `profile-indicator`:

```sh
claude-shim profile new work --statusline
```

## Set it on an existing profile

`profile statusline` configures a profile that already exists.
Without `--profile` it targets the profile active in the current directory —
the same [resolution](resolution.md) the shim uses,
so it fails if no profile is active there.

```sh
# Built-in preset, on the active profile (or name one with --profile):
claude-shim profile statusline --preset colored
claude-shim profile statusline --profile work --preset compact

# Or supply your own statusLine command:
claude-shim profile statusline 'echo "🔧 work"'

# Replace an existing statusLine, e.g. to switch presets:
claude-shim profile statusline --preset labeled --force
```

Other keys in `settings.json` are preserved.
If a `statusLine` is already set,
the command fails rather than overwrite it —
pass `--force` to replace it.
Each profile keeps its own `statusLine`,
so profiles can run different presets.

## How a preset renders

The preset writes a command,
which Claude Code runs on every status bar refresh:
`compact`, `labeled`, `colored`, `full` and `full-week` write `claude-shim statusline <preset>`,
which reads Claude Code's JSON on stdin, prints one line and exits 0;
`profile-indicator` writes a shell snippet calling `claude-shim profile current`.
Either way `claude-shim` must be on the `PATH` of the shell Claude Code runs it in,
or the line stays blank.

To preview a line by hand:

```sh
echo '{"model":{"display_name":"Opus 5.5"},"effort":{"level":"high"}}' \
  | claude-shim statusline labeled
```
