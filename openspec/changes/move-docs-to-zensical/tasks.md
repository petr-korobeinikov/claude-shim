# Tasks

## 1. Assumptions

- [ ] 1.1 `curl -sIL https://petr-korobeinikov.github.io/claude-shim/guide/installation` answers 200 with the Installation page. 200 → proceed; 404 → stop and replan the URLs (proposal, Decisions).
- [ ] 1.2 In a scratch clone given `[tool_config] locked = true` and an empty `MISE_DATA_DIR`, `mise install` installs every declared tool. Exit 0 → proceed; a refusal → stop and replan (design, Risks).

## 2. Toolchain

- [ ] 2.1 `mise.toml` per design, through the mise skill: `mise install uv python@3.14.10`, `mise lock`, the `pypi:zensical` entry carrying a `uv` line, then the strict install. Check: `toolchain-isolation`'s scenarios of the private interpreter, `PATH`, the foreign directory and the caller's environment.
- [ ] 2.2 Under a throwaway `MISE_DATA_DIR`, "Fresh clone with unlocked global tools", "Graph missing" and "Graph edited" pass; the real `.mise/locks/` and `mise.lock` stay unchanged by `git diff`.

## 3. Site

- [ ] 3.1 `zensical.toml`, `.mise/tasks/docs-build` and `docs-dev`, `docs/assets/logo.svg` copied from the site repository's `icon.svg`, `docs/stylesheets/extra.css`, `docs/templates/main.html`; `.gitignore` gains `/.site/` and `/.pycache/`. Check: "Build", "Dead link", "Page files" and "Shared runtime untouched".
- [ ] 3.2 Pages per design: `:::` boxes to admonitions, code groups to tabs, root links to relative, `index.md` to Markdown with `public/banner.svg`; `ver=` in `installation.md` keeps the line the release commit edits. Check: "Sidebar", "Installation", "Index", and `grep -rn ':::' docs` finds nothing.
- [ ] 3.3 The operator opens `mise run docs-dev`'s site and confirms "Scheme", "Header", "Network log" and "Live edit".

## 4. Removals and workflows

- [ ] 4.1 Remove `package.json`, `package-lock.json`, `docs/.vitepress/`, the npm entry of `.github/dependabot.yml` and the VitePress lines of `.gitignore`; rewrite `SECURITY.md`'s docs paragraph for Zensical's locked graph. Check: `git grep -n -i -e vitepress -e 'npm run'` finds only `openspec/changes/`; `mise run docs-build` green.
- [ ] 4.2 `docs.yml` and `ci.yml`'s `docs` job per design. Check: actionlint clean; `act` runs both build jobs green, and `ci.yml`'s red on a planted dead link ("Dead link in a pull request"); "Workflow" and "Job log" read off the files and `act`'s log.

## 5. Integration

- [ ] 5.1 A fresh clone: `mise install`, `mise run docs-build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` and `mise run e2e` green, `git status --short` empty.

## Workflow follow-up

- `openspec archive move-docs-to-zensical -y` within the change's one commit on `develop`, subject `docs: move the site from vitepress to zensical`.
- After the next release publishes the site: "Published" and "README links" are checked on `https://petr-korobeinikov.github.io/claude-shim/`; Dependabot's six alerts close with `package-lock.json`.
