use std::env;
use std::path::Path;

use directories::BaseDirs;

use crate::cli::Target;
use crate::render::shell_quote;

pub(crate) fn snippet(target: Target) -> String {
    let exe = env::current_exe().ok().map_or_else(
        || "claude-shim".to_string(),
        |p| p.to_string_lossy().into_owned(),
    );
    let template = match target {
        Target::Zsh => ZSH_TEMPLATE,
        Target::Bash => BASH_TEMPLATE,
    };
    template
        .replace("__CLAUDE_SHIM_BIN__", &shell_quote(&exe))
        .replace("__CLAUDE_SHIM_SHIMS__", &shims_dir())
}

fn shims_dir() -> String {
    let base = BaseDirs::new();
    render_shims_dir(base.as_ref().map(BaseDirs::data_dir))
}

fn render_shims_dir(data_dir: Option<&Path>) -> String {
    match data_dir {
        Some(dir) => shell_quote(&dir.join("claude-shim").join("shims").to_string_lossy()),
        None => format!(
            "\"${{HOME:-}}\"/{}",
            shell_quote(".local/share/claude-shim/shims")
        ),
    }
}

const ZSH_TEMPLATE: &str = r#"# claude-shim zsh integration
# Add ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] } to your PS1.
typeset -g _claude_shim_shims=__CLAUDE_SHIM_SHIMS__

# Keep the shim dir first on PATH on every prompt — survives later
# prepends from mise/brew/sdkman/etc. so the eval line can sit anywhere
# in ~/.zshrc, not strictly at the end.
_claude_shim_ensure_path() {
    path=("$_claude_shim_shims" "${(@)path:#"$_claude_shim_shims"}")
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

const BASH_TEMPLATE: &str = r#"# claude-shim bash integration
# Add ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] } to your PS1.
_claude_shim_shims=__CLAUDE_SHIM_SHIMS__

# Keep the shim dir first on PATH on every prompt — survives later
# prepends from mise/brew/sdkman/etc. so the eval line can sit anywhere
# in ~/.bashrc, not strictly at the end. Pure bash 3.2: string dedup, no
# ${path} array (zsh-only) and no PROMPT_COMMAND array (bash 5.1+).
_claude_shim_ensure_path() {
    local rest=":$PATH:"
    rest="${rest//:"$_claude_shim_shims":/:}"
    rest="${rest#:}"
    rest="${rest%:}"
    PATH="$_claude_shim_shims${rest:+:$rest}"
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
case "${PROMPT_COMMAND-}" in
    *_claude_shim_precmd*) ;;
    *) PROMPT_COMMAND="_claude_shim_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}" ;;
esac
"#;

#[cfg(test)]
mod tests;
