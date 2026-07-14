# Quick Start

Set up per-project profile switching in three steps.
Each step links to the page with the full details.

## 1. Install

::: tip Preferred — install with mise
[mise](https://mise.jdx.dev/) picks the right prebuilt binary for your platform
and verifies its GitHub artifact attestations:

```sh
mise use -g github:petr-korobeinikov/claude-shim@latest
```
:::

Or install from crates.io —
all install routes are in [Installation](/guide/installation):

```sh
cargo install claude-shim
```

`cargo install` lands the binary in `~/.cargo/bin` (already on your `PATH` if you use cargo).

## 2. Wire up the shell hook

Keeps the shims dir first on your `PATH` so the `claude` shim is what runs.
It also exports `CLAUDE_SHIM_ACTIVE_PROFILE` on every prompt
as a secondary effect the optional prompt indicator consumes.
Add to `~/.zshrc` and re-source:

```sh
eval "$(claude-shim init zsh)"
```

::: info
The shell hook is zsh-only for now.
:::

## 3. Create a profile and point a project at it

```sh
# A global default + a second profile (--statusline adds the in-session indicator)
claude-shim profile new personal --default
claude-shim profile new work --statusline

# Point one project at the work profile
cd ~/Workspace/acme
claude-shim profile use work
```

`claude` launched from that directory now runs under the chosen profile;
Claude Code initializes its contents on first launch.

## Next steps

- **Show the active profile in your shell prompt** —
  [Prompt indicator](/guide/prompt-indicator) has the plain `PS1` plus oh-my-posh variants.
- **Show it in Claude Code's status bar (statusLine)** —
  [statusLine indicator](/guide/statusline).
- **Manage profiles** —
  [Profiles](/guide/profiles) covers `current` / `list` and the workspace marker.
- **Understand profile resolution** —
  [Profile resolution](/guide/resolution) explains how the active profile is chosen.
