# statusLine indicator

Claude Code can show the active profile in its in-session status bar.
`claude-shim` writes a `statusLine` block into a profile's `settings.json`,
so every session under that profile renders:

```
Current profile: <name>
```

This is Claude Code's own status bar inside the session —
distinct from the shell [prompt indicator](/guide/prompt-indicator),
which marks your terminal prompt.

## Enable it on a new profile

Pass `--statusline` to `profile new`:

```sh
claude-shim profile new work --statusline
```

## Set it on an existing profile

`profile statusline` configures a profile that already exists.
Without `--profile` it targets the profile active in the current directory —
the same [resolution](/guide/resolution) the shim uses,
so it fails if no profile is active there.

```sh
# Built-in preset, on the active profile (or name one with --profile):
claude-shim profile statusline --preset profile-indicator
claude-shim profile statusline --profile work --preset profile-indicator

# Or supply your own statusLine command:
claude-shim profile statusline 'echo "🔧 work"'

# Replace an existing statusLine:
claude-shim profile statusline 'echo "🔧 work"' --force
```

Other keys in `settings.json` are preserved.
If a `statusLine` is already set,
the command fails rather than overwrite it —
pass `--force` to replace it.
