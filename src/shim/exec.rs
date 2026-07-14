//! The shim entry point: invoked as `claude`, resolve the environment, decide
//! the profile's config dir via `super::config_dir_for` (which is unit-tested),
//! then exec the real `claude`. Plus `ensure_shim`, which installs the `claude`
//! symlink. This file is env + process-replacement glue with no decision logic
//! of its own, so it is excluded from coverage via `--ignore-filename-regex`.

use std::convert::Infallible;
use std::env;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, ExitCode};

use directories::BaseDirs;

use crate::profile;
use crate::render::PathCtx;

use super::{ShimError, config_dir_for, effort_to_inject, ensure_shim_at, find_real_claude};

#[must_use]
pub fn run() -> ExitCode {
    match try_run() {
        Ok(never) => match never {},
        Err(e) => {
            // Re-resolved only to shorten the message; this CLI never chdirs, so
            // it matches what try_run saw. An absent anchor leaves paths absolute.
            let cwd = env::current_dir().ok();
            let home = BaseDirs::new().map(|b| b.home_dir().to_path_buf());
            let ctx = PathCtx::new(cwd.as_deref(), home.as_deref());
            eprintln!("{}", e.show(ctx));
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

    let dirs = profile::Dirs {
        data_dir: base.data_dir(),
        config_dir: base.config_dir(),
        home: base.home_dir(),
    };
    let resolution = profile::resolve(&cwd, dirs.home, dirs.config_dir);
    if let Some(dir) = config_dir_for(&resolution, &dirs, &cwd)? {
        cmd.env("CLAUDE_CONFIG_DIR", &dir);
    }
    if let profile::Resolution::Profile(p) = &resolution {
        let effort = profile::resolve_effort(dirs.data_dir, p);
        let ctx = PathCtx::new(Some(cwd.as_path()), Some(dirs.home));
        for (path, warning) in &effort.warnings {
            eprintln!("claude-shim: {}: {warning}", ctx.show(path));
        }
        let shell_already_set = env::var_os("CLAUDE_CODE_EFFORT_LEVEL").is_some();
        if let Some(token) = effort_to_inject(effort.level, shell_already_set) {
            cmd.env("CLAUDE_CODE_EFFORT_LEVEL", token);
        }
    }

    let err = cmd.exec();
    Err(ShimError::ExecFailed {
        path: real_claude,
        error: err,
    })
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
        let cwd = env::current_dir().ok();
        let ctx = PathCtx::new(cwd.as_deref(), Some(base.home_dir()));
        eprintln!(
            "claude-shim: failed to ensure shim symlink at {}: {e}",
            ctx.show(&shims)
        );
    }
}
