#!/usr/bin/env bats

load helpers/common

setup() { _common_setup; }
teardown() { _common_teardown; }

@test "precmd exports the active profile, and clears it, across cd" {
    command -v zsh >/dev/null || skip "zsh not installed"

    "$CLAUDE_SHIM_BIN" profile new zt
    mkdir -p proj plain
    ( cd proj && "$CLAUDE_SHIM_BIN" profile use zt )

    run zsh -c '
        eval "$("$CLAUDE_SHIM_BIN" init zsh)"
        cd proj;     _claude_shim_precmd; print -r -- "in=$CLAUDE_SHIM_ACTIVE_PROFILE"
        cd ../plain; _claude_shim_precmd; print -r -- "out=$CLAUDE_SHIM_ACTIVE_PROFILE"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == *"in=zt"* ]]
    [[ "$output" == *"out="* ]]
    [[ "$output" != *"out=zt"* ]]
}

@test "the shim dir stays first on PATH after a later prepend" {
    command -v zsh >/dev/null || skip "zsh not installed"

    run zsh -c '
        eval "$("$CLAUDE_SHIM_BIN" init zsh)"
        path=(/opt/decoy $path)
        _claude_shim_precmd
        print -r -- "${path[1]}"
    '
    [ "$status" -eq 0 ]
    [[ "$output" == */claude-shim/shims ]]
}

@test "the dedup stays literal when the shims dir path holds a glob metacharacter" {
    command -v zsh >/dev/null || skip "zsh not installed"

    export HOME="$TEST_HOME/ho*me"
    export XDG_DATA_HOME="$HOME/.local/share"
    mkdir -p "$XDG_DATA_HOME"

    cat > "$TEST_HOME/probe.zsh" <<'PROBE'
[[ -n "$PROBE_OPTS" ]] && setopt ${=PROBE_OPTS}
eval "$("$CLAUDE_SHIM_BIN" init zsh)"
decoy="${_claude_shim_shims//\*/ZZZ}"
typeset -a path=("$decoy" /usr/bin /bin)
_claude_shim_ensure_path
_claude_shim_ensure_path
_claude_shim_ensure_path
if (( ${path[(Ie)$decoy]} )); then kept=kept; else kept=lost; fi
print -r -- "${#path[@]} decoy=$kept first=${path[1]}"
PROBE

    for opts in "" "glob_subst" "glob_subst extended_glob"; do
        PROBE_OPTS="$opts" run zsh -f "$TEST_HOME/probe.zsh"
        [ "$status" -eq 0 ]
        if [[ "$output" != "4 decoy=kept first="*/claude-shim/shims ]]; then
            echo "opts='${opts:-<default>}': expected 4 entries, decoy kept, shims first; got '$output'" >&2
            return 1
        fi
    done
}

@test "init with an unknown target exits 2" {
    run "$CLAUDE_SHIM_BIN" init bogus
    [ "$status" -eq 2 ]
}
