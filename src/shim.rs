use std::convert::Infallible;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use directories::BaseDirs;

use crate::profile::{self, Resolution};

#[must_use]
pub fn run() -> ExitCode {
    match try_run() {
        Ok(never) => match never {},
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn try_run() -> Result<Infallible, ShimError> {
    let base = BaseDirs::new().ok_or(ShimError::BaseDirsUnavailable)?;
    let cwd = env::current_dir().map_err(ShimError::CwdUnreadable)?;
    let path = env::var_os("PATH").ok_or(ShimError::PathUnset)?;
    let self_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));

    let real_claude = find_real_claude(&path, self_dir.as_deref())?;
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let mut cmd = Command::new(&real_claude);
    cmd.args(&args);

    match profile::resolve(&cwd, base.home_dir(), base.config_dir()) {
        Resolution::Profile(p) => {
            let dir = profile::profile_dir(base.data_dir(), &p.name);
            if !dir.is_dir() {
                return Err(ShimError::ProfileDirMissing {
                    name: p.name,
                    marker: p.marker,
                    expected: dir,
                });
            }
            cmd.env("CLAUDE_CONFIG_DIR", &dir);
        }
        Resolution::Legacy => {}
        Resolution::None => {
            let default_marker = base.config_dir().join("claude-shim").join("default-profile");
            return Err(ShimError::NoProfileInScope {
                cwd,
                home: base.home_dir().to_path_buf(),
                default_marker,
            });
        }
    }

    let err = cmd.exec();
    Err(ShimError::ExecFailed {
        path: real_claude,
        error: err,
    })
}

#[derive(Debug)]
enum ShimError {
    BaseDirsUnavailable,
    CwdUnreadable(io::Error),
    PathUnset,
    RealClaudeNotFound {
        self_dir: Option<PathBuf>,
    },
    NoProfileInScope {
        cwd: PathBuf,
        home: PathBuf,
        default_marker: PathBuf,
    },
    ProfileDirMissing {
        name: String,
        marker: PathBuf,
        expected: PathBuf,
    },
    ExecFailed {
        path: PathBuf,
        error: io::Error,
    },
}

impl fmt::Display for ShimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseDirsUnavailable => {
                write!(f, "claude-shim: cannot resolve base directories")
            }
            Self::CwdUnreadable(e) => {
                write!(f, "claude-shim: cannot read current directory: {e}")
            }
            Self::PathUnset => write!(f, "claude-shim: PATH is unset"),
            Self::RealClaudeNotFound { self_dir } => write!(
                f,
                "claude-shim: real `claude` not found on PATH (excluded shim dir: {}).\n\
                 Install Claude Code first.",
                self_dir
                    .as_deref()
                    .map_or_else(|| "<unknown>".to_string(), |p| p.display().to_string()),
            ),
            Self::NoProfileInScope {
                cwd,
                home,
                default_marker,
            } => write!(
                f,
                "claude-shim: refusing to run `claude` — no profile in scope.\n  \
                 searched .claude/claude-shim-profile from {} up to {}\n  \
                 and {}\n\n\
                 Pick a profile explicitly to avoid leaking credentials across contexts:\n  \
                 echo <name> > .claude/claude-shim-profile      # for this project\n  \
                 echo <name> > {}    # as your default",
                cwd.display(),
                home.display(),
                default_marker.display(),
                default_marker.display(),
            ),
            Self::ProfileDirMissing {
                name,
                marker,
                expected,
            } => write!(
                f,
                "claude-shim: refusing to run `claude` — profile '{name}' is configured but missing.\n  \
                 marker:   {}\n  \
                 expected: {}\n\n\
                 Create the profile or fix the marker:\n  \
                 mkdir -p {}",
                marker.display(),
                expected.display(),
                expected.display(),
            ),
            Self::ExecFailed { path, error } => {
                write!(f, "claude-shim: failed to exec {}: {error}", path.display())
            }
        }
    }
}

fn find_real_claude(path: &OsStr, self_dir: Option<&Path>) -> Result<PathBuf, ShimError> {
    for dir in env::split_paths(path) {
        if Some(dir.as_path()) == self_dir {
            continue;
        }
        let candidate = dir.join("claude");
        if is_executable(&candidate) {
            return Ok(candidate);
        }
    }
    Err(ShimError::RealClaudeNotFound {
        self_dir: self_dir.map(Path::to_path_buf),
    })
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111) != 0)
}

pub(crate) fn ensure_shim() {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: cannot resolve base directories for shim");
        return;
    };
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("claude-shim: cannot determine current executable for shim: {e}");
            return;
        }
    };
    let shims = base.data_dir().join("claude-shim").join("shims");
    if let Err(e) = ensure_shim_at(&exe, &shims) {
        eprintln!(
            "claude-shim: failed to ensure shim symlink at {}: {e}",
            shims.display()
        );
    }
}

fn ensure_shim_at(exe: &Path, shims_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(shims_dir)?;
    let shim = shims_dir.join("claude");
    if let Ok(existing) = fs::read_link(&shim)
        && existing == exe
    {
        return Ok(());
    }
    let _ = fs::remove_file(&shim);
    std::os::unix::fs::symlink(exe, &shim)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn shim_error_base_dirs_unavailable_renders() {
        let msg = ShimError::BaseDirsUnavailable.to_string();
        assert!(msg.contains("cannot resolve base directories"), "got: {msg}");
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
        assert_eq!(find_real_claude(&path, Some(shim_dir.path())).unwrap(), real);
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
}
