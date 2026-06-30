use std::env;

use directories::BaseDirs;

pub(crate) fn zsh() -> String {
    let exe = env::current_exe().ok().map_or_else(
        || "claude-shim".to_string(),
        |p| p.to_string_lossy().into_owned(),
    );
    ZSH_TEMPLATE
        .replace("__CLAUDE_SHIM_BIN__", &shell_quote(&exe))
        .replace("__CLAUDE_SHIM_SHIMS__", &shell_quote(&shims_dir()))
}

fn shims_dir() -> String {
    BaseDirs::new().map_or_else(
        || "$HOME/.local/share/claude-shim/shims".to_string(),
        |b| {
            b.data_dir()
                .join("claude-shim")
                .join("shims")
                .to_string_lossy()
                .into_owned()
        },
    )
}

fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str(r"'\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

const ZSH_TEMPLATE: &str = r#"# claude-shim zsh integration
# Add ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] } to your PS1.
typeset -g _claude_shim_shims=__CLAUDE_SHIM_SHIMS__

# Keep the shim dir first on PATH on every prompt — survives later
# prepends from mise/brew/sdkman/etc. so the eval line can sit anywhere
# in ~/.zshrc, not strictly at the end.
_claude_shim_ensure_path() {
    path=("$_claude_shim_shims" "${(@)path:#$_claude_shim_shims}")
}
_claude_shim_ensure_path

_claude_shim_precmd() {
    _claude_shim_ensure_path
    local out rc
    if [[ "${_CLAUDE_SHIM_LAST_WARN_PWD-}" == "$PWD" ]]; then
        out=$(__CLAUDE_SHIM_BIN__ profile current 2>/dev/null)
        rc=$?
    else
        out=$(__CLAUDE_SHIM_BIN__ profile current)
        rc=$?
    fi
    if (( rc == 0 )); then
        export CLAUDE_SHIM_ACTIVE_PROFILE="$out"
        unset _CLAUDE_SHIM_LAST_WARN_PWD
    else
        export CLAUDE_SHIM_ACTIVE_PROFILE=""
        _CLAUDE_SHIM_LAST_WARN_PWD="$PWD"
    fi
}
typeset -ag precmd_functions
precmd_functions=(_claude_shim_precmd ${precmd_functions[@]:#_claude_shim_precmd})
"#;

#[cfg(test)]
mod tests;
