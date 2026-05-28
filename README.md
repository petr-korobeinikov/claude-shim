# claudectl

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

The binary lands at `./target/release/claudectl`.
Substitute `<claudectl>` below with its absolute path on your machine.

## Shell integration (zsh)

Installs a precmd hook that exports `CLAUDECTL_ACTIVE_PROFILE` on every prompt.
Required for both prompt-rendering paths below.

Add to `~/.zshrc` and re-source:

```sh
eval "$(<claudectl> init zsh)"
```

## Prompt indicator

Pick one.

**oh-my-posh** —
print the JSON segment and inline it into your theme's `segments` array:

```sh
<claudectl> init oh-my-posh
```

**Plain PS1:**

```sh
PS1='%n@%m %~ ${CLAUDECTL_ACTIVE_PROFILE:+[$CLAUDECTL_ACTIVE_PROFILE] }%# '
```

## Creating a profile

Until a `create` command lands, do it by hand:

```sh
# macOS
mkdir -p "$HOME/Library/Application Support/claudectl/profiles/<name>"
# Linux
mkdir -p "${XDG_DATA_HOME:-$HOME/.local/share}/claudectl/profiles/<name>"

mkdir -p .claude
echo <name> > .claude/claudectl-profile
```

`claudectl current` in a directory with a valid profile prints the name and exits 0;
without a profile, it prints nothing and exits 0;
if the file points at a non-existent profile directory, it warns on stderr and exits 2.

### Default profile

`.claude/claudectl-profile` is discovered by walking up from `$PWD` through the project tree —
the nearest match wins.
The walk stops before `$HOME`,
so a `~/.claude/claudectl-profile` is not picked up as a global default.

When no project marker is found, the shim resolves the profile in this order:

1. `~/.config/claudectl/default-profile` — a text file containing one profile name.
   Recommended way to set a global default.
2. `~/.claude/` itself — used as the profile if it exists,
   for installs that pre-date claudectl.
3. Otherwise the shim refuses to run rather than silently fall back to an arbitrary profile.

To switch from the legacy `~/.claude` setup to a named profile,
do it by hand
(a `claudectl migrate` command will land in a later release):

```sh
# macOS
mv ~/.claude "$HOME/Library/Application Support/claudectl/profiles/default"
# Linux
mv ~/.claude "${XDG_DATA_HOME:-$HOME/.local/share}/claudectl/profiles/default"

mkdir -p ~/.config/claudectl
echo default > ~/.config/claudectl/default-profile
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
