#!/usr/bin/env bash
# Sandbox for the release task tests: a repository shaped like the checkout, with a bare origin. Plain bats.

_release_setup() {
    TASKS_DIR="$BATS_TEST_DIRNAME/../.mise/tasks"
    SANDBOX="$(mktemp -d "${BATS_TMPDIR}/claude-shim-release.XXXXXX")"

    # The developer's own config may sign commits and tags, which would prompt or fail here.
    printf '[user]\n\tname = Release Test\n\temail = release@example.com\n[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n' \
        > "$SANDBOX/gitconfig"
    export GIT_CONFIG_GLOBAL="$SANDBOX/gitconfig" GIT_CONFIG_NOSYSTEM=1
    # Any editor prompt fails the test instead of hanging it.
    export GIT_EDITOR=false

    ORIGIN="$SANDBOX/origin.git"
    git init -q --bare "$ORIGIN"
    git init -q -b main "$SANDBOX/work"
    cd "$SANDBOX/work" || return 1

    cp "$BATS_TEST_DIRNAME/../.gitflow" .gitflow
    printf '[package]\nname = "claude-shim"\nversion = "2026.1.0"\nedition = "2024"\n\n[dependencies]\n' > Cargo.toml
    mkdir -p src docs/guide
    : > src/main.rs
    printf '# Installation\n\nver=2026.1.0\n' > docs/guide/installation.md
    cargo generate-lockfile --offline --quiet
    git add -A
    git commit -q -m "chore: start"
    git branch develop
    git checkout -q develop
    git flow config sync > /dev/null
    git remote add origin "$ORIGIN"
    git push -q origin main develop

    MONTH="$(date -u +%Y.%-m)"
}

_release_teardown() {
    cd "$BATS_TEST_DIRNAME" || true
    [[ -n "${SANDBOX:-}" ]] && rm -rf "$SANDBOX"
}

# Every local branch and tag, and the tree: what a refused release must leave as it was.
local_state() {
    git for-each-ref refs/heads refs/tags
    git status --porcelain
}

origin_state() {
    git -C "$ORIGIN" for-each-ref
}
