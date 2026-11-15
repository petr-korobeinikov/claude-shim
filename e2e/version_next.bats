#!/usr/bin/env bats

load helpers/release

setup() { _release_setup; }
teardown() { _release_teardown; }

@test "version:next starts the month at 0" {
    git tag v2000.1.5   # another month's number does not count

    run "$TASKS_DIR/version/next"
    [ "$status" -eq 0 ]
    [ "$output" = "$MONTH.0" ]
}

@test "version:next skips numbers taken by tags and open branches" {
    git tag "v$MONTH.0"
    git branch "hotfix/$MONTH.1"

    run "$TASKS_DIR/version/next"
    [ "$status" -eq 0 ]
    [ "$output" = "$MONTH.2" ]
}

@test "version:next leaves a beta's number to its final release" {
    git tag "v$MONTH.0"
    git tag "v$MONTH.1-beta.1"

    run "$TASKS_DIR/version/next"
    [ "$status" -eq 0 ]
    [ "$output" = "$MONTH.1" ]

    run "$TASKS_DIR/version/next" --beta
    [ "$status" -eq 0 ]
    [ "$output" = "$MONTH.1-beta.2" ]
}
