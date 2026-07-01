# Profile resolution

Markers are discovered by walking up from `$PWD` through the project tree —
the nearest match wins.
On each ancestor directory,
`.claude/claude-shim.json` (per-project) is checked first
and takes priority over `.claude-shim.json` (workspace-wide) at the same level.
The walk stops before `$HOME`,
so a marker placed directly in `$HOME` is not picked up as a global default.

When no project marker is found, the shim resolves the profile in this order:

1. `<config dir>/claude-shim/default-profile` — a text file containing one profile name.
   Recommended way to set a global default.
   The config dir is platform-specific:
   `~/Library/Application Support/claude-shim/` on macOS,
   `${XDG_CONFIG_HOME:-~/.config}/claude-shim/` on Linux.
2. `~/.claude/` itself — used as the profile if it exists,
   for installs that pre-date claude-shim.
3. Otherwise the shim refuses to run rather than silently fall back to an arbitrary profile.

Migrating an existing `~/.claude` setup to a named profile
is covered in [Migration](/guide/migration).
