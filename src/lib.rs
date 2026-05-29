mod cli;
mod init;
mod profile;
pub mod shim;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command, Target};

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
        Command::Current => profile::current(),
    }
}
