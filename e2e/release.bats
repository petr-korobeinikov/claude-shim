#!/usr/bin/env bats

load helpers/release

setup() { _release_setup; }
teardown() { _release_teardown; }

release_commit() {
    git log -1 --format=%H --fixed-strings --grep="chore: release $1" main
}

numstat() {
    git show --numstat --format= "$1" | LC_ALL=C sort
}

# A commit origin has and the local branch lacks: push one, then drop it locally.
put_origin_ahead() {
    git checkout -q "$1"
    git commit -q --allow-empty -m "chore: elsewhere"
    git push -q origin "$1"
    git reset -q --hard HEAD~1
    git checkout -q develop
}

@test "release commits the version into all three files" {
    local start
    start="$(git rev-parse develop)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 0 ]

    local commit
    commit="$(release_commit "$MONTH.0")"
    [ "$(git log -1 --format=%s "$commit")" = "chore: release $MONTH.0" ]
    [ "$(git rev-list --no-merges --count "$start..main")" -eq 1 ]
    [ "$(numstat "$commit")" = "$(printf '1\t1\tCargo.lock\n1\t1\tCargo.toml\n1\t1\tdocs/guide/installation.md')" ]
    grep -qx "version = \"$MONTH.0\"" Cargo.toml
    grep -qx "ver=$MONTH.0" docs/guide/installation.md
}

@test "release --beta leaves the install guide alone" {
    run "$TASKS_DIR/release" --beta
    [ "$status" -eq 0 ]

    local commit
    commit="$(release_commit "$MONTH.0-beta.1")"
    [ "$(git log -1 --format=%s "$commit")" = "chore: release $MONTH.0-beta.1" ]
    [ "$(numstat "$commit")" = "$(printf '1\t1\tCargo.lock\n1\t1\tCargo.toml')" ]
    grep -qx "version = \"$MONTH.0-beta.1\"" Cargo.toml
}

@test "release lands tagged on main, merged into develop and unpushed" {
    local origin_before
    origin_before="$(origin_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 0 ]

    local name="$MONTH.0"
    [ "$(git rev-parse "v$name^{commit}")" = "$(git rev-parse main)" ]
    [ "$(git cat-file -t "v$name")" = tag ]
    [ "$(git tag -l --format='%(contents:subject)' "v$name")" = "v$name" ]
    git merge-base --is-ancestor main develop
    ! git rev-parse -q --verify "refs/heads/release/$name"
    [ "$(git branch --show-current)" = develop ]
    [ "$(origin_state)" = "$origin_before" ]
    [ "${lines[${#lines[@]}-1]}" = "git push origin develop main v$name" ]
}

@test "release refuses off develop" {
    git checkout -q main
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 1 ]
    [[ "$output" == *"not on develop"* ]]
    [ "$(local_state)" = "$before" ]
}

@test "release refuses a changed tree" {
    echo "# local edit" >> Cargo.toml
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 1 ]
    [[ "$output" == *"the tree has changes"* ]]
    [ "$(local_state)" = "$before" ]
}

@test "release refuses while a release branch is open" {
    git branch release/2000.1.0
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 1 ]
    [[ "$output" == *"release/2000.1.0 is open"* ]]
    [ "$(local_state)" = "$before" ]
}

@test "release refuses a develop behind origin" {
    put_origin_ahead develop
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 1 ]
    [[ "$output" == *"develop lacks commits of origin/develop"* ]]
    [ "$(local_state)" = "$before" ]
}

@test "release refuses a main behind origin" {
    put_origin_ahead main
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release"
    [ "$status" -eq 1 ]
    [[ "$output" == *"main lacks commits of origin/main"* ]]
    [ "$(local_state)" = "$before" ]
}

@test "release proceeds with a develop ahead of origin" {
    git commit -q --allow-empty -m "chore: not pushed yet"

    run "$TASKS_DIR/release"
    [ "$status" -eq 0 ]
    git rev-parse -q --verify "refs/tags/v$MONTH.0"
}

@test "release refuses an unknown argument" {
    local before
    before="$(local_state)"

    run "$TASKS_DIR/release" --push
    [ "$status" -eq 2 ]
    [[ "$output" == *"--push"* ]]
    [ "$(local_state)" = "$before" ]
}
