use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use directories::BaseDirs;

use crate::profile::find_profile_name_bounded;

pub fn run() -> ExitCode {
    let env = match Env::from_system() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    let resolution = match resolve_profile(&env) {
        Ok(r) => r,
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
    if let Resolution::Profile(dir) = &resolution {
        cmd.env("CLAUDE_CONFIG_DIR", dir);
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

#[derive(Debug, PartialEq)]
pub(crate) enum Resolution {
    Profile(PathBuf),
    Legacy,
}

pub(crate) fn resolve_profile(env: &Env) -> Result<Resolution, String> {
    if let Some(name) = find_profile_name_bounded(&env.cwd, Some(&env.home)) {
        let marker = env.cwd.join(".claude").join("claudectl-profile");
        return profile_dir(&env.data_dir, &name, &marker);
    }

    let default_marker = env.config_dir.join("claudectl").join("default-profile");
    if let Some(name) = read_marker(&default_marker) {
        return profile_dir(&env.data_dir, &name, &default_marker);
    }

    if env.home.join(".claude").is_dir() {
        return Ok(Resolution::Legacy);
    }

    Err(format!(
        "claudectl: refusing to run `claude` — no profile in scope.\n  \
         searched .claude/claudectl-profile from {} up to {}\n  \
         and {}\n\n\
         Pick a profile explicitly to avoid leaking credentials across contexts:\n  \
         echo <name> > .claude/claudectl-profile      # for this project\n  \
         echo <name> > {}    # as your default",
        env.cwd.display(),
        env.home.display(),
        default_marker.display(),
        default_marker.display(),
    ))
}

fn profile_dir(data_dir: &Path, name: &str, marker_origin: &Path) -> Result<Resolution, String> {
    let dir = data_dir.join("claudectl").join("profiles").join(name);
    if !dir.is_dir() {
        return Err(format!(
            "claudectl: refusing to run `claude` — profile '{name}' is configured but missing.\n  \
             marker:   {}\n  \
             expected: {}\n\n\
             Create the profile or fix the marker:\n  \
             mkdir -p {}",
            marker_origin.display(),
            dir.display(),
            dir.display(),
        ));
    }
    Ok(Resolution::Profile(dir))
}

fn read_marker(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let name = content.lines().next().unwrap_or("").trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
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

    fn make_env(home: &Path, data: &Path, config: &Path, cwd: &Path) -> Env {
        Env {
            home: home.to_path_buf(),
            data_dir: data.to_path_buf(),
            config_dir: config.to_path_buf(),
            cwd: cwd.to_path_buf(),
            path: OsString::new(),
            self_dir: None,
        }
    }

    fn write_profile_dir(data: &Path, name: &str) -> PathBuf {
        let dir = data.join("claudectl").join("profiles").join(name);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_project_marker(project: &Path, name: &str) {
        let claude = project.join(".claude");
        fs::create_dir_all(&claude).unwrap();
        fs::write(claude.join("claudectl-profile"), name).unwrap();
    }

    fn write_default_marker(config: &Path, name: &str) {
        let dir = config.join("claudectl");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("default-profile"), name).unwrap();
    }

    #[test]
    fn resolve_project_marker_wins_over_xdg_default() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        let expected = write_profile_dir(data.path(), "proj");
        write_project_marker(project.path(), "proj");
        write_profile_dir(data.path(), "default");
        write_default_marker(config.path(), "default");

        let env = make_env(home.path(), data.path(), config.path(), project.path());
        assert_eq!(
            resolve_profile(&env).unwrap(),
            Resolution::Profile(expected)
        );
    }

    #[test]
    fn resolve_xdg_default_when_no_project_marker() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        let expected = write_profile_dir(data.path(), "personal");
        write_default_marker(config.path(), "personal");

        let env = make_env(home.path(), data.path(), config.path(), project.path());
        assert_eq!(
            resolve_profile(&env).unwrap(),
            Resolution::Profile(expected)
        );
    }

    #[test]
    fn resolve_legacy_when_no_markers_but_home_claude_exists() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        fs::create_dir_all(home.path().join(".claude")).unwrap();

        let env = make_env(home.path(), data.path(), config.path(), project.path());
        assert_eq!(resolve_profile(&env).unwrap(), Resolution::Legacy);
    }

    #[test]
    fn resolve_fail_loud_when_nothing_in_scope() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        let env = make_env(home.path(), data.path(), config.path(), project.path());
        let err = resolve_profile(&env).unwrap_err();
        assert!(err.contains("no profile in scope"), "got: {err}");
    }

    #[test]
    fn resolve_fail_loud_when_profile_dir_missing() {
        let home = TempDir::new().unwrap();
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        write_project_marker(project.path(), "nonexistent");

        let env = make_env(home.path(), data.path(), config.path(), project.path());
        let err = resolve_profile(&env).unwrap_err();
        assert!(err.contains("'nonexistent'"), "got: {err}");
        assert!(err.contains("configured but missing"), "got: {err}");
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
        ensure_shim_at(&exe, &shims).unwrap(); // should not error

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
