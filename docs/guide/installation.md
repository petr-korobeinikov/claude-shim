# Installation

`claude-shim` is a single self-contained binary.
Pick whichever route fits.

## From a release

Every [release](https://github.com/petr-korobeinikov/claude-shim/releases)
attaches prebuilt archives:

- **Linux** —
  `claude-shim-<version>-x86_64-unknown-linux-musl.tar.gz`
  (static, runs on any distro — glibc or musl);
- **macOS** —
  `claude-shim-<version>-macos-universal.tar.gz`
  (Apple Silicon + Intel).

```sh
ver=2026.7.0
base=https://github.com/petr-korobeinikov/claude-shim/releases/download
curl -fsSL "$base/v$ver/claude-shim-$ver-x86_64-unknown-linux-musl.tar.gz" | tar xz
./claude-shim --version
```

The macOS binary is not yet notarized —
if you downloaded it through a browser,
clear quarantine once with `xattr -d com.apple.quarantine claude-shim`.

## Globally via mise

With [mise](https://mise.jdx.dev/)
you can install straight from the GitHub releases.
mise selects the right asset for your platform
and verifies its GitHub artifact attestations:

```sh
mise use -g github:petr-korobeinikov/claude-shim@latest
claude-shim --version
```

## From crates.io

The crate is published to [crates.io](https://crates.io/crates/claude-shim),
so any Rust toolchain can fetch and build it:

```sh
cargo install claude-shim
claude-shim --version
```

This compiles from source —
slower than the prebuilt archives above —
and lands the binary in `~/.cargo/bin`
(already on your `PATH` if you use cargo).

## From source

```sh
cargo build --release
```

The binary lands at `./target/release/claude-shim`.

## The `claude` shim

However you install it,
the same binary doubles as the `claude` shim when invoked under that name.
The shim symlink (`<data dir>/claude-shim/shims/claude → claude-shim`)
is created automatically on every `claude-shim` run —
no manual step,
no separate binary to copy.
Keep the real `claude` (npm, brew, nvm, …) reachable on `PATH`;
the shim looks it up there,
skipping its own directory.

Commands throughout this documentation call `claude-shim` directly,
assuming it is on your `PATH`
(it will be after a mise install, or once you drop the binary into a `PATH` directory).
Otherwise substitute its full path.

## Shell integration (zsh)

Installs a precmd hook that exports `CLAUDE_SHIM_ACTIVE_PROFILE` on every prompt.
Required for both prompt-rendering paths in [Prompt indicator](/guide/prompt-indicator).

::: info
Only zsh is supported for now.
:::

Add to `~/.zshrc` and re-source:

```sh
eval "$(claude-shim init zsh)"
```
