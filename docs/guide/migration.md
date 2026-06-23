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
