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
            let default_marker = base
                .config_dir()
                .join("claude-shim")
                .join("default-profile");
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
mod tests;
