use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use directories::BaseDirs;

use crate::profile::{self, Resolution};

pub fn run() -> ExitCode {
    let env = match Env::from_system() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let real_claude = match find_real_claude(&env) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let mut cmd = Command::new(&real_claude);
    cmd.args(&args);

    match profile::resolve(&env.cwd, &env.home, &env.config_dir) {
        Resolution::Profile(p) => {
            let dir = profile::profile_dir(&env.data_dir, &p.name);
            if !dir.is_dir() {
                eprintln!("{}", fmt_profile_dir_missing(&p.name, &p.marker, &dir));
                return ExitCode::from(2);
            }
            cmd.env("CLAUDE_CONFIG_DIR", &dir);
        }
        Resolution::Legacy => {}
        Resolution::None => {
            let default_marker = env.config_dir.join("claudectl").join("default-profile");
            eprintln!(
                "{}",
                fmt_no_profile_in_scope(&env.cwd, &env.home, &default_marker)
            );
            return ExitCode::from(2);
        }
    }

    let err = cmd.exec();
    eprintln!("claudectl: failed to exec {}: {err}", real_claude.display());
    ExitCode::from(2)
}

pub(crate) struct Env {
    pub home: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cwd: PathBuf,
    pub path: OsString,
    pub self_dir: Option<PathBuf>,
}

impl Env {
    fn from_system() -> Result<Self, String> {
        let base = BaseDirs::new()
            .ok_or_else(|| "claudectl: cannot resolve base directories".to_string())?;
        let cwd = env::current_dir()
            .map_err(|e| format!("claudectl: cannot read current directory: {e}"))?;
        let path = env::var_os("PATH").ok_or_else(|| "claudectl: PATH is unset".to_string())?;
        let self_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf));
        Ok(Self {
            home: base.home_dir().to_path_buf(),
            data_dir: base.data_dir().to_path_buf(),
            config_dir: base.config_dir().to_path_buf(),
            cwd,
            path,
            self_dir,
        })
    }
}

fn fmt_no_profile_in_scope(cwd: &Path, home: &Path, default_marker: &Path) -> String {
    format!(
        "claudectl: refusing to run `claude` — no profile in scope.\n  \
         searched .claude/claudectl-profile from {} up to {}\n  \
         and {}\n\n\
         Pick a profile explicitly to avoid leaking credentials across contexts:\n  \
         echo <name> > .claude/claudectl-profile      # for this project\n  \
         echo <name> > {}    # as your default",
        cwd.display(),
        home.display(),
        default_marker.display(),
        default_marker.display(),
    )
}

fn fmt_profile_dir_missing(name: &str, marker: &Path, expected: &Path) -> String {
    format!(
        "claudectl: refusing to run `claude` — profile '{name}' is configured but missing.\n  \
         marker:   {}\n  \
         expected: {}\n\n\
         Create the profile or fix the marker:\n  \
         mkdir -p {}",
        marker.display(),
        expected.display(),
        expected.display(),
    )
}

