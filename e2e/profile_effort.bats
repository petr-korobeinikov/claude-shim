#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "effort pins the active profile's default and round-trips" {
    run "$CLAUDE_SHIM_BIN" profile new p
    local dir
    dir="$(path_after_at "${lines[0]}")"
    "$CLAUDE_SHIM_BIN" profile use p

    run "$CLAUDE_SHIM_BIN" profile effort high
    [ "$status" -eq 0 ]
    [[ "$output" == *"set effort 'high' on profile 'p'"* ]]

    grep -q '"high"' "$dir/claude-shim.json"
}

@test "effort --profile targets a named profile default" {
    run "$CLAUDE_SHIM_BIN" profile new p
    local dir
    dir="$(path_after_at "${lines[0]}")"

    run "$CLAUDE_SHIM_BIN" profile effort medium --profile p
    [ "$status" -eq 0 ]
    grep -q '"medium"' "$dir/claude-shim.json"
}

@test "effort --local pins this dir's binding and round-trips" {
    "$CLAUDE_SHIM_BIN" profile new p
    "$CLAUDE_SHIM_BIN" profile use p

    run "$CLAUDE_SHIM_BIN" profile effort low --local
    [ "$status" -eq 0 ]
    [[ "$output" == *"set effort 'low' on 'p'"* ]]

    grep -q '"name"' .claude/claude-shim.json
    grep -q '"low"' .claude/claude-shim.json
}

@test "effort --profile together with --local exits 2" {
    "$CLAUDE_SHIM_BIN" profile new p
    "$CLAUDE_SHIM_BIN" profile use p

    run "$CLAUDE_SHIM_BIN" profile effort low --profile p --local
    [ "$status" -eq 2 ]
}

@test "effort with no active profile and no target exits 2" {
    run "$CLAUDE_SHIM_BIN" profile effort high
    [ "$status" -eq 2 ]
    [[ "$output" == *"no active profile"* ]]
}

@test "effort --local with no binding here exits 2" {
    run "$CLAUDE_SHIM_BIN" profile effort high --local
    [ "$status" -eq 2 ]
    [[ "$output" == *"no project or workspace binding"* ]]
}
