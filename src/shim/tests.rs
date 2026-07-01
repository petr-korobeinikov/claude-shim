use super::*;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

#[test]
fn shim_error_base_dirs_unavailable_renders() {
    let msg = ShimError::BaseDirsUnavailable.to_string();
    assert!(
        msg.contains("cannot resolve base directories"),
        "got: {msg}"
    );
}

#[test]
fn shim_error_cwd_unreadable_renders() {
    let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let msg = ShimError::CwdUnreadable(err).to_string();
    assert!(msg.contains("cannot read current directory"), "got: {msg}");
    assert!(msg.contains("denied"), "got: {msg}");
}

#[test]
fn shim_error_path_unset_renders() {
    let msg = ShimError::PathUnset.to_string();
    assert!(msg.contains("PATH is unset"), "got: {msg}");
}

#[test]
fn shim_error_real_claude_not_found_renders_with_dir() {
    let msg = ShimError::RealClaudeNotFound {
        self_dir: Some(PathBuf::from("/some/shim/dir")),
    }
    .to_string();
    assert!(msg.contains("not found on PATH"), "got: {msg}");
    assert!(msg.contains("/some/shim/dir"), "got: {msg}");
    assert!(msg.contains("Install Claude Code"), "got: {msg}");
}

#[test]
fn shim_error_real_claude_not_found_renders_without_dir() {
    let msg = ShimError::RealClaudeNotFound { self_dir: None }.to_string();
    assert!(msg.contains("not found on PATH"), "got: {msg}");
    assert!(msg.contains("<unknown>"), "got: {msg}");
}

#[test]
fn shim_error_no_profile_in_scope_renders_key_paths() {
    let msg = ShimError::NoProfileInScope {
        cwd: PathBuf::from("/work/proj"),
        home: PathBuf::from("/home/u"),
        default_marker: PathBuf::from("/cfg/claude-shim/default-profile"),
    }
    .to_string();
    assert!(msg.contains("no profile in scope"), "got: {msg}");
    assert!(msg.contains("/work/proj"), "got: {msg}");
    assert!(msg.contains("/home/u"), "got: {msg}");
    assert!(
        msg.contains("/cfg/claude-shim/default-profile"),
        "got: {msg}"
    );
}

#[test]
fn shim_error_profile_dir_missing_renders_name_and_paths() {
    let msg = ShimError::ProfileDirMissing {
        name: "nonexistent".to_string(),
        marker: PathBuf::from("/marker/path"),
        expected: PathBuf::from("/expected/dir"),
    }
    .to_string();
    assert!(msg.contains("'nonexistent'"), "got: {msg}");
    assert!(msg.contains("configured but missing"), "got: {msg}");
    assert!(msg.contains("/marker/path"), "got: {msg}");
    assert!(msg.contains("/expected/dir"), "got: {msg}");
}

#[test]
fn shim_error_exec_failed_renders() {
    let err = io::Error::new(io::ErrorKind::NotFound, "no such file");
    let msg = ShimError::ExecFailed {
        path: PathBuf::from("/bin/claude"),
        error: err,
    }
    .to_string();
    assert!(msg.contains("failed to exec"), "got: {msg}");
    assert!(msg.contains("/bin/claude"), "got: {msg}");
    assert!(msg.contains("no such file"), "got: {msg}");
}

fn make_executable(dir: &Path, name: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, "#!/bin/sh\n").unwrap();
    let mut perms = fs::metadata(&p).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&p, perms).unwrap();
    p
}

fn join_path(dirs: &[&Path]) -> OsString {
    std::env::join_paths(dirs.iter().map(|p| p.as_os_str())).unwrap()
}

#[test]
fn find_real_claude_returns_first_match_on_path() {
    let dir = TempDir::new().unwrap();
    let claude = make_executable(dir.path(), "claude");

    let path = join_path(&[dir.path()]);
    assert_eq!(find_real_claude(&path, None).unwrap(), claude);
}

#[test]
fn find_real_claude_skips_shim_dir() {
    let shim_dir = TempDir::new().unwrap();
    let real_dir = TempDir::new().unwrap();
    let _shim = make_executable(shim_dir.path(), "claude");
    let real = make_executable(real_dir.path(), "claude");

    let path = join_path(&[shim_dir.path(), real_dir.path()]);
    assert_eq!(
        find_real_claude(&path, Some(shim_dir.path())).unwrap(),
        real
    );
}

#[test]
fn find_real_claude_fails_when_absent() {
    let dir = TempDir::new().unwrap();
    let path = join_path(&[dir.path()]);
    match find_real_claude(&path, None).unwrap_err() {
        ShimError::RealClaudeNotFound { self_dir } => assert!(self_dir.is_none()),
        other => panic!("expected RealClaudeNotFound, got {other:?}"),
    }
}

#[test]
fn find_real_claude_skips_non_executable_file() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("claude");
    fs::write(&p, "not exec").unwrap();

    let path = join_path(&[dir.path()]);
    assert!(matches!(
        find_real_claude(&path, None),
        Err(ShimError::RealClaudeNotFound { .. })
    ));
}

#[test]
fn ensure_shim_creates_symlink_in_missing_dir() {
    let root = TempDir::new().unwrap();
    let exe = make_executable(root.path(), "claude-shim");
    let shims = root.path().join("shims");
    assert!(!shims.exists());

    ensure_shim_at(&exe, &shims).unwrap();

    let shim = shims.join("claude");
    assert_eq!(fs::read_link(&shim).unwrap(), exe);
}

