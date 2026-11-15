## Why

The site builds with VitePress 1.6.4, which pins `vite ^5`: six Dependabot alerts stay open in `package-lock.json` - five on `vite` (one high) and one on `minimatch` - and no lock bump closes them. f0-claude-remote builds its site with Zensical from a locked, wheel-only graph on a private Python, with no such alerts and a look the operator prefers; one toolchain serves both projects.

Relatively simple: the binary does not move; the work is tooling, ten pages of markup and two workflows, mirroring f0. Implemented on `develop`.

Builds on `add-statusline-presets`, applied before it: this change converts the `docs/guide/statusline.md` that plan writes.

Subject: `docs: move the site from vitepress to zensical`

Decisions:

- Zensical 0.1.1 with a fresh audit, over f0's audited 0.1.0. (operator)
- The home page is plain Markdown over a custom hero template that any Zensical update could break. (operator)
- The logo and favicon are the site's project icon, `petr-korobeinikov-com-web/static/projects/oss/claude-shim/img/icon.svg`; its `#d97757` is the accent. (operator)
- Every published URL keeps resolving: `use_directory_urls = false` writes `guide/installation.html` as VitePress does, over directory URLs that would break every `.html` link already out. *Arguable.*
- Python isolation follows f0 whole, detailed in design. (operator)
- `docs.yml` takes f0's form - commit-pinned actions, `persist-credentials: false`, Pages permissions in the deploy job alone - and drops `actions/configure-pages`, which f0 deploys without. *Arguable.*
- `ci.yml` gains a strict docs build, so a dead link fails the pull request instead of the release. *Arguable.*
- `docs/public/banner.svg` stays: READMEs published on crates.io point at it.

Assumptions, each checked by a task of group 1:

- GitHub Pages serves `guide/installation.html` for `/claude-shim/guide/installation`, the form `README.md` links.
- `[tool_config] locked = true` still installs every tool `mise.toml` declares - `rust` among them, whose `mise.lock` entry has no download URL - in the jobs `ci.yml` and `release.yml` run.

Audit, run on 2026-11-15 by `pip-audit` (OSV and PyPI), Snyk, and a container with no host mount whose build ran with the network cut:

| Package | Version | License | Source | Known vulnerabilities | Code at install |
|---|---|---|---|---|---|
| `zensical` | 0.1.1 | MIT | PyPI, `zensical/zensical` | none | none, abi3 wheel |
| `zensical-ui` | 0.0.42 | MIT | PyPI | none | none, wheel |
| 9 transitive | | BSD, MIT | PyPI | none | none, wheels |
| `actions/checkout` | v7.0.1 `3d3c42e5` | MIT | GitHub | none | none |
| `actions/upload-pages-artifact` | v5.0.0 `fc324d35` | MIT | GitHub | none | `actions/upload-artifact` by commit |
| `actions/deploy-pages` | v5.0.1 `368f8252` | MIT | GitHub | none | none |
| `jdx/mise-action` | v5.2.6 `f5be966e` | MIT | GitHub | none | none; v4 → v5 is a major bump |

The nine: click 8.5.1, Jinja2 3.1.6, Markdown 3.11.3, MarkupSafe 3.0.4, pathspec 1.1.1, Pygments 2.21.0, pymdown-extensions 13.1, PyYAML 6.0.3, tomli 2.5.0. Against f0's graph only `zensical-ui` (0.0.41) and `pymdown-extensions` (12.3, a major bump) moved. Each action's tag resolves to that commit and its repository lists no advisory; f0 runs the same four. Socket was not run: it needs a token, as in f0.

## What Changes

- New: `zensical.toml`, `.mise/tasks/docs-build` and `docs-dev`, `docs/assets/logo.svg`, `docs/stylesheets/extra.css`, `docs/templates/main.html`.
- `mise.toml` gains `uv`, `pypi:zensical`, `[tool_config]` and the Python guard; `mise.lock` and `.mise/locks/pypi-zensical/` follow.
- Pages move from VitePress containers to admonitions and tabs, and from root links to relative ones.
- Removed: `package.json`, `package-lock.json`, `docs/.vitepress/`, and the npm entry of `.github/dependabot.yml`.
- `docs.yml` and `ci.yml` build with Zensical; `SECURITY.md` and `.gitignore` follow.

Public contract: none moves - the binary is untouched, and the site keeps its URLs.

## Capabilities

### New Capabilities

- `docs-site`: what the site builds, shows and serves, from `docs/`.
- `docs-deploy`: how the site is checked and published.
- `toolchain-isolation`: the docs toolchain's Python and packages kept to the project.

### Modified Capabilities

None.

## Impact

The files under What Changes; nothing under `src/` or `e2e/`.
