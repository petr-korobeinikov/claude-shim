# Design

## Context

No core, glue or shell integration moves: every file is docs tooling or a page.
Today `package.json` installs VitePress 1.6.4 and `vitepress-plugin-llms` through `node`,
`docs.yml` builds them, and `release.yml`'s `deploy-docs` calls `docs.yml` after the Release.
`node` stays in `mise.toml`: `npm:@fission-ai/openspec` runs on it.

`mise.toml` gains:

```toml
[tool_config]
locked = true

[tools]
uv = "0.13.7"
# Builds docs/. Not 3.15: PyYAML 6.0.3 has no cp315 wheel.
"pypi:zensical" = { version = "0.1.1", python = "3.14.10" }

[env]
PYTHONPATH = false
PYTHONHOME = false
PIP_REQUIRE_VIRTUALENV = "true"
UV_PYTHON_DOWNLOADS = "never"
PYTHONPYCACHEPREFIX = "{{config_root}}/.pycache"
```

No `python` in `[tools]` and no `_.path`: the runtime stays private to Zensical's venv.

## Goals / Non-Goals

**Goals:** close each leak below and prove it by the `toolchain-isolation` scenarios.

**Non-Goals:** other workflows' pins and forms; a `.mise/tasks/setup` with `sandbox-exec`, which f0 needs for ufbt's SDK and Zensical does not; Mermaid, which needs a CDN.

## Decisions

Measured on 2026-11-15 with 0.1.1 in the audit's container unless a source is cited:

| Leak | Closure | Alternative beaten |
|---|---|---|
| Plain `mise install`, `mise exec` and `mise run` resolve afresh without a graph, and may build sdists | `[tool_config] locked`: "require tools declared by this config root to resolve and install from their lockfiles" ([schema](https://mise.jdx.dev/schema/mise.json)) | `mise install --locked` by hand: `mise exec` stays loose |
| A build writes stdlib `.pyc` into the shared runtime (`xml/etree`, `zoneinfo`) | `PYTHONPYCACHEPREFIX` into `.pycache/`, git-ignored | `PYTHONDONTWRITEBYTECODE`: recompiles on each start |
| Zensical's cache | `.cache/` at the root, which writes its own `.gitignore` of `*` | None needed |
| `PYTHONPATH` shadows the venv's imports; stray `pip`; uv's own CPython | The guard block | None |
| In CI, `mise exec` and `mise run` install every tool a job leaves out | `MISE_AUTO_INSTALL: "false"` on the docs jobs, as in f0 | None |

- **`use_directory_urls = false`,** over Zensical's default `true` (`zensical/config.py:671` of 0.1.0):
  the container's build wrote `guide/installation.html`, and `llms.txt` linked `guide/installation.md`.
- **The accent is a custom palette color,** `primary = "custom"` with `--md-primary-fg-color` in `docs/stylesheets/extra.css`, loaded through `extra_css`, over the nearest named color, `deep orange`, which is not the icon's.
- **`pymdownx.tabbed` with `alternate_style`** for VitePress's `::: code-group`, over stacked blocks: the container rendered a `tabbed-set` from `=== "zsh"`.
- **Links between pages are relative** (`installation.md`), over root links (`/guide/installation`), which `--strict` cannot check.
- **`docs/templates/main.html` sets `darkreader-lock`,** as f0 does: the site has its own dark scheme.
- **`font = false`:** the theme calls Google Fonts otherwise.
- **The bundle's CDN URLs stay:** `unpkg.com` and `cdn.jsdelivr.net` appear in `bundle.*.min.js` only to load Mermaid, glightbox, ace and pyodide, none of which the site enables; the network-log scenario proves none loads.

`docs.yml` keeps `workflow_call` and `workflow_dispatch`.
Its build job checks out by commit with `persist-credentials: false`,
runs `jdx/mise-action` by commit with `version: 2026.11.2`, `cache: false` and `install_args: uv pypi:zensical`,
then `mise run docs-build`, and uploads `.site`.
The deploy job alone holds `pages: write` and `id-token: write`.
`ci.yml`'s new `docs` job runs the same build steps; `release.yml` already gates on `ci.yml`.

`.mise/tasks/docs-build` runs `zensical build --strict --clean`;
`docs-dev` runs `zensical serve -a 127.0.0.1:8000`.

Testing: no business logic.
The seam is the process boundary, and each spec scenario is a command or an observation;
`act` runs `docs.yml`'s build job and `ci.yml`'s `docs` job before the commit.

## Risks / Trade-offs

- `[tool_config]` refuses `rust`, whose `mise.lock` entry carries no URL, or another tool → task 1.2 finds it before anything is written; a refusal stops the change for a replan.
- `mise-action`'s mise 2026.11.2 reads a `mise.lock` that mise 2026.11.8 wrote → `act` runs the jobs; a format error pins `version` to the local mise.
- A Zensical 0.1.x bump changes `zensical.toml` keys → the version is pinned, and a bump is a change of its own.
- `pymdown-extensions` 13 is a major bump past f0's 12.3 → the container built admonitions and tabs on 13.1; the build checks it again.
- `/claude-shim/banner.svg` stops resolving → nothing links it: the website keeps its own copy.

## Migration Plan

One commit on `develop`; the next release publishes the site.
Rollback is `git revert` of that commit and a manual run of `docs.yml`.
