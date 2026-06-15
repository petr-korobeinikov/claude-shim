use std::env;

use directories::BaseDirs;

pub(crate) fn zsh() -> String {
    let exe = env::current_exe()
        .ok()
        .map_or_else(|| "claude-shim".to_string(), |p| p.to_string_lossy().into_owned());
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
        out=$(__CLAUDE_SHIM_BIN__ current 2>/dev/null)
        rc=$?
    else
        out=$(__CLAUDE_SHIM_BIN__ current)
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
mod tests {
    use super::*;

    #[test]
    fn shell_quote_wraps_in_single_quotes() {
        assert_eq!(shell_quote("simple"), "'simple'");
    }

    #[test]
    fn shell_quote_empty_string() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn shell_quote_preserves_spaces() {
        assert_eq!(shell_quote("with space"), "'with space'");
    }

    #[test]
    fn shell_quote_escapes_single_quote() {
        assert_eq!(shell_quote("can't"), r"'can'\''t'");
    }

    #[test]
    fn shell_quote_only_a_quote() {
        assert_eq!(shell_quote("'"), r"''\'''");
    }

    #[test]
    fn shell_quote_preserves_other_specials() {
        assert_eq!(shell_quote("$(rm -rf /)"), "'$(rm -rf /)'");
    }

    #[test]
    fn zsh_substitutes_placeholders() {
        let snippet = zsh();
        assert!(
            !snippet.contains("__CLAUDE_SHIM_BIN__"),
            "bin placeholder must be replaced"
        );
        assert!(
            !snippet.contains("__CLAUDE_SHIM_SHIMS__"),
            "shims placeholder must be replaced"
        );
    }

    #[test]
    fn zsh_keeps_shims_first_on_every_prompt() {
        let snippet = zsh();
        assert!(snippet.contains("_claude_shim_shims="));
        assert!(snippet.contains("_claude_shim_ensure_path()"));
        assert!(snippet.contains(r#"path=("$_claude_shim_shims" "${(@)path:#$_claude_shim_shims}")"#));
        // Initial call right after defining the function.
        assert!(snippet.contains("\n_claude_shim_ensure_path\n"));
        // And on every prompt — first line inside _claude_shim_precmd.
        let precmd_idx = snippet
            .find("_claude_shim_precmd() {")
            .expect("precmd defined");
        let after = &snippet[precmd_idx..];
        assert!(
            after
                .lines()
                .take(3)
                .any(|l| l.contains("_claude_shim_ensure_path")),
            "ensure_path call must be inside precmd"
        );
    }

    #[test]
    fn zsh_contains_precmd_hook_and_prepend() {
        let snippet = zsh();
        assert!(snippet.contains("_claude_shim_precmd"));
        assert!(snippet.contains("precmd_functions=(_claude_shim_precmd"));
    }

    #[test]
    fn zsh_exports_profile_var() {
        let snippet = zsh();
        assert!(snippet.contains("export CLAUDE_SHIM_ACTIVE_PROFILE"));
    }
}
