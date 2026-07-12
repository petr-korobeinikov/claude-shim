#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "profile delete without --yes previews and removes nothing" {
    "$CLAUDE_SHIM_BIN" profile new dev

    run "$CLAUDE_SHIM_BIN" profile delete dev
    [ "$status" -eq 2 ]
    [[ "$output" == *"would remove profile 'dev'"* ]]

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" == *"dev"* ]]
}

@test "profile delete --yes removes the profile" {
    "$CLAUDE_SHIM_BIN" profile new dev

    run "$CLAUDE_SHIM_BIN" profile delete dev --yes
    [ "$status" -eq 0 ]
    [[ "$output" == *"deleted profile 'dev'"* ]]

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" != *"dev"* ]]
}

@test "deleting the default clears it and leaves current/list clean" {
    "$CLAUDE_SHIM_BIN" profile new dev --default

    run "$CLAUDE_SHIM_BIN" profile delete dev --yes
    [ "$status" -eq 0 ]
    [[ "$output" == *"cleared the global default marker"* ]]

    run "$CLAUDE_SHIM_BIN" profile list
    [ -z "$output" ]

    run "$CLAUDE_SHIM_BIN" profile current
    [ "$status" -eq 0 ]
    [ -z "$output" ]
}

@test "deleting a bound profile drops it from list and current fails loud" {
    "$CLAUDE_SHIM_BIN" profile new dev
    "$CLAUDE_SHIM_BIN" profile use dev

    run "$CLAUDE_SHIM_BIN" profile delete dev --yes
    [ "$status" -eq 0 ]

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" != *"dev"* ]]

    run "$CLAUDE_SHIM_BIN" profile current
    [ "$status" -eq 2 ]
    [[ "$output" == *"referenced but"* ]]
}

@test "profile delete on a missing profile exits 2 either way" {
    run "$CLAUDE_SHIM_BIN" profile delete ghost
    [ "$status" -eq 2 ]
    [[ "$output" == *"does not exist"* ]]

    run "$CLAUDE_SHIM_BIN" profile delete ghost --yes
    [ "$status" -eq 2 ]
    [[ "$output" == *"does not exist"* ]]
}
