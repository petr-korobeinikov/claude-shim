mod cli;
mod init;
mod profile;
pub mod shim;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command, Target};

pub fn run() -> ExitCode {
    match Cli::parse().command {
        Command::Init { target } => {
            shim::ensure_shim();
            let snippet = match target {
                Target::Zsh => init::zsh(),
                Target::OhMyPosh => init::oh_my_posh(),
            };
            print!("{snippet}");
            ExitCode::SUCCESS
        }
        Command::Current => profile::current(),
    }
}
