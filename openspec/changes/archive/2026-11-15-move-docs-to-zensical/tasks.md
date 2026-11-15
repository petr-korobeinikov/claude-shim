# Tasks

## 1. Assumptions

- [x] 1.1 `curl -sIL https://petr-korobeinikov.github.io/claude-shim/guide/installation` answers 200 with the Installation page. 200 → proceed; 404 → stop and replan the URLs (proposal, Decisions).
  - 200 with no redirect, `<title>Installation | claude-shim</title>`.
- [x] 1.2 In a scratch clone given `[tool_config] locked = true` and an empty `MISE_DATA_DIR`, `mise install` installs every declared tool. Exit 0 → proceed; a refusal → stop and replan (design, Risks).
  - Exit 0 on mise 2026.11.8, `mise.lock` unchanged; `rust`, `cargo:cargo-audit` and an unlocked global `aqua:jqlang/jq` = "latest" (standing in for the machine's ~30 global tools) all accepted.
  - This machine refuses connections to `nodejs.org`, so node 26.13.1 was copied in after `locked` had accepted its lock URL; `npm:@fission-ai/openspec` then installed from its graph.

## 2. Toolchain

- [x] 2.1 `mise.toml` per design, through the mise skill: `mise install uv python@3.14.10`, `mise lock`, the `pypi:zensical` entry carrying a `uv` line, then the strict install. Check: `toolchain-isolation`'s scenarios of the private interpreter, `PATH`, the foreign directory and the caller's environment.
  - Under `locked`, `mise install uv` refuses an unlocked tool, and `mise lock` refuses the unlocked private `python@3.14.10`. So `mise lock uv` ran first, then `mise lock` with `[tool_config]` taken out for that one run and put back.
  - Graph: the audit's 11 packages, digest `sha256:65bc977f…`; the strict install exits 0, and a later plain `mise install` leaves `mise.lock`'s hash as it was.
  - The `python3` shim falls through to Xcode's Python, the `pip` shim refuses, and the runtime's `site-packages` holds only its own pip.
- [x] 2.2 Under a throwaway `MISE_DATA_DIR`, "Fresh clone with unlocked global tools", "Graph missing" and "Graph edited" pass; the real `.mise/locks/` and `mise.lock` stay unchanged by `git diff`.
  - Fresh: exit 0, all 14 tools, `git status` empty; node seeded as in 1.2, the global config again a one-tool stand-in.
  - Missing: both exit 1 on `dependency sidecar …: No such file or directory`; no `installs/pypi-zensical`. Edited: exit 1 on `digest mismatch`.

## 3. Site

- [x] 3.1 `zensical.toml`, `.mise/tasks/docs-build` and `docs-dev`, `docs/assets/logo.svg` copied from the site repository's `icon.svg`, `docs/stylesheets/extra.css`, `docs/templates/main.html`; `.gitignore` gains `/.site/` and `/.pycache/`. Check: "Build", "Dead link", "Page files" and "Shared runtime untouched".
  - Build 0.26 s, `No issues found`; a planted `nowhere.md` fails it with `page does not exist` at its line; nothing under the runtime or the venv is newer.
  - Links in the light scheme take `#b5593a` (4.7:1 on white), since `#d97757` gives 3.1:1; the accent and the dark scheme's links stay `#d97757`. (session; for the operator to strike)
- [x] 3.2 Pages per design: `:::` boxes to admonitions, code groups to tabs, root links to relative, `index.md` to Markdown with `public/banner.svg`; `ver=` in `installation.md` keeps the line the release commit edits. Check: "Sidebar", "Installation", "Index", and `grep -rn ':::' docs` finds nothing.
  - 7 admonitions, 3 tab sets; `ver=2026.11.0` stays on line 19. `grep -rn ':::' docs` matched only the git-ignored `docs/.vitepress/dist/` until 4.1 removed it, and `git grep` finds nothing.
  - Quick start's H1 now reads `Quick start`, matching the sidebar the spec names.
- [x] 3.3 The operator opens `mise run docs-dev`'s site and confirms "Scheme", "Header", "Network log" and "Live edit".
  - Confirmed by the operator; `lsof` listed `127.0.0.1:8000` alone.

## 4. Removals and workflows

- [x] 4.1 Remove `package.json`, `package-lock.json`, `docs/.vitepress/`, the npm entry of `.github/dependabot.yml` and the VitePress lines of `.gitignore`; rewrite `SECURITY.md`'s docs paragraph for Zensical's locked graph. Check: `git grep -n -i -e vitepress -e 'npm run'` finds only `openspec/changes/`; `mise run docs-build` green.
  - `/node_modules/` left `.gitignore` with the VitePress lines, and the local `node_modules/` (154 VitePress packages) went with it.
  - `SECURITY.md` now says Dependabot does not track the graph: a bump is audited as a change of its own.
- [x] 4.2 `docs.yml` and `ci.yml`'s `docs` job per design. Check: actionlint clean; `act` runs both build jobs green, and `ci.yml`'s red on a planted dead link ("Dead link in a pull request"); "Workflow" and "Job log" read off the files and `act`'s log.
  - act 0.2.97, arm64, image `catthehacker/ubuntu:act-24.04`. `ci.yml`'s `docs` job goes green, and red on the planted link. mise 2026.11.2 installed only uv, python 3.14.10 and zensical 0.1.1.
  - In `docs.yml`'s `build` job every step through `docs-build` is green. `Upload artifact` was not checked: act's artifact server rejects `upload-artifact`'s request with `unknown field "mime_type"`. (for the operator to strike)
  - actionlint ran with `-shellcheck=`: the `shellcheck` shim on PATH has no version set in this project.

## 5. Integration

- [x] 5.1 A fresh clone: `mise install`, `mise run docs-build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` and `mise run e2e` green, `git status --short` empty.
  - The clone carried the uncommitted tree as one scratch commit; 308 unit tests and 56 e2e tests passed, clippy was clean, and `git status --short` stayed empty.

## Workflow follow-up

- `openspec archive move-docs-to-zensical -y` within the change's one commit on `develop`, subject `docs: move the site from vitepress to zensical`.
- After the next release publishes the site: "Published" and "README links" are checked on `https://petr-korobeinikov.github.io/claude-shim/`; Dependabot's six alerts close with `package-lock.json`.
