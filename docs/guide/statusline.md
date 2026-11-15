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

`compact`, `labeled` and `colored` take the model and the effort
from the JSON Claude Code passes the command on stdin —
`model.display_name` and `effort.level` —
and leave out a segment that has no data:
without an effort, `labeled` prints `Profile: work · Model: Haiku 5.5`.
The profile is the one the session runs under,
read from `CLAUDE_CONFIG_DIR`;
when that is not a profile directory of the shim,
it is the profile active in the current directory.

### Colors

`colored` paints the model and the effort louder the more they cost;
labels, ` · ` and parentheses are dim.

| Model | Color | Effort | Color |
| --- | --- | --- | --- |
| Fable | bold orange | `max` | bold red |
| Opus | yellow | `xhigh` | yellow |
| Sonnet, unknown | plain | `high` | green |
| Haiku | dim | any other | dim |

The model's family is looked up in `model.id` first, then in `model.display_name`.
With [`NO_COLOR`](https://no-color.org) set and non-empty,
`colored` prints the `labeled` line.

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
`compact`, `labeled` and `colored` write `claude-shim statusline <preset>`,
which reads Claude Code's JSON on stdin, prints one line and exits 0;
`profile-indicator` writes a shell snippet calling `claude-shim profile current`.
Either way `claude-shim` must be on the `PATH` of the shell Claude Code runs it in,
or the line stays blank.

To preview a line by hand:

```sh
echo '{"model":{"display_name":"Opus 5.5"},"effort":{"level":"high"}}' \
  | claude-shim statusline labeled
```
