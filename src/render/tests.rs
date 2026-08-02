use super::*;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Render `path` against optional string anchors — the concise form used by the
/// table-style cases below.
fn show(path: &str, cwd: Option<&str>, home: Option<&str>) -> String {
    PathCtx::new(cwd.map(Path::new), home.map(Path::new))
        .show(Path::new(path))
        .to_string()
}

/// The `show_shell` analogue of `show` above — renders the shell-safe copy-paste
/// form.
fn show_shell(path: &str, cwd: Option<&str>, home: Option<&str>) -> String {
    PathCtx::new(cwd.map(Path::new), home.map(Path::new))
        .show_shell(Path::new(path))
        .to_string()
}

#[test]
fn descendant_of_cwd_renders_relative() {
    assert_eq!(
        show("/w/proj/src/main.rs", Some("/w/proj"), Some("/home/u")),
        "src/main.rs"
    );
}

#[test]
fn descendant_of_cwd_has_no_dot_slash_prefix() {
    assert_eq!(show("/w/proj/a", Some("/w/proj"), None), "a");
}

#[test]
fn descendant_of_home_renders_tilde() {
    assert_eq!(
        show("/home/u/.config/x", Some("/w/proj"), Some("/home/u")),
        "~/.config/x"
    );
}

#[test]
fn outside_cwd_and_home_stays_absolute() {
    assert_eq!(
        show("/etc/hosts", Some("/w/proj"), Some("/home/u")),
        "/etc/hosts"
    );
}

#[test]
fn path_equal_to_home_renders_bare_tilde() {
    assert_eq!(show("/home/u", Some("/w/proj"), Some("/home/u")), "~");
}

#[test]
fn path_equal_to_cwd_is_not_a_descendant() {
    assert_eq!(
        show("/home/u/proj", Some("/home/u/proj"), Some("/home/u")),
        "~/proj"
    );
}

#[test]
fn cwd_takes_precedence_over_home() {
    assert_eq!(
        show("/home/u/proj/f", Some("/home/u/proj"), Some("/home/u")),
        "f"
    );
}

#[test]
fn home_prefix_is_component_wise_not_textual() {
    assert_eq!(show("/homework/x", None, Some("/home")), "/homework/x");
}

#[test]
fn cwd_prefix_is_component_wise_not_textual() {
    assert_eq!(
        show("/w/projected/x", Some("/w/proj"), None),
        "/w/projected/x"
    );
}

#[test]
fn empty_context_is_always_absolute() {
    assert_eq!(
        PathCtx::EMPTY.show(Path::new("/home/u/x")).to_string(),
        "/home/u/x"
    );
}

#[test]
fn absent_cwd_still_applies_the_home_rule() {
    assert_eq!(show("/home/u/x", None, Some("/home/u")), "~/x");
}

#[test]
fn non_utf8_component_renders_lossily_under_home() {
    use std::os::unix::ffi::OsStrExt;
    let mut p = PathBuf::from("/home/u");
    p.push(OsStr::from_bytes(b"bad\xffname"));
    let out = PathCtx::new(None, Some(Path::new("/home/u")))
        .show(&p)
        .to_string();
    assert!(out.starts_with("~/"), "tilde form preserved: {out}");
    assert!(out.contains('\u{fffd}'), "invalid byte is lossy: {out}");
}

#[test]
fn non_utf8_component_renders_lossily_when_absolute() {
    use std::os::unix::ffi::OsStrExt;
    let p = PathBuf::from(OsStr::from_bytes(b"/etc/bad\xffname"));
    let out = PathCtx::EMPTY.show(&p).to_string();
    assert!(out.contains('\u{fffd}'), "invalid byte is lossy: {out}");
}

#[test]
fn shell_quotes_a_space_under_home_but_keeps_tilde_expandable() {
    assert_eq!(
        show_shell(
            "/home/u/Library/Application Support/claude-shim/default-profile",
            None,
            Some("/home/u"),
        ),
        "~/'Library/Application Support/claude-shim/default-profile'"
    );
}

#[test]
fn shell_quotes_a_space_in_an_absolute_path() {
    assert_eq!(
        show_shell("/opt/App Support/x", None, None),
        "'/opt/App Support/x'"
    );
}

#[test]
fn shell_ignores_cwd_and_stays_absolute() {
    assert_eq!(
        show_shell("/w/proj/a b", Some("/w/proj"), None),
        "'/w/proj/a b'"
    );
}

#[test]
fn shell_ignores_cwd_and_prefers_the_home_anchor() {
    assert_eq!(
        show_shell("/home/u/Library/x y", Some("/home/u"), Some("/home/u")),
        "~/'Library/x y'"
    );
}

#[test]
fn shell_quotes_a_home_path_without_spaces() {
    assert_eq!(
        show_shell(
            "/home/u/.config/claude-shim/default-profile",
            None,
            Some("/home/u"),
        ),
        "~/'.config/claude-shim/default-profile'"
    );
}

#[test]
fn shell_form_of_bare_home_is_unquoted_tilde() {
    assert_eq!(show_shell("/home/u", None, Some("/home/u")), "~");
}

#[test]
fn shell_escapes_an_embedded_single_quote() {
    assert_eq!(show_shell("/o/it's here", None, None), "'/o/it'\\''s here'");
}

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
