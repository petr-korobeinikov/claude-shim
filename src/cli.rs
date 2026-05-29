use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "claudectl", version, about = "Claude Code profile manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Print integration snippet for the given target
    Init {
        #[arg(value_enum)]
        target: Target,
    },
    /// Resolve active profile (used by the precmd hook)
    Current,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum Target {
    Zsh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_init_zsh() {
        let cli = Cli::try_parse_from(["claudectl", "init", "zsh"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Init {
                target: Target::Zsh
            }
        ));
    }

    #[test]
    fn parses_current() {
        let cli = Cli::try_parse_from(["claudectl", "current"]).unwrap();
        assert!(matches!(cli.command, Command::Current));
    }

    #[test]
    fn rejects_unknown_subcommand() {
        assert!(Cli::try_parse_from(["claudectl", "unknown"]).is_err());
    }

    #[test]
    fn rejects_unknown_init_target() {
        assert!(Cli::try_parse_from(["claudectl", "init", "fish"]).is_err());
    }
}
