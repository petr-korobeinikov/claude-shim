## Why

A release is cut by hand: `git flow release start`, three edits - `version` in `Cargo.toml`, the `claude-shim` entry in `Cargo.lock`, `ver=` in `docs/guide/installation.md` - one commit, `git flow release finish` with the tag message typed again. All ten tags, `v2026.6.0-alpha.1` to `v2026.11.0`, were cut so, and the name's MICRO is counted from the tags by eye. Issue #34 asks for one owner of the bump; the next release, `2026.11.1`, is the first to have it.

Relatively simple: two shell tasks and their bats tests, no Rust, no public contract. Implemented on `develop`.

Builds on `develop` alone.

Subject: `chore: add a mise task to cut a release`

Decisions:

- Two tasks: `version:next` prints the next name, `release` cuts it - the name stays checkable without cutting anything.
- MICRO is one above the highest of this UTC month's final tags `vYYYY.M.N` and open `release/` and `hotfix/` branches; a hotfix started during a release does not take its number.
- `release --beta` cuts `<next>-beta.K`, K one above the highest `v<next>-beta.K` tag, from 1; a beta tag does not count toward MICRO, so the final release after betas takes their number, as `v2026.7.0` did after `-beta.2`. (operator)
- A beta leaves `ver=` alone: the install guide serves the latest final release, as both July betas left it. *Arguable.*
- The task pushes nothing and prints the push command; the tag starts `release.yml`, which publishes to crates.io irreversibly. (operator)
- The task refuses before it writes anything: off `develop`, a dirty tree, an open `release/` branch, or a local `develop` or `main` behind `origin`. Being ahead of `origin` passes. *Arguable.*
- The bump is a `sed` and `cargo update --workspace`, over `cargo-release`, which would need an audit and adds nothing a two-line edit lacks; this closes #34, which the operator closes after the change lands. (operator)
- Tests are bats in `e2e/` against a sandbox repository with a bare `origin`; `ci.yml`'s `e2e` job installs git-flow-next for them. A separate suite and task beat it only on naming. *Arguable.*

Assumptions, each checked by a task of group 1:

- `cargo update --workspace --offline` after the `Cargo.toml` edit changes one line of `Cargo.lock`, the package's `version`.
- `mise run release --beta` hands `--beta` to the script.
- `git flow release finish` opens no editor for its merges.

## What Changes

- New: `.mise/tasks/version/next`, `.mise/tasks/release`, `e2e/version_next.bats`, `e2e/release.bats`, `e2e/helpers/release.bash`.
- `ci.yml`'s `e2e` job installs `aqua:gittower/git-flow-next`.
- `docs/contributing.md` gains a Releases section; `CLAUDE.md`'s release paragraph names `mise run release`.

Public contract: none moves - the binary, its crate and its release assets are untouched.

## Capabilities

### New Capabilities

- `release`: how the next release is named and cut into a tagged commit on `main`.

### Modified Capabilities

None.

## Impact

The files under What Changes; nothing under `src/`.
