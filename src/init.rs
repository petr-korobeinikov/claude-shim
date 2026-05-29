use std::env;

use directories::BaseDirs;

pub(crate) fn zsh() -> String {
    let exe = env::current_exe()
        .ok()
        .map_or_else(|| "claudectl".to_string(), |p| p.to_string_lossy().into_owned());
    ZSH_TEMPLATE
        .replace("__CLAUDECTL_BIN__", &shell_quote(&exe))
        .replace("__CLAUDECTL_SHIMS__", &shell_quote(&shims_dir()))
}

fn shims_dir() -> String {
    BaseDirs::new().map_or_else(
        || "$HOME/.local/share/claudectl/shims".to_string(),
        |b| {
            b.data_dir()
                .join("claudectl")
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

const ZSH_TEMPLATE: &str = r#"# claudectl zsh integration
# Add ${CLAUDECTL_ACTIVE_PROFILE:+[$CLAUDECTL_ACTIVE_PROFILE] } to your PS1.
typeset -g _claudectl_shims=__CLAUDECTL_SHIMS__

# Keep the shim dir first on PATH on every prompt — survives later
# prepends from mise/brew/sdkman/etc. so the eval line can sit anywhere
# in ~/.zshrc, not strictly at the end.
_claudectl_ensure_path() {
    path=("$_claudectl_shims" "${(@)path:#$_claudectl_shims}")
}
_claudectl_ensure_path

_claudectl_precmd() {
    _claudectl_ensure_path
    local out rc
    if [[ "${_CLAUDECTL_LAST_WARN_PWD-}" == "$PWD" ]]; then
        out=$(__CLAUDECTL_BIN__ current 2>/dev/null)
        rc=$?
    else
        out=$(__CLAUDECTL_BIN__ current)
        rc=$?
    fi
    if (( rc == 0 )); then
        export CLAUDECTL_ACTIVE_PROFILE="$out"
        unset _CLAUDECTL_LAST_WARN_PWD
    else
        export CLAUDECTL_ACTIVE_PROFILE=""
        _CLAUDECTL_LAST_WARN_PWD="$PWD"
    fi
}
typeset -ag precmd_functions
precmd_functions=(_claudectl_precmd ${precmd_functions[@]:#_claudectl_precmd})
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
            !snippet.contains("__CLAUDECTL_BIN__"),
            "bin placeholder must be replaced"
        );
        assert!(
            !snippet.contains("__CLAUDECTL_SHIMS__"),
            "shims placeholder must be replaced"
        );
    }

    #[test]
    fn zsh_keeps_shims_first_on_every_prompt() {
        let snippet = zsh();
        assert!(snippet.contains("_claudectl_shims="));
        assert!(snippet.contains("_claudectl_ensure_path()"));
        assert!(snippet.contains(r#"path=("$_claudectl_shims" "${(@)path:#$_claudectl_shims}")"#));
        // Initial call right after defining the function.
        assert!(snippet.contains("\n_claudectl_ensure_path\n"));
        // And on every prompt — first line inside _claudectl_precmd.
        let precmd_idx = snippet
            .find("_claudectl_precmd() {")
            .expect("precmd defined");
        let after = &snippet[precmd_idx..];
        assert!(
            after
                .lines()
                .take(3)
                .any(|l| l.contains("_claudectl_ensure_path")),
            "ensure_path call must be inside precmd"
        );
    }

    #[test]
    fn zsh_contains_precmd_hook_and_prepend() {
        let snippet = zsh();
        assert!(snippet.contains("_claudectl_precmd"));
        assert!(snippet.contains("precmd_functions=(_claudectl_precmd"));
    }

    #[test]
    fn zsh_exports_profile_var() {
        let snippet = zsh();
        assert!(snippet.contains("export CLAUDECTL_ACTIVE_PROFILE"));
    }
}
