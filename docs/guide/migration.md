# Migration

To switch from the legacy `~/.claude` setup to a named profile,
do it by hand:

```sh
# macOS
mv ~/.claude "$HOME/Library/Application Support/claude-shim/profiles/default"
echo default > "$HOME/Library/Application Support/claude-shim/default-profile"

# Linux
mv ~/.claude "${XDG_DATA_HOME:-$HOME/.local/share}/claude-shim/profiles/default"
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/claude-shim"
echo default > "${XDG_CONFIG_HOME:-$HOME/.config}/claude-shim/default-profile"
```

::: info
A `claude-shim migrate` command will land in a later release.
:::

## Marker format

Profile markers are JSON —
`.claude/claude-shim.json` (project) and `.claude-shim.json` (workspace),
each holding at least a profile `name`.
Earlier alpha builds wrote plaintext markers;
those are no longer read,
so a directory that still has one silently stops resolving to that profile until you recreate it.
Recreate it with `claude-shim profile use <name>` (add `--workspace` for a workspace marker).
