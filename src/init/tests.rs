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
