# Contributing

!!! info
    This page currently covers local development setup;
    community contribution guidelines will be added in a later update.

## Git flow setup

Tooling is pinned in `mise.toml`
(git-flow-next via the `aqua` backend),
so `mise install` brings in `git-flow` along with the Rust toolchain.
The branch model itself is committed in `.gitflow`,
so it survives cloning — `.git/config` does not.
Copy it into `.git/config` once per fresh working copy:

```sh
mise install
git flow config sync
```

`main` and `develop` are the long-lived branches.
Feature and bugfix branches are squashed into one commit on finish,
with the subject given on the command line:

```sh
git flow feature start <id>
git flow feature finish <id> --squash-message "<subject>"
```

Releases and hotfixes merge and tag `v<version>`.

## Releases

A release is named `YYYY.M.MICRO`,
MICRO counting this month's releases from 0;
a beta appends `-beta.N`.
`mise run version:next` prints the next name and changes nothing:

```sh
mise run version:next          # 2026.11.1
mise run version:next --beta   # 2026.11.1-beta.1
```

`mise run release` cuts it, from a clean `develop` that is not behind `origin`:
it starts `release/<name>`,
commits the version into `Cargo.toml`, `Cargo.lock`
and `docs/guide/installation.md`,
finishes the branch into `main` with tag `v<name>`,
and merges `main` back into `develop`.
`--beta` cuts the next beta instead
and leaves the install guide on the latest final release.
Nothing is pushed:
the task ends by printing the command that publishes the release,
since the tag starts `release.yml` and its publish to crates.io cannot be undone:

```sh
mise run release   # ends with: git push origin develop main v2026.11.1
```

## Pre-commit hooks

`mise install` also brings in `prek`
(a Rust drop-in for `pre-commit`, pinned in `mise.toml`, configured in `prek.toml`).
Wire both git hooks into `.git/hooks/` once per fresh working copy,
and again after a prek upgrade —
the installed hooks pin the path of the prek binary, and that path carries the version:

```sh
prek install
```

At `pre-commit`, prek runs its built-in file-hygiene checks
and the `cargo fmt` / `clippy` gates,
which mirror `.github/workflows/ci.yml`,
so a clean local commit means a green CI check.
At `commit-msg`, it refuses a subject over 72 characters,
a subject that is not Conventional Commits,
and a line that credits an AI agent or a model.

```sh
prek run --all-files   # every pre-commit hook over the whole tree
```

## Spec-driven development

Changes are planned with [OpenSpec](https://github.com/Fission-AI/OpenSpec)
(pinned in `mise.toml`) before their code is written,
driven from Claude Code via the `openspec-*` skills.
`CLAUDE.md` holds the process,
and `openspec/config.yaml` the rules every plan follows.
In short:
a plan is one OpenSpec change,
committed on `develop` as `chore(openspec): plan <id>`;
a simple change is implemented on `develop`,
a complex one on `feature/<id>`,
and either reaches `develop` as one commit
with its code, tests, specs and archived plan.

```sh
mise exec -- openspec list   # changes planned and under way
mise exec -- openspec view   # browse specs and changes
```

## End-to-end tests

Unit tests run under `cargo test`.
End-to-end tests drive the real binary — and the zsh and bash integrations —
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
Skills are materialized into `.claude/skills/` (gitignored,
except the links to the `openspec-*` skills OpenSpec generates,
which are committed along with the skills themselves in `.agents/skills/`).
Install them on a fresh working copy with the `skills` CLI:

```sh
npx skills add petr-korobeinikov/skills --skill '*' --copy --agent claude-code -y
```
