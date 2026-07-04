# Profiles

Create the profile directory,
then point projects at it via a marker file.

## Creating a profile

```sh
# --default sets the global default;
# --statusline adds the in-session statusLine indicator.
claude-shim profile new personal --default
claude-shim profile new work --statusline
claude-shim profile new client-acme
```

Claude Code initializes the profile contents (settings, credentials, history) on first launch.
`profile new` fails loud rather than overwrite an existing profile —
remove the old profile directory before retrying.

::: tip
`profile new` seeds a `CLAUDE.md` into the profile directory.
Because the shim points `CLAUDE_CONFIG_DIR` there,
Claude Code loads it as user memory and learns its config lives in the profile dir —
not in `~/.claude` (which is overridden). A project's own `.claude/` is unaffected.
:::

See [statusLine indicator](/guide/statusline) to show the active profile
in Claude Code's status bar (enable at creation with `--statusline`).

See [Effort level](/guide/effort) to pin a Claude Code effort level
per profile or per project.

## Binding a directory

```sh
# Point a single project at a profile (writes .claude/claude-shim.json):
cd ~/Workspace/my-project
claude-shim profile use work

# Or, for a whole workspace of projects (writes .claude-shim.json in the root):
cd ~/Workspace/work
claude-shim profile use work --workspace
```

`profile use` fails loud rather than overwrite an existing marker —
remove the old marker file before retrying.

## Listing profiles

`claude-shim profile list` prints every profile directory one per line,
appending `(default)` for the global default
and `(active)` for the one that would resolve in the current directory:

```sh
$ claude-shim profile list
client-acme
personal (default, active)
work
```

`claude-shim profile current` in a directory with a valid profile prints the name and exits 0;
without a profile, it prints nothing and exits 0;
if a project or workspace marker points at a non-existent profile directory, it warns on stderr and exits 2.

## Deleting a profile

`claude-shim profile delete <name>` removes a profile directory.
It is irreversible,
so it requires an explicit `--yes`;
without it, the command prints what would be removed and exits non-zero,
changing nothing.
Deleting a missing profile fails loud.
If the deleted profile was the global default,
the `default-profile` marker is cleared too.
Resolution markers in other directories are left untouched.
A directory still bound to the deleted profile fails loud on the next `claude` launch —
its profile directory is gone —
until you recreate the profile or rebind with `claude-shim profile use`.

```sh
$ claude-shim profile delete work
would remove profile 'work' at …/claude-shim/profiles/work
hint: re-run with `claude-shim profile delete work --yes`

$ claude-shim profile delete work --yes
deleted profile 'work' at …/claude-shim/profiles/work
```

::: warning macOS: the Keychain entry is left behind
Deleting a profile removes its directory,
but not its macOS Keychain login entry —
that entry is keyed by a hash of the profile's `CLAUDE_CONFIG_DIR` path,
which `claude-shim` never manages.
Once the directory is gone the orphaned entry is unreachable and harmless —
the same Keychain limitation noted for [migration](/guide/migration).
:::

## Profile resolution

Which marker a directory resolves to —
and how the global default is chosen when there is none —
is covered in [Profile resolution](/guide/resolution).
