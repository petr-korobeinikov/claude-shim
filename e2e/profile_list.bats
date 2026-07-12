#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "profile list is empty when no profiles exist" {
    run "$CLAUDE_SHIM_BIN" profile list
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}

@test "profile list tags the default and the active profile separately" {
    "$CLAUDE_SHIM_BIN" profile new base --default
    "$CLAUDE_SHIM_BIN" profile new work
    "$CLAUDE_SHIM_BIN" profile use work

    run "$CLAUDE_SHIM_BIN" profile list
    [ "$status" -eq 0 ]
    [[ "$output" == *"base (default)"* ]]
    [[ "$output" == *"work (active)"* ]]
}

@test "profile list marks the bound profile active" {
    "$CLAUDE_SHIM_BIN" profile new work
    "$CLAUDE_SHIM_BIN" profile use work

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" == *"work (active)"* ]]
}

@test "profile list shows a lone default as both default and active" {
    "$CLAUDE_SHIM_BIN" profile new solo --default

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" == *"solo (default, active)"* ]]
}
