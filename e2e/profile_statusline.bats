#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "statusline --preset installs the profile indicator" {
    run "$CLAUDE_SHIM_BIN" profile new p
    local dir
    dir="$(path_after_at "${lines[0]}")"

    run "$CLAUDE_SHIM_BIN" profile statusline --profile p --preset profile-indicator
    [ "$status" -eq 0 ]
    [[ "$output" == *"set statusLine on profile 'p'"* ]]
    grep -q '"statusLine"' "$dir/settings.json"
    grep -q 'Current profile' "$dir/settings.json"
}

@test "statusline installs a custom command" {
    run "$CLAUDE_SHIM_BIN" profile new p
    local dir
    dir="$(path_after_at "${lines[0]}")"

    run "$CLAUDE_SHIM_BIN" profile statusline --profile p 'echo custom-indicator'
    [ "$status" -eq 0 ]
    grep -q 'echo custom-indicator' "$dir/settings.json"
}

@test "statusline with neither a preset nor a command exits 2" {
    "$CLAUDE_SHIM_BIN" profile new p
    run "$CLAUDE_SHIM_BIN" profile statusline --profile p
    [ "$status" -eq 2 ]
}

@test "statusline with both a preset and a command exits 2" {
    "$CLAUDE_SHIM_BIN" profile new p
    run "$CLAUDE_SHIM_BIN" profile statusline --profile p --preset profile-indicator 'echo x'
    [ "$status" -eq 2 ]
}

@test "statusline preserves an existing statusLine without --force" {
    run "$CLAUDE_SHIM_BIN" profile new p
    local dir
    dir="$(path_after_at "${lines[0]}")"

    "$CLAUDE_SHIM_BIN" profile statusline --profile p 'echo first'

    run "$CLAUDE_SHIM_BIN" profile statusline --profile p 'echo second'
    [ "$status" -eq 2 ]
    [[ "$output" == *"already set"* ]]
    grep -q 'echo first' "$dir/settings.json"
    run grep -q 'echo second' "$dir/settings.json"
    [ "$status" -ne 0 ]

    run "$CLAUDE_SHIM_BIN" profile statusline --profile p --force 'echo second'
    [ "$status" -eq 0 ]
    grep -q 'echo second' "$dir/settings.json"
    run grep -q 'echo first' "$dir/settings.json"
    [ "$status" -ne 0 ]
}

@test "statusline without --profile targets the active profile" {
    "$CLAUDE_SHIM_BIN" profile new p
    "$CLAUDE_SHIM_BIN" profile use p

    run "$CLAUDE_SHIM_BIN" profile statusline --preset profile-indicator
    [ "$status" -eq 0 ]
    [[ "$output" == *"set statusLine on profile 'p'"* ]]
}
