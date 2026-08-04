use std::path::Path;

use super::*;
use crate::cli::Target;

#[test]
fn zsh_substitutes_placeholders() {
    let snippet = snippet(Target::Zsh);
    assert!(
        !snippet.contains("__CLAUDE_SHIM_BIN__"),
        "bin placeholder must be replaced"
    );
    assert!(
        !snippet.contains("__CLAUDE_SHIM_SHIMS__"),
        "shims placeholder must be replaced"
    );
    assert!(
        snippet.contains(&format!("_claude_shim_shims={}\n", shims_dir())),
        "shims word must be substituted verbatim, not re-quoted"
    );
}

#[test]
fn zsh_keeps_shims_first_on_every_prompt() {
    let snippet = snippet(Target::Zsh);
    assert!(snippet.contains("_claude_shim_shims="));
    assert!(snippet.contains("_claude_shim_ensure_path()"));
    assert!(snippet.contains(r#"path=("$_claude_shim_shims" "#));
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
fn zsh_dedup_pattern_treats_the_path_as_a_literal() {
    // The pattern half of ${(@)path:#pat} goes glob-active under GLOB_SUBST; the
    // path must be quoted there so a shims dir containing * ? [ dedups literally
    // instead of dropping unrelated PATH entries.
    let snippet = snippet(Target::Zsh);
    assert!(snippet.contains(r#""${(@)path:#"$_claude_shim_shims"}""#));
}

#[test]
fn zsh_contains_precmd_hook_and_prepend() {
    let snippet = snippet(Target::Zsh);
    assert!(snippet.contains("_claude_shim_precmd"));
    assert!(snippet.contains("precmd_functions=(_claude_shim_precmd"));
}

#[test]
fn zsh_exports_profile_var() {
    let snippet = snippet(Target::Zsh);
    assert!(snippet.contains("export CLAUDE_SHIM_ACTIVE_PROFILE"));
}

#[test]
fn zsh_and_bash_snippets_differ() {
    assert_ne!(snippet(Target::Zsh), snippet(Target::Bash));
}

#[test]
fn bash_substitutes_placeholders() {
    let snippet = snippet(Target::Bash);
    assert!(
        !snippet.contains("__CLAUDE_SHIM_BIN__"),
        "bin placeholder must be replaced"
    );
    assert!(
        !snippet.contains("__CLAUDE_SHIM_SHIMS__"),
        "shims placeholder must be replaced"
    );
    assert!(
        snippet.contains(&format!("_claude_shim_shims={}\n", shims_dir())),
        "shims word must be substituted verbatim, not re-quoted"
    );
}

#[test]
fn bash_keeps_shims_first_on_every_prompt() {
    let snippet = snippet(Target::Bash);
    assert!(snippet.contains("_claude_shim_shims="));
    assert!(snippet.contains("_claude_shim_ensure_path()"));
    assert!(snippet.contains(r#"PATH="$_claude_shim_shims${rest:+:$rest}""#));
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
fn bash_dedup_pattern_treats_the_path_as_a_literal() {
    // Inside ${rest//pat/repl} the pattern is a glob; the path must be quoted
    // there so a shims dir containing * ? [ dedups literally instead of
    // matching as a pattern (which would grow or wipe PATH).
    let snippet = snippet(Target::Bash);
    assert!(snippet.contains(r#"rest="${rest//:"$_claude_shim_shims":/:}""#));
}

#[test]
fn bash_registers_prompt_command_hook_idempotently() {
    let snippet = snippet(Target::Bash);
    // Hook installed via PROMPT_COMMAND — bash has no precmd_functions.
    assert!(snippet.contains(r#"PROMPT_COMMAND="_claude_shim_precmd"#));
    // Idempotent guard so re-sourcing does not stack the hook.
    assert!(snippet.contains("*_claude_shim_precmd*)"));
}

#[test]
fn bash_exports_profile_var() {
    let snippet = snippet(Target::Bash);
    assert!(snippet.contains("export CLAUDE_SHIM_ACTIVE_PROFILE"));
}

#[test]
fn bash_uses_no_zsh_arrayisms() {
    let snippet = snippet(Target::Bash);
    for zshism in ["precmd_functions", "typeset", "${(@)", "path=("] {
        assert!(
            !snippet.contains(zshism),
            "bash snippet must not contain zsh-ism: {zshism}"
        );
    }
}

#[test]
fn shims_dir_under_a_data_dir_is_a_single_quoted_absolute_word() {
    let word = render_shims_dir(Some(Path::new("/data")));
    assert_eq!(word, "'/data/claude-shim/shims'");
}

#[test]
fn shims_dir_single_quotes_a_data_dir_containing_a_space() {
    let word = render_shims_dir(Some(Path::new("/Application Support")));
    assert_eq!(word, "'/Application Support/claude-shim/shims'");
}

#[test]
fn shims_dir_without_a_data_dir_defers_home_expansion_to_the_shell() {
    let word = render_shims_dir(None);
    assert_eq!(word, r#""${HOME:-}"/'.local/share/claude-shim/shims'"#);
}
