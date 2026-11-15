# docs-site Specification

## Purpose
Builds the documentation site from `docs/` with Zensical, at the URLs it has published.

## Requirements

### Requirement: The site builds from the repository
`mise run docs-build` SHALL build `docs/` into `.site/`, fail on a link to a missing page,
and leave the worktree clean.

#### Scenario: Build
- **WHEN** `mise run docs-build` runs in a fresh clone
- **THEN** it exits 0, `.site/index.html` exists, and `git status --short` prints nothing

#### Scenario: Dead link
- **WHEN** a page links a missing page
- **THEN** the build exits non-zero and names the link

### Requirement: Pages keep their URLs
Each page SHALL be written as `<path>.html`, as VitePress wrote it.

#### Scenario: Page files
- **WHEN** `mise run docs-build` has run
- **THEN** `.site/` holds `guide/installation.html` and `contributing.html`, and no `guide/installation/index.html`

### Requirement: Pages read from simple to deep
The sidebar SHALL list Home; the Guide's Quick start, Installation, Profiles, Effort level, statusLine indicator,
Prompt indicator, Profile resolution and Migration; then Contributing, in that order.

#### Scenario: Sidebar
- **WHEN** a built page is read
- **THEN** its sidebar lists those ten pages in order

### Requirement: Pages keep their callouts and tabs
Each tip, warning and info box SHALL render as a titled callout, and each code group as tabs.

#### Scenario: Installation
- **WHEN** `guide/installation.html` opens
- **THEN** the shell variants are tabs, and no page shows `:::`

### Requirement: The site follows the reader's color scheme
The site SHALL use Zensical's `modern` theme, open light or dark as the reader's system prefers,
and offer a switch to the other that holds across pages.

#### Scenario: Scheme
- **WHEN** a page opens with the system dark, the switch is used, and another page opens
- **THEN** the first renders dark, the second light

### Requirement: The header shows the project icon
The logo and favicon SHALL be the project icon, `#d97757` on `#1a1a1a`, and the accent `#d97757`.

#### Scenario: Header
- **WHEN** a page opens, light and then dark
- **THEN** the header and the tab show the icon, whole

### Requirement: Language models get the site as Markdown
The site SHALL serve `llms.txt` at its root, listing every sidebar page by the absolute URL of its Markdown,
and `llms-full.txt` with their text.

#### Scenario: Index
- **WHEN** the built `llms.txt` and `llms-full.txt` are read
- **THEN** the first links each page as `https://petr-korobeinikov.github.io/claude-shim/<path>.md`, the second holds each page's text once

### Requirement: The site calls no one else
A built page SHALL load nothing from another host,
but the repository's facts the header shows from `api.github.com`, and SHALL search its own pages in the browser.

#### Scenario: Network log
- **WHEN** `docs-dev`'s site opens and `effort` is searched, the network log on
- **THEN** Effort level is offered, and every request goes to `127.0.0.1:8000` or `api.github.com/repos/petr-korobeinikov/claude-shim`

### Requirement: The site reloads as it is edited
`mise run docs-dev` SHALL serve the site to this machine alone
and show each saved edit without a manual reload.

#### Scenario: Live edit
- **WHEN** `mise run docs-dev` runs and an open page's file is saved
- **THEN** `http://127.0.0.1:8000/claude-shim/` shows the edit unprompted, and `lsof -nP -iTCP:8000 -sTCP:LISTEN` lists `127.0.0.1:8000` alone
