#!/usr/bin/env bats

load helpers/common

setup() {
    _common_setup
    run "$CLAUDE_SHIM_BIN" profile new work
    WORK_DIR="$(path_after_at "${lines[0]}")"
}
teardown() { _common_teardown; }

EVERY_SEGMENT='{"model":{"id":"claude-opus-5-5","display_name":"Opus 5.5"},"effort":{"level":"high"}}'

@test "statusline prints an empty line for garbage when no profile is found" {
    run "$CLAUDE_SHIM_BIN" statusline compact <<<"not json"
    [ "$status" -eq 0 ]
    [ "$output" = "" ]
}

@test "statusline with an unknown preset exits 2" {
    run "$CLAUDE_SHIM_BIN" statusline fancy <<<"{}"
    [ "$status" -eq 2 ]
}

@test "statusline names the session profile over the cwd binding" {
    "$CLAUDE_SHIM_BIN" profile new home
    "$CLAUDE_SHIM_BIN" profile use home

    CLAUDE_CONFIG_DIR="$WORK_DIR" run "$CLAUDE_SHIM_BIN" statusline compact <<<"{}"
    [ "$status" -eq 0 ]
    [ "$output" = "work" ]
}

@test "statusline falls back to the cwd binding for a foreign config dir" {
    "$CLAUDE_SHIM_BIN" profile new home
    "$CLAUDE_SHIM_BIN" profile use home
    mkdir "$HOME/other"

    CLAUDE_CONFIG_DIR="$HOME/other" run "$CLAUDE_SHIM_BIN" statusline compact <<<"{}"
    [ "$status" -eq 0 ]
    [ "$output" = "home" ]
}

@test "statusline compact and labeled print every segment" {
    export CLAUDE_CONFIG_DIR="$WORK_DIR"

    run "$CLAUDE_SHIM_BIN" statusline compact <<<"$EVERY_SEGMENT"
    [ "$output" = "work · Opus 5.5 · high" ]

    run "$CLAUDE_SHIM_BIN" statusline labeled <<<"$EVERY_SEGMENT"
    [ "$output" = "Profile: work · Model: Opus 5.5 (high)" ]
}

@test "statusline compact and labeled leave out a missing effort" {
    export CLAUDE_CONFIG_DIR="$WORK_DIR"
    local no_effort='{"model":{"display_name":"Haiku 5.5"}}'

    run "$CLAUDE_SHIM_BIN" statusline compact <<<"$no_effort"
    [ "$output" = "work · Haiku 5.5" ]

    run "$CLAUDE_SHIM_BIN" statusline labeled <<<"$no_effort"
    [ "$output" = "Profile: work · Model: Haiku 5.5" ]
}

@test "statusline colored paints Fable at max" {
    local fable_at_max='{"model":{"id":"claude-fable-5-1","display_name":"Fable 5.1"},"effort":{"level":"max"}}'

    run "$CLAUDE_SHIM_BIN" statusline colored <<<"$fable_at_max"
    [ "$status" -eq 0 ]
    [[ "$output" == *$'\e[1;38;5;208mFable 5.1\e[0m'* ]]
    [[ "$output" == *$'\e[1;31mmax\e[0m'* ]]
}

@test "statusline colored under NO_COLOR prints the labeled line" {
    export CLAUDE_CONFIG_DIR="$WORK_DIR"

    run "$CLAUDE_SHIM_BIN" statusline labeled <<<"$EVERY_SEGMENT"
    local labeled="$output"

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline colored <<<"$EVERY_SEGMENT"
    [ "$status" -eq 0 ]
    [ "$output" = "$labeled" ]
    [[ "$output" != *$'\e'* ]]
}
