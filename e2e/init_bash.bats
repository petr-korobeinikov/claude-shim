#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "precmd exports the active profile, and clears it, across cd" {
    command -v bash >/dev/null || skip "bash not installed"

    "$CLAUDE_SHIM_BIN" profile new bt
    mkdir -p proj plain
    ( cd proj && "$CLAUDE_SHIM_BIN" profile use bt )

    run bash -c '
        eval "$("$CLAUDE_SHIM_BIN" init bash)"
        cd proj;     _claude_shim_precmd; printf "in=%s\n" "$CLAUDE_SHIM_ACTIVE_PROFILE"
        cd ../plain; _claude_shim_precmd; printf "out=%s\n" "$CLAUDE_SHIM_ACTIVE_PROFILE"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == *"in=bt"* ]]
    [[ "$output" == *"out="* ]]
    [[ "$output" != *"out=bt"* ]]
}

@test "the shim dir stays first on PATH after a later prepend" {
    command -v bash >/dev/null || skip "bash not installed"

    run bash -c '
        eval "$("$CLAUDE_SHIM_BIN" init bash)"
        PATH="/opt/decoy:$PATH"
        _claude_shim_precmd
        printf "%s\n" "${PATH%%:*}"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == */claude-shim/shims ]]
}

@test "init bash output sources cleanly and installs the hook first" {
    command -v bash >/dev/null || skip "bash not installed"

    run bash -c '
        eval "$("$CLAUDE_SHIM_BIN" init bash)"
        case "$PROMPT_COMMAND" in
            _claude_shim_precmd*) echo HOOK_FIRST ;;
            *) echo "HOOK_BAD:$PROMPT_COMMAND" ;;
        esac
    '
    [ "$status" -eq 0 ]
    [[ "$output" == *HOOK_FIRST* ]]
}
