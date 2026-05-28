use std::env;

pub fn zsh() -> String {
    let exe = env::current_exe()
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "claudectl".to_string());
    ZSH_TEMPLATE.replace("__CLAUDECTL_BIN__", &shell_quote(&exe))
}

pub fn oh_my_posh() -> String {
    OH_MY_POSH_SNIPPET.to_string()
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
_claudectl_precmd() {
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

const OH_MY_POSH_SNIPPET: &str = r#"{
  "type": "text",
  "style": "plain",
  "template": "{{ if .Env.CLAUDECTL_ACTIVE_PROFILE }}[{{ .Env.CLAUDECTL_ACTIVE_PROFILE }}] {{ end }}"
}
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
    fn zsh_substitutes_placeholder() {
        let snippet = zsh();
        assert!(
            !snippet.contains("__CLAUDECTL_BIN__"),
            "placeholder must be replaced"
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

    #[test]
    fn oh_my_posh_is_a_text_segment() {
        let snippet = oh_my_posh();
        assert!(snippet.trim_start().starts_with('{'));
        assert!(snippet.contains(r#""type": "text""#));
        assert!(snippet.contains(".Env.CLAUDECTL_ACTIVE_PROFILE"));
    }
}
