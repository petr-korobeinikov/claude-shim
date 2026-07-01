//! Command entry points: resolve `BaseDirs`/cwd from the environment, then
//! delegate to the matching path-explicit `*_at` inner fn in the parent module (where the logic
//! lives and is unit-tested). This file is thin env glue with no logic of its
//! own, so it is excluded from coverage via `--ignore-filename-regex`.

use std::env;
use std::io;
use std::process::ExitCode;

use directories::BaseDirs;

use super::{
    Dirs, EffortLevel, StatusLinePreset, current_at, effort_at, list_at, new_at, statusline_at,
    use_profile_at,
};

/// Resolve base directories, printing the standard error on failure.
/// `None` → the caller should exit 2.
fn base_dirs() -> Option<BaseDirs> {
    let base = BaseDirs::new();
    if base.is_none() {
        eprintln!("claude-shim: unable to determine base directories");
    }
    base
}

fn dirs(base: &BaseDirs) -> Dirs<'_> {
    Dirs {
        data_dir: base.data_dir(),
        config_dir: base.config_dir(),
        home: base.home_dir(),
    }
}

pub(crate) fn current() -> ExitCode {
    // The prompt/precmd path: stay silent (no eprintln, exit 0) on a missing
    // environment so a broken setup never spams every shell prompt — hence the
    // raw BaseDirs::new() here rather than the loud base_dirs() helper.
    let Ok(cwd) = env::current_dir() else {
        return ExitCode::SUCCESS;
    };
    let Some(base) = BaseDirs::new() else {
        return ExitCode::SUCCESS;
    };
    current_at(&dirs(&base), &cwd, &mut io::stdout())
}

pub(crate) fn new(
    name: &str,
    set_default: bool,
    statusline: bool,
    effort: Option<EffortLevel>,
) -> ExitCode {
    let Some(base) = base_dirs() else {
        return ExitCode::from(2);
    };
    new_at(
        base.data_dir(),
        base.config_dir(),
        name,
        set_default,
        statusline,
        effort,
    )
}

pub(crate) fn statusline(
    profile: Option<&str>,
    preset: Option<StatusLinePreset>,
    command: Option<String>,
    force: bool,
) -> ExitCode {
    let Some(base) = base_dirs() else {
        return ExitCode::from(2);
    };
    let cwd = env::current_dir().ok();
    statusline_at(
        &dirs(&base),
        cwd.as_deref(),
        profile,
        preset,
        command,
        force,
    )
}

pub(crate) fn use_profile(name: &str, workspace: bool, effort: Option<EffortLevel>) -> ExitCode {
    let Ok(cwd) = env::current_dir() else {
        eprintln!("claude-shim: unable to read current directory");
        return ExitCode::from(2);
    };
    let Some(base) = base_dirs() else {
        return ExitCode::from(2);
    };
    use_profile_at(&cwd, base.data_dir(), name, workspace, effort)
}

pub(crate) fn list() -> ExitCode {
    let Some(base) = base_dirs() else {
        return ExitCode::from(2);
    };
    let cwd = env::current_dir().ok();
    list_at(&dirs(&base), cwd.as_deref(), &mut io::stdout())
}

pub(crate) fn effort(level: EffortLevel, profile: Option<&str>, local: bool) -> ExitCode {
    let Some(base) = base_dirs() else {
        return ExitCode::from(2);
    };
    let cwd = env::current_dir().ok();
    effort_at(&dirs(&base), cwd.as_deref(), level, profile, local)
}
