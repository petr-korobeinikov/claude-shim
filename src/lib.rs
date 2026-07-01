mod cli;
mod init;
mod profile;
pub mod shim;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command, ProfileAction, Target};

#[must_use]
pub fn run() -> ExitCode {
    match Cli::parse().command {
        Command::Init {
            target: Target::Zsh,
        } => {
            shim::ensure_shim();
            print!("{}", init::zsh());
            ExitCode::SUCCESS
        }
        Command::Profile {
            action: ProfileAction::Current,
        } => profile::current(),
        Command::Profile {
            action:
                ProfileAction::New {
                    name,
                    default,
                    statusline,
                    effort,
                },
        } => profile::new(&name, default, statusline, effort),
        Command::Profile {
            action:
                ProfileAction::Statusline {
                    profile,
                    preset,
                    command,
                    force,
                },
        } => profile::statusline(profile.as_deref(), preset, command, force),
        Command::Profile {
            action:
                ProfileAction::Use {
                    name,
                    workspace,
                    effort,
                },
        } => profile::use_profile(&name, workspace, effort),
        Command::Profile {
            action:
                ProfileAction::Effort {
                    level,
                    profile,
                    local,
                },
        } => profile::effort(level, profile.as_deref(), local),
        Command::Profile {
            action: ProfileAction::List,
        } => profile::list(),
    }
}
