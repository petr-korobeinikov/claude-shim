use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::profile::{self, Dirs, EffortLevel, Resolution};

mod exec;
pub(crate) use exec::ensure_shim;
pub use exec::run;

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
    MarkerUnusable {
        path: PathBuf,
        reason: String,
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
                 searched .claude/claude-shim.json from {} up to {}\n  \
                 and {}\n\n\
                 Pick a profile explicitly to avoid leaking credentials across contexts:\n  \
                 claude-shim profile use <name>     # for this project\n  \
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
            Self::MarkerUnusable { path, reason } => write!(
                f,
                "claude-shim: refusing to run `claude` — the profile marker at {} is unusable ({reason}).\n\n\
                 Fix or recreate it:\n  \
                 claude-shim profile use <name>",
                path.display(),
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

/// Decide the `CLAUDE_CONFIG_DIR` for a resolved profile, or refuse to run.
/// `Ok(Some(dir))` → export it; `Ok(None)` → legacy layout, no override;
/// `Err` → refuse (no profile in scope, or the configured profile is missing).
/// This is the shim's credential-isolation decision — kept here and unit-tested,
/// not inline in the env/exec glue of `exec.rs`.
fn config_dir_for(
    resolution: &Resolution,
    dirs: &Dirs,
    cwd: &Path,
) -> Result<Option<PathBuf>, ShimError> {
    match resolution {
        Resolution::Profile(p) => {
            let dir = profile::profile_dir(dirs.data_dir, &p.name);
            if !dir.is_dir() {
                return Err(ShimError::ProfileDirMissing {
                    name: p.name.clone(),
                    marker: p.marker.clone(),
                    expected: dir,
                });
            }
            Ok(Some(dir))
        }
        Resolution::Legacy => Ok(None),
        Resolution::None => Err(ShimError::NoProfileInScope {
            cwd: cwd.to_path_buf(),
            home: dirs.home.to_path_buf(),
            default_marker: dirs.config_dir.join("claude-shim").join("default-profile"),
        }),
        Resolution::Malformed(fault) => Err(ShimError::MarkerUnusable {
            path: fault.path.clone(),
            reason: fault.reason.clone(),
        }),
    }
}

fn effort_to_inject(level: Option<EffortLevel>, shell_already_set: bool) -> Option<&'static str> {
    if shell_already_set {
        return None;
    }
    level.map(EffortLevel::as_token)
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
