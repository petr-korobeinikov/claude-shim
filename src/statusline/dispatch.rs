use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use directories::BaseDirs;

use super::{LinePreset, SessionFacts, render, session_profile};
use crate::profile::active_profile;

// Always exit 0, a failed write included: Claude Code blanks the line on a
// non-zero exit.
pub(crate) fn print_line(preset: LinePreset) -> ExitCode {
    let stdin = io::read_to_string(io::stdin()).unwrap_or_default();
    let no_color = env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
    let facts = SessionFacts::from_stdin(&stdin, profile());
    let _ = writeln!(
        io::stdout(),
        "{}",
        render(preset.effective(no_color), &facts)
    );
    ExitCode::SUCCESS
}

fn profile() -> Option<String> {
    let base_dirs = BaseDirs::new()?;
    let config_dir_env = env::var_os("CLAUDE_CONFIG_DIR");
    session_profile(
        config_dir_env.as_deref().map(Path::new),
        base_dirs.data_dir(),
    )
    .or_else(|| {
        let cwd = env::current_dir().ok()?;
        active_profile(&cwd, base_dirs.home_dir(), base_dirs.config_dir())
    })
}
