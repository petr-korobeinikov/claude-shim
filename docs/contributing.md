# Contributing

::: info
This page currently covers local development setup;
community contribution guidelines will be added in a later update.
:::

## Git flow setup

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

## Pre-commit hooks

`mise install` also brings in `prek`
(a Rust drop-in for `pre-commit`, pinned in `mise.toml`),
so local commits run the same `cargo fmt` / `clippy` gates as CI.
Wire the hook into `.git/hooks/` once per fresh working copy:

```sh
prek install
```

The hooks in `.pre-commit-config.yaml` mirror `.github/workflows/ci.yml`,
so a clean local commit means a green CI check.

## End-to-end tests

Unit tests run under `cargo test`.
End-to-end tests drive the real binary — and the zsh integration —
with [bats-core](https://bats-core.readthedocs.io/),
pinned in `mise.toml` and brought in by `mise install`.
Build first, then run the suite:

```sh
cargo build
bats e2e/
```

Every test runs against a throwaway `HOME`,
so your real profiles stay untouched.

## Claude Code skills

`skills-lock.json` pins the Claude Code skill set this repo expects.
Skills are materialized into `.claude/skills/` (gitignored).
Install them on a fresh working copy with the `skills` CLI:

```sh
npx skills add petr-korobeinikov/skills --skill '*' --copy --agent claude-code -y
```
