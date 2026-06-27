# claude-shim

Profile manager for Claude Code:
swaps `CLAUDE_CONFIG_DIR` per project
and shows the active profile in the shell prompt.

> Alpha.
> Prebuilt binaries ship per tag;
> layout and flags may still change.

**Full documentation:** <https://petr-korobeinikov.github.io/claude-shim/>

## Install & setup

Build the binary —
prebuilt release archives and a `mise` install are in the
[docs](https://petr-korobeinikov.github.io/claude-shim/guide/installation):

```sh
cargo build --release
```

The same binary doubles as the `claude` shim:
it auto-creates the `claude` symlink on first run
and resolves the real `claude` from your `PATH`.
Put `./target/release/claude-shim` on your `PATH`, or substitute its full path below.

Install the shell hook that exports `CLAUDE_SHIM_ACTIVE_PROFILE` on every prompt —
add to `~/.zshrc` and re-source:

```sh
eval "$(claude-shim init zsh)"
```

Show the active profile in the prompt
(minimal PS1; oh-my-posh variants are in the
[docs](https://petr-korobeinikov.github.io/claude-shim/guide/prompt-indicator)):

```sh
PS1='%n@%m %~ ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] }%# '
```

Create a profile and point a project at it:

```sh
claude-shim profile new personal --default
claude-shim profile new work
cd ~/Workspace/acme
claude-shim profile use work
```

`claude` launched from that directory now runs under the chosen profile;
Claude Code initializes its contents on first launch.
See the [docs](https://petr-korobeinikov.github.io/claude-shim/) for profile
resolution, the workspace marker, and migrating an existing `~/.claude`.

## Contributing

Local Gitflow setup and the Claude Code skill set are documented in
[Contributing](https://petr-korobeinikov.github.io/claude-shim/contributing).

## License

[MIT](LICENSE)
