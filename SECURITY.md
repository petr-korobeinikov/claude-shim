# Security Policy

## Supported Versions

claude-shim is in pre-release.
Only the latest published release receives security fixes;
older pre-release tags are not maintained.

| Version            | Supported |
| ------------------ | --------- |
| latest release     | ✅        |
| older pre-releases | ❌        |

## Reporting a Vulnerability

Please report security vulnerabilities privately
through GitHub's [private vulnerability reporting][report].
Do not open a public issue for security problems.

You can expect an initial response within a few days.
This is a personal project,
so fixes are best-effort.

[report]: https://github.com/petr-korobeinikov/claude-shim/security/advisories/new

## Dependency Security Posture

The shipped artifact is a single Rust binary.
Its dependencies are audited in CI with [`cargo audit`][cargo-audit].

The documentation site is built with [Zensical][zensical] from this repository
and ships in neither the released binary nor the published site's runtime.
Zensical installs from a dependency graph committed in `.mise/locks/`,
every wheel pinned by its hash,
wheels only, so no package code runs at install time,
into a venv of its own on a private Python.
It runs only at docs-build time, over trusted in-repo Markdown.
Dependabot does not track that graph:
a bump is audited and lands as a change of its own.

[zensical]: https://zensical.org

[cargo-audit]: https://github.com/rustsec/rustsec
