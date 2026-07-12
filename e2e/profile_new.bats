#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "profile new creates the dir and seeds CLAUDE.md" {
    run "$CLAUDE_SHIM_BIN" profile new dev
    [ "$status" -eq 0 ]
    [[ "$output" == *"created profile 'dev'"* ]]

    local dir
    dir="$(path_after_at "${lines[0]}")"
    [ -d "$dir" ]
    [ -f "$dir/CLAUDE.md" ]
    grep -q "claude-shim profile" "$dir/CLAUDE.md"
    grep -q "CLAUDE_CONFIG_DIR" "$dir/CLAUDE.md"
}

@test "profile new --default makes it the global default" {
    run "$CLAUDE_SHIM_BIN" profile new dev --default
    [ "$status" -eq 0 ]
    [[ "$output" == *"as the global default"* ]]

    run "$CLAUDE_SHIM_BIN" profile list
    [[ "$output" == *"dev (default"* ]]
}

@test "profile new --statusline writes settings.json with the indicator" {
    run "$CLAUDE_SHIM_BIN" profile new dev --statusline
    [ "$status" -eq 0 ]

    local dir
    dir="$(path_after_at "${lines[0]}")"
    [ -f "$dir/settings.json" ]
    grep -q '"statusLine"' "$dir/settings.json"
    grep -q 'Current profile' "$dir/settings.json"
}

@test "profile new --effort pins the profile default" {
    run "$CLAUDE_SHIM_BIN" profile new dev --effort high
    [ "$status" -eq 0 ]
    [[ "$output" == *"pinned default effort"* ]]

    local dir
    dir="$(path_after_at "${lines[0]}")"
    [ -f "$dir/claude-shim.json" ]
    grep -q '"effort"' "$dir/claude-shim.json"
    grep -q '"high"' "$dir/claude-shim.json"
}

@test "profile new on an existing name exits 2" {
    "$CLAUDE_SHIM_BIN" profile new dev
    run "$CLAUDE_SHIM_BIN" profile new dev
    [ "$status" -eq 2 ]
    [[ "$output" == *"already exists"* ]]
}

@test "profile new rejects a name with path separators" {
    run "$CLAUDE_SHIM_BIN" profile new "foo/bar"
    [ "$status" -eq 2 ]
    [[ "$output" == *"invalid profile name"* ]]
}

@test "profile new rejects a traversal name" {
    run "$CLAUDE_SHIM_BIN" profile new ".."
    [ "$status" -eq 2 ]
    [[ "$output" == *"invalid profile name"* ]]
}
