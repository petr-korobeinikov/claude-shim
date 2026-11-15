# Tasks

## 1. Assumptions

- [x] 1.1 In a sandbox built by hand as design's Tests describe, edit `Cargo.toml`'s `version`, run `cargo update --workspace --offline`. `git diff --numstat Cargo.lock` reads `1 1` → proceed; anything else → stop and replan (proposal, Assumptions).
  - `1	1	Cargo.lock` on cargo 1.100.0.
- [x] 1.2 A throwaway task printing `"$@"`, run as `mise run <it> --beta`, prints `--beta` → proceed; mise consuming the flag → the spec and docs say `mise run release -- --beta`, recorded here.
  - mise 2026.11.8 passes `--beta` and `--push` through; no `--` needed.
- [x] 1.3 In that sandbox with `GIT_EDITOR=false`, `git flow release start 0.0.1` and `git flow release finish -m v0.0.1 0.0.1` exit 0 → proceed; an editor prompt → stop and replan.
  - git-flow-next 2.1.0: both exit 0, finish ends on `develop`, pushes nothing.

## 2. Release name

- [x] 2.1 `e2e/version_next.bats` per design's Tests, with `e2e/helpers/release.bash`. Check: `bats e2e/version_next.bats` fails, each test on the missing task.
  - 3 of 3 fail with 127. "First of the month" also tags `v2000.1.5`, to prove another month is ignored.
- [x] 2.2 `.mise/tasks/version/next` per design. Check: `bats e2e/version_next.bats` green; in the checkout `mise run version:next` prints `2026.11.1` and `mise run version:next --beta` prints `2026.11.1-beta.1` (expected, in November 2026).
  - 3 of 3 green; the checkout prints both as expected. An unknown argument exits 2, as `release` does.

## 3. Release

- [x] 3.1 `e2e/release.bats`, one test per scenario of "The release commit carries the version", "The release lands tagged and unpushed" and "An unsafe start is refused", the refusal once per cause. Check: `bats e2e/release.bats` fails on the missing task.
  - 10 of 10 fail with 127.
- [x] 3.2 `.mise/tasks/release` per design. Check: `bats e2e/release.bats` green; `sh -n` passes on both tasks.
  - 10 of 10 green; `sh -n` clean on both. The `[package]` check refuses with exit 1 before `release start`, so it writes nothing either.
- [x] 3.3 `ci.yml`'s `e2e` job adds `aqua:gittower/git-flow-next` to `install_args`. Check: actionlint clean; `act` runs the `e2e` job green, or the reason it cannot is noted here.
  - actionlint 0 with `-shellcheck= -pyflakes=`: the global shellcheck shim has no version.
  - act 0.2.97 on colima, `catthehacker/ubuntu:act-latest` native arm64: 69 of 69. Under `linux/amd64` emulation `mise-action`'s `tar` fails with ENOSYS before any step of ours.

## 4. Docs and integration

- [x] 4.1 `docs/contributing.md` gains a Releases section: `mise run version:next`, `mise run release`, `--beta`, the printed push command. `CLAUDE.md`'s release paragraph says `mise run release` makes the release commit. Check: `mise run docs-build` green.
  - `docs-build`: no issues found.
- [x] 4.2 `cargo build && mise run e2e` green: 56 tests before, 56 + 3 + 10 = 69 after (expected): five refusal causes.
  - 56 before, 69 of 69 green after.

## Workflow follow-up

- The operator closes #34 once the change lands on `develop`.
- `2026.11.1` is cut with `mise run release` and pushed by the operator.
