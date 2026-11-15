# release Specification

## Purpose
Names the next release and cuts it into a tagged commit on `main`, leaving every push to the operator.

## Requirements

### Requirement: The next release name is printed
`mise run version:next` SHALL print `YYYY.M.MICRO` for the current UTC year and month, MICRO one above the highest of that month's final tags `vYYYY.M.N` and open `release/YYYY.M.N` and `hotfix/YYYY.M.N` branches, 0 when none exists.
With `--beta` it SHALL print that name and `-beta.K`, K one above the highest tag `v<name>-beta.K`, 1 when none exists.

#### Scenario: First of the month
- **WHEN** no tag or branch of the current month exists
- **THEN** it prints `<year>.<month>.0`, the month without a leading zero

#### Scenario: Taken numbers
- **WHEN** the month has tag `v<year>.<month>.0` and open branch `hotfix/<year>.<month>.1`
- **THEN** it prints `<year>.<month>.2`

#### Scenario: Betas do not take a number
- **WHEN** the month has tags `v<year>.<month>.0` and `v<year>.<month>.1-beta.1`
- **THEN** it prints `<year>.<month>.1`, and with `--beta` `<year>.<month>.1-beta.2`

### Requirement: The release commit carries the version
`mise run release` SHALL commit on `release/<name>` one commit, `chore: release <name>`, setting `<name>` as `version` in `Cargo.toml` and in `Cargo.lock`'s `claude-shim` entry, and as `ver=` in `docs/guide/installation.md` for a final release only.

#### Scenario: Final release
- **WHEN** `mise run release` runs
- **THEN** the release commit changes one line in each of the three files and nothing else

#### Scenario: Beta
- **WHEN** `mise run release --beta` runs
- **THEN** the release commit changes `Cargo.toml` and `Cargo.lock` only

### Requirement: The release lands tagged and unpushed
`mise run release` SHALL finish `release/<name>` into `main` with tag `v<name>`, whose message is `v<name>`, merge `main` into `develop`, delete the branch, end on `develop`, push nothing, and print `git push origin develop main v<name>`.

#### Scenario: Finished
- **WHEN** `mise run release` exits 0
- **THEN** `v<name>` points at `main`, `develop` contains it, `release/<name>` is gone, `HEAD` is `develop`, and every ref on `origin` is as before

### Requirement: An unsafe start is refused
`mise run release` SHALL exit 1 with a message naming the cause and change no branch, tag or file when `HEAD` is not `develop`, the tree has a change, a `release/` branch is open, or `develop` or `main` lacks a commit of its `origin` counterpart. An unknown argument SHALL exit 2.

#### Scenario: Refused
- **WHEN** any one of those holds
- **THEN** it exits 1 naming that cause, and `git for-each-ref refs/heads refs/tags` and `git status --porcelain` read as before

#### Scenario: Ahead of origin
- **WHEN** `develop` has a commit `origin/develop` lacks and nothing else holds
- **THEN** the release proceeds

#### Scenario: Unknown argument
- **WHEN** `mise run release --push` runs
- **THEN** it exits 2 naming `--push`, and nothing changes