pub(crate) fn find_real_claude(env: &Env) -> Result<PathBuf, String> {
    let self_dir = env.self_dir.as_deref();

    for dir in env::split_paths(&env.path) {
        if Some(dir.as_path()) == self_dir {
            continue;
        }
        let candidate = dir.join("claude");
        if is_executable(&candidate) {
            return Ok(candidate);
        }
    }

    Err(format!(
        "claudectl: real `claude` not found on PATH (excluded shim dir: {}).\n\
         Install Claude Code first.",
        self_dir
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
    ))
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata()
        .map(|m| m.is_file() && (m.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}

pub(crate) fn ensure_shim() {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claudectl: cannot resolve base directories for shim");
        return;
    };
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("claudectl: cannot determine current executable for shim: {e}");
            return;
        }
    };
    let shims = base.data_dir().join("claudectl").join("shims");
    if let Err(e) = ensure_shim_at(&exe, &shims) {
        eprintln!(
            "claudectl: failed to ensure shim symlink at {}: {e}",
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
    fn fmt_no_profile_in_scope_mentions_key_paths() {
        let msg = fmt_no_profile_in_scope(
            Path::new("/work/proj"),
            Path::new("/home/u"),
            Path::new("/cfg/claudectl/default-profile"),
        );
        assert!(msg.contains("no profile in scope"), "got: {msg}");
        assert!(msg.contains("/work/proj"), "got: {msg}");
        assert!(msg.contains("/home/u"), "got: {msg}");
        assert!(
            msg.contains("/cfg/claudectl/default-profile"),
            "got: {msg}"
        );
    }

    #[test]
    fn fmt_profile_dir_missing_includes_name_and_paths() {
        let msg = fmt_profile_dir_missing(
            "nonexistent",
            Path::new("/marker/path"),
            Path::new("/expected/dir"),
        );
        assert!(msg.contains("'nonexistent'"), "got: {msg}");
        assert!(msg.contains("configured but missing"), "got: {msg}");
        assert!(msg.contains("/marker/path"), "got: {msg}");
        assert!(msg.contains("/expected/dir"), "got: {msg}");
    }

    fn make_executable(dir: &Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        fs::write(&p, "#!/bin/sh\n").unwrap();
        let mut perms = fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&p, perms).unwrap();
        p
    }

    fn env_with_path(dirs: &[&Path], self_dir: Option<&Path>) -> Env {
        let path = std::env::join_paths(dirs.iter().map(|p| p.as_os_str())).unwrap();
        Env {
            home: PathBuf::new(),
            data_dir: PathBuf::new(),
            config_dir: PathBuf::new(),
            cwd: PathBuf::new(),
            path,
            self_dir: self_dir.map(Path::to_path_buf),
        }
    }

    #[test]
    fn find_real_claude_returns_first_match_on_path() {
        let dir = TempDir::new().unwrap();
        let claude = make_executable(dir.path(), "claude");

        let env = env_with_path(&[dir.path()], None);
        assert_eq!(find_real_claude(&env).unwrap(), claude);
    }

    #[test]
    fn find_real_claude_skips_shim_dir() {
        let shim_dir = TempDir::new().unwrap();
        let real_dir = TempDir::new().unwrap();
        let _shim = make_executable(shim_dir.path(), "claude");
        let real = make_executable(real_dir.path(), "claude");

        let env = env_with_path(&[shim_dir.path(), real_dir.path()], Some(shim_dir.path()));
        assert_eq!(find_real_claude(&env).unwrap(), real);
    }

    #[test]
    fn find_real_claude_fails_when_absent() {
        let dir = TempDir::new().unwrap();
        let env = env_with_path(&[dir.path()], None);
        let err = find_real_claude(&env).unwrap_err();
        assert!(err.contains("not found on PATH"), "got: {err}");
    }

    #[test]
    fn find_real_claude_skips_non_executable_file() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("claude");
        fs::write(&p, "not exec").unwrap();

        let env = env_with_path(&[dir.path()], None);
        assert!(find_real_claude(&env).is_err());
    }

    #[test]
    fn ensure_shim_creates_symlink_in_missing_dir() {
        let root = TempDir::new().unwrap();
        let exe = make_executable(root.path(), "claudectl");
        let shims = root.path().join("shims");
        assert!(!shims.exists());

        ensure_shim_at(&exe, &shims).unwrap();

        let shim = shims.join("claude");
        assert_eq!(fs::read_link(&shim).unwrap(), exe);
    }

    #[test]
    fn ensure_shim_is_idempotent() {
        let root = TempDir::new().unwrap();
        let exe = make_executable(root.path(), "claudectl");
        let shims = root.path().join("shims");

        ensure_shim_at(&exe, &shims).unwrap();
        ensure_shim_at(&exe, &shims).unwrap();

        assert_eq!(fs::read_link(shims.join("claude")).unwrap(), exe);
    }

    #[test]
    fn ensure_shim_replaces_stale_target() {
        let root = TempDir::new().unwrap();
        let old_exe = make_executable(root.path(), "old-claudectl");
        let new_exe = make_executable(root.path(), "new-claudectl");
        let shims = root.path().join("shims");

        ensure_shim_at(&old_exe, &shims).unwrap();
        ensure_shim_at(&new_exe, &shims).unwrap();

        assert_eq!(fs::read_link(shims.join("claude")).unwrap(), new_exe);
    }
}
