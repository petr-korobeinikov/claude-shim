# docs-deploy Specification

## Purpose
Checks the documentation site on every push and pull request,
and publishes it on GitHub Pages with each release.

## Requirements

### Requirement: Every push and pull request builds the site
`ci.yml` SHALL build the site as `mise run docs-build` does,
so a dead link fails the pull request and the release `ci.yml` gates.

#### Scenario: Dead link in a pull request
- **WHEN** a pull request links a missing page
- **THEN** `ci.yml`'s docs job fails, naming the link

### Requirement: The site publishes with each release
`release.yml` SHALL publish the site from the release's commit to `https://petr-korobeinikov.github.io/claude-shim/`,
as SHALL a manual run of `docs.yml`, and only the publishing job SHALL hold Pages' permissions.

#### Scenario: Workflow
- **WHEN** `docs.yml` is read
- **THEN** it runs on `workflow_call` and `workflow_dispatch`, grants `contents: read`,
  adds `pages: write` and `id-token: write` in the deploy job alone,
  pins each `uses:` to a 40-hex commit and checks out with `persist-credentials: false`

#### Scenario: Published
- **WHEN** `docs.yml` has run
- **THEN** the site, its `llms.txt` and `guide/installation.md` answer 200 with that commit's text

### Requirement: Published links keep working
Each link to a page that `README.md` holds, extension-less, SHALL answer 200 with that page.

#### Scenario: README links
- **WHEN** `https://petr-korobeinikov.github.io/claude-shim/guide/installation` and `.../guide/prompt-indicator` are fetched after a publish
- **THEN** each answers 200, following redirects, with the Installation and Prompt indicator pages

### Requirement: The docs jobs install only the docs toolchain
A job that builds the site SHALL install `uv` and Zensical from `mise.lock` alone,
and no tool `mise.toml` declares beside them.

#### Scenario: Job log
- **WHEN** the log of a job that built the site is read
- **THEN** mise installs `uv`, `python` 3.14.10 and `pypi:zensical` 0.1.1, and nothing else
