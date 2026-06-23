# Quick Start

Set up per-project profile switching in four steps.
Each step links to the page with the full details.

## 1. Install

::: tip Preferred — install with mise
[mise](https://mise.jdx.dev/) picks the right prebuilt binary for your platform
and verifies its GitHub artifact attestations:

```sh
mise use -g github:petr-korobeinikov/claude-shim@2026.6.0-alpha.2
```
:::

Or build from source —
all install routes are in [Installation](/guide/installation):

```sh
cargo build --release
```

The binary lands at `./target/release/claude-shim`.
Put it on your `PATH`, or substitute its full path below.

## 2. Wire up the shell hook

Exports `CLAUDE_SHIM_ACTIVE_PROFILE` on every prompt.
Add to `~/.zshrc` and re-source:

```sh
eval "$(claude-shim init zsh)"
```

## 3. Show the active profile

The minimal indicator — a plain PS1:

```sh
PS1='%n@%m %~ ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] }%# '
```

oh-my-posh users have minimal and powerline variants in
[Prompt indicator](/guide/prompt-indicator).

## 4. Create a profile and point a project at it

```sh
# A global default + a second profile
claude-shim profile new personal --default
claude-shim profile new work

# Point one project at the work profile
cd ~/Workspace/acme
claude-shim profile use work
```

`claude` launched from that directory now runs under the chosen profile;
Claude Code initializes its contents on first launch.
See [Profiles](/guide/profiles) for `current` / `list` and the workspace marker,
and [Profile resolution](/guide/resolution) for how the active profile is chosen.
