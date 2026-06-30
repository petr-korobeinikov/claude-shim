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
mod tests;
