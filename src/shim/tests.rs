use super::*;
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
