#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "shim forwards args, exports the config dir, injects effort, propagates exit" {
    run "$CLAUDE_SHIM_BIN" profile new foo --effort medium
    local dir
    dir="$(path_after_at "${lines[0]}")"
    "$CLAUDE_SHIM_BIN" profile use foo

    make_shim
    make_stub_claude
    export PATH="$(stub_path)"
    export STUB_EXIT=7

    run "$SHIM" chat --model x
    [ "$status" -eq 7 ]
    [[ "$output" == *"args: chat --model x"* ]]
    [[ "$output" == *"CLAUDE_CONFIG_DIR=$dir"* ]]
    [[ "$output" == *"CLAUDE_CODE_EFFORT_LEVEL=medium"* ]]
}

@test "shim never clobbers an effort already in the environment" {
    "$CLAUDE_SHIM_BIN" profile new foo --effort medium
    "$CLAUDE_SHIM_BIN" profile use foo

    make_shim
    make_stub_claude
    export PATH="$(stub_path)"
    export CLAUDE_CODE_EFFORT_LEVEL=high

    run "$SHIM"
    [ "$status" -eq 0 ]
    [[ "$output" == *"CLAUDE_CODE_EFFORT_LEVEL=high"* ]]
}

@test "shim refuses when no profile is in scope" {
    make_shim
    make_stub_claude
    export PATH="$(stub_path)"

    run "$SHIM"
    [ "$status" -eq 2 ]
    [[ "$output" == *"no profile in scope"* ]]
}

@test "shim refuses when the bound profile's dir is missing" {
    run "$CLAUDE_SHIM_BIN" profile new ghost
    local dir
    dir="$(path_after_at "${lines[0]}")"
    "$CLAUDE_SHIM_BIN" profile use ghost
    rm -rf "$dir"

    make_shim
    make_stub_claude
    export PATH="$(stub_path)"

    run "$SHIM"
    [ "$status" -eq 2 ]
    [[ "$output" == *"configured but missing"* ]]
}

@test "shim refuses when no real claude is on PATH" {
    "$CLAUDE_SHIM_BIN" profile new foo
    "$CLAUDE_SHIM_BIN" profile use foo

    make_shim
    mkdir -p "$HOME/empty"
    export PATH="$HOME/empty"

    run "$SHIM"
    [ "$status" -eq 2 ]
    [[ "$output" == *"not found on PATH"* ]]
}
