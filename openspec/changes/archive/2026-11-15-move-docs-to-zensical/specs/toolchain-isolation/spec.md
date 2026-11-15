# Spec Delta

## Purpose

Confines the docs toolchain's Python interpreter and packages to the project,
away from other interpreters, other projects and system paths.

## ADDED Requirements

### Requirement: Tools install only from the committed lock
Every tool `mise.toml` declares SHALL install from `mise.lock`,
and Zensical from its committed graph, wheels only, refusing a missing or edited graph.

#### Scenario: Fresh clone with unlocked global tools
- **WHEN** `mise install` runs on a fresh clone with global tools absent from `mise.lock`
- **THEN** it exits 0, installing every declared tool, `rust` among them, and leaves `mise.lock` unchanged

#### Scenario: Graph missing
- **WHEN** Zensical is uninstalled and `.mise/locks/pypi-zensical` is gone
- **THEN** `mise install` and `mise exec -- zensical --version` exit non-zero, and no Zensical install directory appears

#### Scenario: Graph edited
- **WHEN** Zensical is uninstalled and its `uv.lock` mismatches the recorded digest
- **THEN** `mise install` exits non-zero with `digest mismatch`

### Requirement: Zensical runs on a private interpreter
Zensical SHALL run from its own venv on CPython 3.14.10, without `pip`,
adding nothing to that runtime.

#### Scenario: Venv configuration
- **WHEN** the venv's `pyvenv.cfg` is read
- **THEN** `home` names `installs/python/3.14.10/bin` and `include-system-site-packages` is `false`

#### Scenario: No package manager in the venv
- **WHEN** the venv's `bin` is listed
- **THEN** it holds no `pip`

#### Scenario: Runtime stays clean
- **WHEN** the runtime's `site-packages` is listed after a build
- **THEN** it holds none of the eleven packages of Zensical's graph

### Requirement: No project interpreter reaches PATH
The project environment SHALL expose Zensical's commands,
never the runtime, the venv's `python` or its `pip`.

#### Scenario: Inside the project environment
- **WHEN** `mise exec -- sh -c 'command -v python3; command -v pip'` runs
- **THEN** no path printed lies under the runtime or the venv

#### Scenario: mise does not activate Python
- **WHEN** `mise which python3` runs in the project
- **THEN** it exits non-zero

### Requirement: Zensical does not run outside the project
Zensical SHALL fail outside the project, even with mise's shims on `PATH`.

#### Scenario: Foreign directory
- **WHEN** `zensical --version` runs in a directory outside the project
- **THEN** it exits non-zero

### Requirement: A build writes only inside the project
A docs build SHALL write its bytecode and cache inside the project, ignored by git,
and nothing under the runtime or the venv.

#### Scenario: Shared runtime untouched
- **WHEN** `mise run docs-build` follows a timestamp file
- **THEN** no file under the runtime or the venv is newer, and `git status --short` prints nothing

### Requirement: The caller's environment cannot redirect the tools
The project environment SHALL unset `PYTHONPATH` and `PYTHONHOME`,
refuse `pip` outside a venv, and forbid uv's Python downloads.

#### Scenario: Inherited Python variables
- **WHEN** a shell exporting `PYTHONPATH` runs `mise exec -- printenv PYTHONPATH`
- **THEN** it exits non-zero

#### Scenario: Guard values
- **WHEN** `mise exec -- printenv PIP_REQUIRE_VIRTUALENV UV_PYTHON_DOWNLOADS` runs
- **THEN** it prints `true` and `never`
