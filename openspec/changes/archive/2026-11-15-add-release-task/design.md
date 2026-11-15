## Context

`.gitflow` already gives releases `tagPrefix = v` and merges them into `main` and back into `develop`; `release.yml` gates the tag against `Cargo.toml`'s `version`. The tag is signed by the operator's own `tag.gpgsign`, which the task leaves to git. No layer of the binary is touched: core, glue and shell integration stay as they are.

## Goals / Non-Goals

**Goals:** the steps of a release, from the name to the merge back into `develop`, run as one command that stops before anything leaves the machine.

**Non-Goals:** a hotfix task; pushing; release notes, which `release.yml` writes with git-cliff; dropping `ver=` for a `releases/latest/download` link, which the versioned asset names rule out.

## Decisions

- **Two POSIX `sh` files, `set -eu`**, as `.mise/tasks/` already holds: `version/next` (mise names it `version:next`) and `release`. One file beat it only by hiding the name from a check.
- **The working directory's repository is the subject.** Each task `cd`s to `git rev-parse --show-toplevel`, and `release` calls `version/next` by `$(dirname "$0")`, not `mise run`; this is the seam the tests use - the checkout's scripts run inside a sandbox repository.
- **`release` in order:** parse the argument (`--beta` or none, else exit 2); `git fetch -q --tags origin`; the guards of spec "An unsafe start is refused", each a `git` query; `version/next`; `git flow release start <name>`; edit; `git add` the files; `git commit -m "chore: release <name>"`; `git flow release finish -m "v<name>" <name>`; `git checkout -q develop`; print the push command. Every guard runs before the first write, so a refusal changes nothing.
- **"Behind" is `git rev-list --count <branch>..origin/<branch>` above 0**, over comparing hashes, which would refuse a `develop` ahead of `origin` the operator has not pushed yet.
- **Edits:** `sed -i.orig` on the first `^version = ` line of `Cargo.toml` (`[package]` comes first, and the line is checked to be inside it) and on `^ver=` of the guide, then `rm` of the `.orig`, since BSD and GNU `sed` agree only on an attached suffix; then `cargo update --workspace --offline` for `Cargo.lock`, over a third `sed` that would have to find the entry by its preceding `name` line.
- **`version/next` lists names with `git for-each-ref refs/tags refs/heads/release refs/heads/hotfix`**, strips the prefixes, keeps those matching `^[0-9]{4}\.([1-9]|1[0-2])\.(0|[1-9][0-9]*)$` for the month, and takes the highest MICRO numerically. With `--beta` it matches `^<name>-beta\.([1-9][0-9]*)$` among the tags.
- **Comments name no other project**: they carry only the why of the line under them.

## Tests

| Part | Test | Seam |
| --- | --- | --- |
| `version/next` | `e2e/version_next.bats`, one test per scenario of "The next release name is printed" | tags and branches put on a sandbox repository; the month is `date -u +%Y.%-m`, as the task reads it |
| `release` | `e2e/release.bats`, one test per remaining scenario | the sandbox below |

`e2e/helpers/release.bash` builds the sandbox under `mktemp -d`: `GIT_CONFIG_GLOBAL` at a file giving `user.name`, `user.email`, `commit.gpgsign = false` and `tag.gpgsign = false`, `GIT_CONFIG_NOSYSTEM=1`; a repository with the checkout's `.gitflow` synced by `git flow config sync`, a `Cargo.toml` of package `claude-shim` with no dependencies, its `Cargo.lock` from `cargo generate-lockfile --offline`, a `docs/guide/installation.md` with a `ver=` line, `main` and `develop`, and a bare `origin` they are pushed to. `GIT_EDITOR=false` makes any editor prompt fail the test. These tests need no binary; `_common_setup` is not called.

## Risks / Trade-offs

- A step fails after `release start` → the task stops under `set -e` with `release/<name>` open; the next run refuses on it, and the operator deletes it.
- A run spans midnight UTC on the last day of a month → the test computes another month than the task; the test retries nothing, and a rerun passes.
- Betas cut in one month and the final in the next → the final takes the new month's number; `release --beta` names only what the current month gives.
