#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "precmd exports the active profile, and clears it, across cd" {
    command -v zsh >/dev/null || skip "zsh not installed"

    "$CLAUDE_SHIM_BIN" profile new zt
    mkdir -p proj plain
    ( cd proj && "$CLAUDE_SHIM_BIN" profile use zt )

    run zsh -c '
        eval "$("$CLAUDE_SHIM_BIN" init zsh)"
        cd proj;     _claude_shim_precmd; print -r -- "in=$CLAUDE_SHIM_ACTIVE_PROFILE"
        cd ../plain; _claude_shim_precmd; print -r -- "out=$CLAUDE_SHIM_ACTIVE_PROFILE"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == *"in=zt"* ]]
    [[ "$output" == *"out="* ]]
    [[ "$output" != *"out=zt"* ]]
}

@test "the shim dir stays first on PATH after a later prepend" {
    command -v zsh >/dev/null || skip "zsh not installed"

    run zsh -c '
        eval "$("$CLAUDE_SHIM_BIN" init zsh)"
        path=(/opt/decoy $path)
        _claude_shim_precmd
        print -r -- "${path[1]}"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == */claude-shim/shims ]]
}

@test "init with an unknown target exits 2" {
    run "$CLAUDE_SHIM_BIN" init bogus
    [ "$status" -eq 2 ]
}
