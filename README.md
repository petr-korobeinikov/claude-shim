# claude-shim

Profile manager for Claude Code:
swaps `CLAUDE_CONFIG_DIR` per project
and shows the active profile in the shell prompt.

> Pre-release.
> Installation instructions will land with the first tagged release;
> until then, build from source.

## Build

```sh
cargo build --release
```

The binary lands at `./target/release/claude-shim`.
Substitute `<claude-shim>` below with its absolute path on your machine.

The same binary acts as the `claude` shim when invoked under that name.
The shim symlink (`<data dir>/claude-shim/shims/claude → <claude-shim>`)
is created automatically on every `claude-shim` run —
no manual install step,
no separate binary to copy.
Make sure the real `claude` (npm, brew, nvm, etc.) is reachable on `PATH` —
the shim looks it up there,
skipping its own directory.

## Shell integration (zsh)

Installs a precmd hook that exports `CLAUDE_SHIM_ACTIVE_PROFILE` on every prompt.
Required for both prompt-rendering paths below.

Add to `~/.zshrc` and re-source:

```sh
eval "$(<claude-shim> init zsh)"
```

## Prompt indicator

Pick one of the styles below.

### Plain PS1

```sh
PS1='%n@%m %~ ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] }%# '
```

### oh-my-posh

Pick a format and inline the segment into your theme:

- **YAML** —
  add as a list item under `segments:` of the target block.
- **TOML** —
  paste right after the `[[blocks]]` table you want the segment to live in;
  `[[blocks.segments]]` attaches to the nearest preceding `[[blocks]]`.
- **JSON** —
  add as an object inside the matching `"segments": [ ... ]` array
  (mind the surrounding commas).

#### Minimal

A bare text segment —
no glyphs, no palette entry, no Nerd Font.
Drops into any existing OMP theme as-is.

**YAML**

```yaml
- type: text
  style: plain
  template: "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
```

**TOML**

```toml
[[blocks.segments]]
type = "text"
style = "plain"
template = "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
```

**JSON**

```json
{
  "type": "text",
  "style": "plain",
  "template": "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
}
```

#### Powerline diamond

A diamond segment with a powerline cap,
brand colour,
and ✳ glyph.

Requires:

- a Nerd Font for the `\ue0b0` powerline cap
  (✳ is plain Unicode U+2733 and renders with any modern font);
- a `claude` palette entry,
  added to the existing top-level `palette` block of your theme.

Pick your format and apply both pieces —
the palette entry goes inside your existing palette,
the segment goes inside the target `segments` array.

**YAML**

Palette entry, added under your existing `palette:`:

```yaml
claude: "#CC785C"
```

Segment:

```yaml
- type: text
  style: diamond
  leading_diamond: "<transparent,background>\ue0b0</>"
  trailing_diamond: "<background,transparent>\ue0b0</>"
  foreground: p:pure_black
  background: p:claude
  template: "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
```

**TOML**

Palette entry, added under your existing `[palette]` table:

```toml
claude = "#CC785C"
```

Segment:

```toml
[[blocks.segments]]
type = "text"
style = "diamond"
leading_diamond = "<transparent,background>\ue0b0</>"
trailing_diamond = "<background,transparent>\ue0b0</>"
foreground = "p:pure_black"
background = "p:claude"
template = "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
```

**JSON**

Palette entry, added inside your existing `"palette"` object:

```json
"claude": "#CC785C"
```

Segment:

```json
{
  "type": "text",
  "style": "diamond",
  "leading_diamond": "<transparent,background>\ue0b0</>",
  "trailing_diamond": "<background,transparent>\ue0b0</>",
  "foreground": "p:pure_black",
  "background": "p:claude",
  "template": "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
}
```

## Creating a profile

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

### Default profile

Markers are discovered by walking up from `$PWD` through the project tree —
the nearest match wins.
On each ancestor directory,
`.claude/claude-shim-profile` (per-project) is checked first
and takes priority over `.claude-shim-profile` (workspace-wide) at the same level.
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

To switch from the legacy `~/.claude` setup to a named profile,
do it by hand
(a `claude-shim migrate` command will land in a later release):

```sh
# macOS
mv ~/.claude "$HOME/Library/Application Support/claude-shim/profiles/default"
echo default > "$HOME/Library/Application Support/claude-shim/default-profile"

# Linux
mv ~/.claude "${XDG_DATA_HOME:-$HOME/.local/share}/claude-shim/profiles/default"
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/claude-shim"
echo default > "${XDG_CONFIG_HOME:-$HOME/.config}/claude-shim/default-profile"
```

## Development

### Git flow setup

Run once per fresh working copy to wire up the local Gitflow workflow.
Tooling is pinned in `mise.toml`
(git-flow-next via the `aqua` backend),
so `mise install` brings in `git-flow` along with the Rust toolchain.

```sh
mise install
git flow init --preset=classic --defaults
git config gitflow.branch.feature.upstreamstrategy rebase
```

The `upstreamstrategy=rebase` key enables the local-finish workflow:
`git flow feature finish` rebases the feature branch onto `develop` linearly,
no merge commit.
Collapse the feature into a single commit
(`git rebase -i $(git merge-base HEAD develop)`)
before running `feature finish`.

### Claude Code skills

`skills-lock.json` pins the Claude Code skill set this repo expects.
Skills are materialized into `.claude/skills/` (gitignored).
Install them on a fresh working copy with the `skills` CLI:

```sh
npx skills add petr-korobeinikov/skills --skill '*' --copy --agent claude-code -y
```