#[test]
fn ensure_shim_is_idempotent() {
    let root = TempDir::new().unwrap();
    let exe = make_executable(root.path(), "claude-shim");
    let shims = root.path().join("shims");

    ensure_shim_at(&exe, &shims).unwrap();
    ensure_shim_at(&exe, &shims).unwrap();

    assert_eq!(fs::read_link(shims.join("claude")).unwrap(), exe);
}

#[test]
fn ensure_shim_replaces_stale_target() {
    let root = TempDir::new().unwrap();
    let old_exe = make_executable(root.path(), "old-claude-shim");
    let new_exe = make_executable(root.path(), "new-claude-shim");
    let shims = root.path().join("shims");

    ensure_shim_at(&old_exe, &shims).unwrap();
    ensure_shim_at(&new_exe, &shims).unwrap();

    assert_eq!(fs::read_link(shims.join("claude")).unwrap(), new_exe);
}

// ---- config_dir_for: the shim's credential-isolation decision ----
//
// `cwd` is nested under `home` so resolve()'s walk-up is bounded by its
// stop_at(home) and cannot escape to a stray marker on the real filesystem.

fn write_project_marker(cwd: &Path, name: &str) {
    let claude = cwd.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(
        claude.join("claude-shim.json"),
        crate::profile::project_body(name, None),
    )
    .unwrap();
}

fn workdir_under(home: &Path) -> std::path::PathBuf {
    let cwd = home.join("work");
    fs::create_dir_all(&cwd).unwrap();
    cwd
}

fn dirs<'a>(data: &'a TempDir, config: &'a TempDir, home: &'a TempDir) -> Dirs<'a> {
    Dirs {
        data_dir: data.path(),
        config_dir: config.path(),
        home: home.path(),
    }
}

#[test]
fn config_dir_for_exports_existing_profile_dir() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = workdir_under(home.path());
    let dir = crate::profile::profile_dir(data.path(), "foo");
    fs::create_dir_all(&dir).unwrap();
    write_project_marker(&cwd, "foo");

    let resolution = crate::profile::resolve(&cwd, home.path(), config.path());
    let got = config_dir_for(&resolution, &dirs(&data, &config, &home), &cwd);
    assert!(matches!(got, Ok(Some(d)) if d == dir));
}

#[test]
fn config_dir_for_refuses_when_configured_profile_is_missing() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = workdir_under(home.path());
    write_project_marker(&cwd, "ghost"); // marker resolves, but no profile dir exists

    let resolution = crate::profile::resolve(&cwd, home.path(), config.path());
    match config_dir_for(&resolution, &dirs(&data, &config, &home), &cwd) {
        Err(ShimError::ProfileDirMissing { name, expected, .. }) => {
            assert_eq!(name, "ghost");
            assert_eq!(expected, crate::profile::profile_dir(data.path(), "ghost"));
        }
        other => panic!("expected ProfileDirMissing, got {other:?}"),
    }
}

#[test]
fn config_dir_for_refuses_when_no_profile_in_scope() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = workdir_under(home.path()); // no marker, no ~/.claude, no default

    let resolution = crate::profile::resolve(&cwd, home.path(), config.path());
    match config_dir_for(&resolution, &dirs(&data, &config, &home), &cwd) {
        Err(ShimError::NoProfileInScope {
            default_marker,
            home: got_home,
            cwd: got_cwd,
        }) => {
            // The remediation hint must point at the real default-profile path,
            // built from config_dir — not data_dir or a wrong segment.
            assert_eq!(
                default_marker,
                config.path().join("claude-shim").join("default-profile")
            );
            assert_eq!(got_home, home.path());
            assert_eq!(got_cwd, cwd);
        }
        other => panic!("expected NoProfileInScope, got {other:?}"),
    }
}

#[test]
fn config_dir_for_allows_legacy_without_override() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = workdir_under(home.path());
    fs::create_dir_all(home.path().join(".claude")).unwrap(); // legacy ~/.claude present

    let resolution = crate::profile::resolve(&cwd, home.path(), config.path());
    assert!(matches!(
        config_dir_for(&resolution, &dirs(&data, &config, &home), &cwd),
        Ok(None)
    ));
}

#[test]
fn config_dir_for_refuses_on_malformed_marker() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = workdir_under(home.path());
    let claude = cwd.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    let marker = claude.join("claude-shim.json");
    fs::write(&marker, "{ not json").unwrap();

    let resolution = crate::profile::resolve(&cwd, home.path(), config.path());
    match config_dir_for(&resolution, &dirs(&data, &config, &home), &cwd) {
        Err(ShimError::MarkerUnusable { path, .. }) => assert_eq!(path, marker),
        other => panic!("expected MarkerUnusable, got {other:?}"),
    }
}

#[test]
fn shim_error_marker_unusable_renders() {
    let msg = ShimError::MarkerUnusable {
        path: PathBuf::from("/proj/.claude/claude-shim.json"),
        reason: "not a JSON object".to_string(),
    }
    .to_string();
    assert!(msg.contains("unusable"), "got: {msg}");
    assert!(msg.contains("/proj/.claude/claude-shim.json"), "got: {msg}");
    assert!(msg.contains("not a JSON object"), "got: {msg}");
}

// ---- effort_to_inject: a shell CLAUDE_CODE_EFFORT_LEVEL is never clobbered ----

#[test]
fn effort_to_inject_returns_token_when_shell_unset() {
    assert_eq!(effort_to_inject(Some(EffortLevel::Max), false), Some("max"));
}

#[test]
fn effort_to_inject_never_clobbers_shell_value() {
    assert_eq!(effort_to_inject(Some(EffortLevel::Max), true), None);
}

#[test]
fn effort_to_inject_none_when_no_level_resolved() {
    assert_eq!(effort_to_inject(None, false), None);
}
