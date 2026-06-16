use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "claude-shim", version, about = "Claude Code profile manager")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Print integration snippet for the given target
    Init {
        #[arg(value_enum)]
        target: Target,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
pub(crate) enum ProfileAction {
    /// Resolve active profile (used by the precmd hook)
    Current,
    /// Create a new profile directory
    New {
        /// Profile name
        name: String,
        /// Also set this profile as the global default
        #[arg(long)]
        default: bool,
    },
    /// Bind the current directory to a profile via a marker file
    Use {
        /// Profile name (must already exist)
        name: String,
        /// Write a workspace-wide marker (.claude-shim-profile) instead of the per-project one
        #[arg(long)]
        workspace: bool,
    },
}

#[derive(Copy, Clone, ValueEnum)]
pub(crate) enum Target {
    Zsh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_init_zsh() {
        let cli = Cli::try_parse_from(["claude-shim", "init", "zsh"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Init {
                target: Target::Zsh
            }
        ));
    }

    #[test]
    fn parses_profile_current() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "current"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Profile {
                action: ProfileAction::Current,
            }
        ));
    }

    #[test]
    fn rejects_unknown_subcommand() {
        assert!(Cli::try_parse_from(["claude-shim", "unknown"]).is_err());
    }

    #[test]
    fn rejects_unknown_init_target() {
        assert!(Cli::try_parse_from(["claude-shim", "init", "fish"]).is_err());
    }

    #[test]
    fn parses_profile_new() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "new", "personal"]).unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::New { name, default },
            } => {
                assert_eq!(name, "personal");
                assert!(!default);
            }
            _ => panic!("expected Profile::New"),
        }
    }

    #[test]
    fn parses_profile_new_with_default_flag() {
        let cli =
            Cli::try_parse_from(["claude-shim", "profile", "new", "personal", "--default"])
                .unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::New { name, default },
            } => {
                assert_eq!(name, "personal");
                assert!(default);
            }
            _ => panic!("expected Profile::New"),
        }
    }

    #[test]
    fn rejects_profile_new_without_name() {
        assert!(Cli::try_parse_from(["claude-shim", "profile", "new"]).is_err());
    }

    #[test]
    fn parses_profile_use() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "use", "work"]).unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::Use { name, workspace },
            } => {
                assert_eq!(name, "work");
                assert!(!workspace);
            }
            _ => panic!("expected Profile::Use"),
        }
    }

    #[test]
    fn parses_profile_use_with_workspace_flag() {
        let cli =
            Cli::try_parse_from(["claude-shim", "profile", "use", "work", "--workspace"]).unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::Use { name, workspace },
            } => {
                assert_eq!(name, "work");
                assert!(workspace);
            }
            _ => panic!("expected Profile::Use"),
        }
    }

    #[test]
    fn rejects_profile_use_without_name() {
        assert!(Cli::try_parse_from(["claude-shim", "profile", "use"]).is_err());
    }
}
