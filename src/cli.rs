use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

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
        /// Also write a settings.json with a statusLine showing the active profile
        #[arg(long)]
        statusline: bool,
    },
    /// Set a profile's statusLine (a preset or a custom command)
    #[command(group(ArgGroup::new("source").required(true).args(["preset", "command"])))]
    Statusline {
        /// Profile to modify (defaults to the active profile)
        #[arg(long)]
        profile: Option<String>,
        /// Built-in preset to install
        #[arg(long, value_enum)]
        preset: Option<crate::profile::StatusLinePreset>,
        /// Custom statusLine command (mutually exclusive with --preset)
        #[arg(value_name = "COMMAND")]
        command: Option<String>,
        /// Overwrite an existing statusLine
        #[arg(long)]
        force: bool,
    },
    /// Bind the current directory to a profile via a marker file
    Use {
        /// Profile name (must already exist)
        name: String,
        /// Write a workspace-wide marker (.claude-shim-profile) instead of the per-project one
        #[arg(long)]
        workspace: bool,
    },
    /// List all profiles, marking default and the one active in the current directory
    List,
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
                action:
                    ProfileAction::New {
                        name,
                        default,
                        statusline,
                    },
            } => {
                assert_eq!(name, "personal");
                assert!(!default);
                assert!(!statusline);
            }
            _ => panic!("expected Profile::New"),
        }
    }

    #[test]
    fn parses_profile_new_with_default_flag() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "new", "personal", "--default"])
            .unwrap();
        match cli.command {
            Command::Profile {
                action:
                    ProfileAction::New {
                        name,
                        default,
                        statusline,
                    },
            } => {
                assert_eq!(name, "personal");
                assert!(default);
                assert!(!statusline);
            }
            _ => panic!("expected Profile::New"),
        }
    }

    #[test]
    fn parses_profile_new_with_statusline_flag() {
        let cli =
            Cli::try_parse_from(["claude-shim", "profile", "new", "personal", "--statusline"])
                .unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::New { statusline, .. },
            } => assert!(statusline),
            _ => panic!("expected Profile::New"),
        }
    }

    #[test]
    fn rejects_profile_new_without_name() {
        assert!(Cli::try_parse_from(["claude-shim", "profile", "new"]).is_err());
    }

    #[test]
    fn parses_profile_statusline_with_preset() {
        let cli = Cli::try_parse_from([
            "claude-shim",
            "profile",
            "statusline",
            "--preset",
            "profile-indicator",
        ])
        .unwrap();
        match cli.command {
            Command::Profile {
                action:
                    ProfileAction::Statusline {
                        profile,
                        preset,
                        command,
                        force,
                    },
            } => {
                assert!(profile.is_none());
                assert!(matches!(
                    preset,
                    Some(crate::profile::StatusLinePreset::ProfileIndicator)
                ));
                assert!(command.is_none());
                assert!(!force);
            }
            _ => panic!("expected Profile::Statusline"),
        }
    }

    #[test]
    fn parses_profile_statusline_with_custom_command() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "statusline", "echo hi"]).unwrap();
        match cli.command {
            Command::Profile {
                action:
                    ProfileAction::Statusline {
                        preset, command, ..
                    },
            } => {
                assert!(preset.is_none());
                assert_eq!(command.as_deref(), Some("echo hi"));
            }
            _ => panic!("expected Profile::Statusline"),
        }
    }

    #[test]
    fn parses_profile_statusline_with_profile_and_force() {
        let cli = Cli::try_parse_from([
            "claude-shim",
            "profile",
            "statusline",
            "--profile",
            "work",
            "--force",
            "--preset",
            "profile-indicator",
        ])
        .unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::Statusline { profile, force, .. },
            } => {
                assert_eq!(profile.as_deref(), Some("work"));
                assert!(force);
            }
            _ => panic!("expected Profile::Statusline"),
        }
    }

    #[test]
    fn parses_profile_statusline_force_after_command() {
        let cli =
            Cli::try_parse_from(["claude-shim", "profile", "statusline", "echo hi", "--force"])
                .unwrap();
        match cli.command {
            Command::Profile {
                action: ProfileAction::Statusline { command, force, .. },
            } => {
                assert_eq!(command.as_deref(), Some("echo hi"));
                assert!(force);
            }
            _ => panic!("expected Profile::Statusline"),
        }
    }

    #[test]
    fn rejects_profile_statusline_without_source() {
        assert!(Cli::try_parse_from(["claude-shim", "profile", "statusline"]).is_err());
    }

    #[test]
    fn rejects_profile_statusline_with_both_preset_and_command() {
        assert!(
            Cli::try_parse_from([
                "claude-shim",
                "profile",
                "statusline",
                "--preset",
                "profile-indicator",
                "echo hi",
            ])
            .is_err()
        );
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

    #[test]
    fn parses_profile_list() {
        let cli = Cli::try_parse_from(["claude-shim", "profile", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Profile {
                action: ProfileAction::List,
            }
        ));
    }
}
