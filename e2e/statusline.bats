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

CALL_OF_66900='{"type":"assistant","message":{"id":"msg_1","usage":{"input_tokens":900,"cache_creation_input_tokens":60000,"cache_read_input_tokens":500000,"output_tokens":6000}}}'

full_input() {
    local transcript_path="$1" rate_limits="$2"
    printf '{"model":{"id":"claude-opus-5-5","display_name":"Opus 5.5"},"effort":{"level":"high"},"transcript_path":"%s","rate_limits":%s}' \
        "$transcript_path" "$rate_limits"
}

write_transcript() {
    TRANSCRIPT="$BATS_TEST_TMPDIR/session.jsonl"
    printf '%s\n%s\n' "$CALL_OF_66900" "$CALL_OF_66900" > "$TRANSCRIPT"
}

@test "statusline full prints every segment" {
    export CLAUDE_CONFIG_DIR="$WORK_DIR"
    write_transcript
    local rate_limits='{"five_hour":{"used_percentage":23.5},"seven_day":{"used_percentage":41.2}}'

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full <<<"$(full_input "$TRANSCRIPT" "$rate_limits")"
    [ "$status" -eq 0 ]
    [ "$output" = "Profile: work · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Session: 24% · Week: 41%" ]
}

@test "statusline full with the weekly window only holds no session" {
    write_transcript

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full \
        <<<"$(full_input "$TRANSCRIPT" '{"seven_day":{"used_percentage":41}}')"
    [ "$status" -eq 0 ]
    [[ "$output" == *" · Week: 41%" ]]
    [[ "$output" != *"Session:"* ]]
}

@test "statusline full without a transcript holds no tokens" {
    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full \
        <<<"$(full_input "$BATS_TEST_TMPDIR/gone.jsonl" '{}')"
    [ "$status" -eq 0 ]
    [ "$output" = "Model: Opus 5.5 · Effort: high" ]
}

# 30 s past 2d 5h: the countdown still floors to `2d 5h` if the binary
# reads the clock up to 30 s after `date +%s`.
@test "statusline full-week counts down to the weekly reset" {
    local resets_at=$(( $(date +%s) + 2 * 86400 + 5 * 3600 + 30 ))

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full-week \
        <<<"$(full_input "$BATS_TEST_TMPDIR/gone.jsonl" "{\"seven_day\":{\"used_percentage\":41,\"resets_at\":$resets_at}}")"
    [ "$status" -eq 0 ]
    [[ "$output" == *" · Week resets: in 2d 5h" ]]
}

@test "statusline full-week after the reset holds no countdown" {
    local resets_at=$(( $(date +%s) - 1 ))

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full-week \
        <<<"$(full_input "$BATS_TEST_TMPDIR/gone.jsonl" "{\"seven_day\":{\"used_percentage\":41,\"resets_at\":$resets_at}}")"
    [ "$status" -eq 0 ]
    [ "$output" = "Model: Opus 5.5 · Effort: high · Week: 41%" ]
}

@test "statusline full under NO_COLOR holds no escape" {
    export CLAUDE_CONFIG_DIR="$WORK_DIR"
    write_transcript
    local input
    input="$(full_input "$TRANSCRIPT" '{"seven_day":{"used_percentage":41}}')"

    run "$CLAUDE_SHIM_BIN" statusline full <<<"$input"
    [[ "$output" == *$'\e[2m'* ]]

    NO_COLOR=1 run "$CLAUDE_SHIM_BIN" statusline full <<<"$input"
    [ "$status" -eq 0 ]
    [[ "$output" != *$'\e'* ]]
}
