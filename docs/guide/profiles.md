# Profiles

Create the profile directory,
then point projects at it via a marker file:

```sh
# Create the profile (--default also sets the global default).
claude-shim profile new personal --default
claude-shim profile new work
claude-shim profile new client-acme

# Point a single project at a profile (writes .claude/claude-shim-profile):
cd ~/Workspace/my-project
claude-shim profile use work

# Or, for a whole workspace of projects (writes .claude-shim-profile in the root):
cd ~/Workspace/work
claude-shim profile use work --workspace
```

Claude Code initializes the profile contents (settings, credentials, history) on first launch.
Both `profile new` and `profile use` fail loud rather than overwrite existing state —
remove the old profile directory or marker file before retrying.

`claude-shim profile current` in a directory with a valid profile prints the name and exits 0;
without a profile, it prints nothing and exits 0;
if the file points at a non-existent profile directory, it warns on stderr and exits 2.

`claude-shim profile list` prints every profile directory one per line,
appending `(default)` for the global default
and `(active)` for the one that would resolve in the current directory:

```sh
$ claude-shim profile list
client-acme
personal (default, active)
work
```

Which marker a directory resolves to —
and how the global default is chosen when there is none —
is covered in [Profile resolution](/guide/resolution).
