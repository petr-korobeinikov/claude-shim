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

The documentation site (VitePress) is built from this repository
but ships in neither the released binary nor the published site's runtime.
Its dev-only documentation toolchain dependencies
may carry advisories that do not reach users of the binary:
they run only at docs-build time, over trusted in-repo Markdown.
Dependabot tracks them
and opens update PRs as upstream fixes land.

[cargo-audit]: https://github.com/rustsec/rustsec
