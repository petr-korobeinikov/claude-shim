#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "claude-shim --version identifies the binary" {
    run "$CLAUDE_SHIM_BIN" --version
    [ "$status" -eq 0 ]
    [[ "$output" == claude-shim* ]]
}

@test "profile new writes inside the isolated HOME" {
    run "$CLAUDE_SHIM_BIN" profile new smoke
    [ "$status" -eq 0 ]
    [[ "$output" == *"created profile 'smoke'"* ]]
    # The data dir lives under HOME, so the confirmation shortens it to ~/….
    [[ "$output" == *"at ~/"* ]]

    local created
    created="$(path_after_at "${lines[0]}")"
    [[ "$created" == "$HOME"/* ]]
    [ -d "$created" ]
}
