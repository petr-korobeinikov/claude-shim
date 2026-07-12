#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "profile use binds the cwd and current resolves it" {
    "$CLAUDE_SHIM_BIN" profile new dev

    run "$CLAUDE_SHIM_BIN" profile use dev
    [ "$status" -eq 0 ]
    [[ "$output" == *"set profile 'dev'"* ]]
    [ -f .claude/claude-shim.json ]

    run "$CLAUDE_SHIM_BIN" profile current
    [ "$status" -eq 0 ]
    [ "$output" = "dev" ]
}

@test "profile use --workspace writes the dir-root marker" {
    "$CLAUDE_SHIM_BIN" profile new dev

    run "$CLAUDE_SHIM_BIN" profile use dev --workspace
    [ "$status" -eq 0 ]
    [ -f .claude-shim.json ]
    [ ! -f .claude/claude-shim.json ]

    run "$CLAUDE_SHIM_BIN" profile current
    [ "$output" = "dev" ]
}

@test "profile use --effort pins the tier on the binding" {
    "$CLAUDE_SHIM_BIN" profile new dev

    run "$CLAUDE_SHIM_BIN" profile use dev --effort low
    [ "$status" -eq 0 ]
    grep -q '"name"' .claude/claude-shim.json
    grep -q '"effort"' .claude/claude-shim.json
    grep -q '"low"' .claude/claude-shim.json
}

@test "profile use on an unknown profile exits 2" {
    run "$CLAUDE_SHIM_BIN" profile use ghost
    [ "$status" -eq 2 ]
    [[ "$output" == *"does not exist"* ]]
}

@test "profile use refuses to clobber an existing binding" {
    "$CLAUDE_SHIM_BIN" profile new dev
    "$CLAUDE_SHIM_BIN" profile use dev

    run "$CLAUDE_SHIM_BIN" profile use dev
    [ "$status" -eq 2 ]
    [[ "$output" == *"marker already exists"* ]]
}
