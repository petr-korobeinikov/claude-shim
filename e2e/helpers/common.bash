#!/usr/bin/env bash
# Shared setup/teardown for the claude-shim end-to-end tests. Plain bats.

_common_setup() {
    CLAUDE_SHIM_BIN="${CLAUDE_SHIM_BIN:-$BATS_TEST_DIRNAME/../target/debug/claude-shim}"
    if [[ ! -x "$CLAUDE_SHIM_BIN" ]]; then
        echo "binary not found at $CLAUDE_SHIM_BIN — run 'cargo build' first" >&2
        return 1
    fi
    export CLAUDE_SHIM_BIN   # reachable by name inside the child shells tests spawn

    # Isolate every dir the tool derives from the environment: HOME (macOS) and
    # XDG_* pinned under it (Linux, regardless of inherited values).
    TEST_HOME="$(mktemp -d "${BATS_TMPDIR}/claude-shim-e2e.XXXXXX")"
    export HOME="$TEST_HOME"
    export XDG_DATA_HOME="$TEST_HOME/.local/share"
    export XDG_CONFIG_HOME="$TEST_HOME/.config"
    mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"

    # The developer runs claude-shim too; clear its env or an inherited value skews the shim assertions.
    unset CLAUDE_CONFIG_DIR CLAUDE_CODE_EFFORT_LEVEL CLAUDE_SHIM_ACTIVE_PROFILE

    # Below HOME: the resolver walks ancestors up to HOME — a marker here is in reach, the real checkout's is not.
    WORKDIR="$HOME/work"
    mkdir -p "$WORKDIR"
    cd "$WORKDIR" || return 1
}

_common_teardown() {
    export PATH="/usr/bin:/bin:${PATH}"   # a shim test may have narrowed PATH; restore before rm
    cd "$BATS_TEST_DIRNAME" || true        # leave the temp tree before removing it
    [[ -n "${TEST_HOME:-}" ]] && rm -rf "$TEST_HOME"
}

path_after_at() {
    printf '%s\n' "${1##* at }"
}

make_shim() {
    SHIM_DIR="$HOME/shim-bin"
    mkdir -p "$SHIM_DIR"
    ln -sf "$CLAUDE_SHIM_BIN" "$SHIM_DIR/claude"
    SHIM="$SHIM_DIR/claude"
}

# A fake `claude` that echoes what the shim passed it and exits STUB_EXIT, so shim tests never reach the real one.
make_stub_claude() {
    STUB_DIR="$HOME/stub-bin"
    mkdir -p "$STUB_DIR"
    cat > "$STUB_DIR/claude" <<'STUB'
#!/usr/bin/env bash
echo "args: $*"
echo "CLAUDE_CONFIG_DIR=${CLAUDE_CONFIG_DIR-<unset>}"
echo "CLAUDE_CODE_EFFORT_LEVEL=${CLAUDE_CODE_EFFORT_LEVEL-<unset>}"
exit "${STUB_EXIT:-0}"
STUB
    chmod +x "$STUB_DIR/claude"
}

stub_path() {
    printf '%s\n' "$STUB_DIR:/usr/bin:/bin"
}
