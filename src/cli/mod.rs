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
        /// Pin this profile's default effort level
        #[arg(long, value_enum)]
        effort: Option<crate::profile::EffortLevel>,
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
        /// Write a workspace-wide marker (.claude-shim.json at the dir root) instead of the per-project one
        #[arg(long)]
        workspace: bool,
        /// Pin an effort level for this binding
        #[arg(long, value_enum)]
        effort: Option<crate::profile::EffortLevel>,
    },
    /// Set the effort level for a profile default or this directory's binding
    #[command(group(ArgGroup::new("effort-target").args(["profile", "local"])))]
    Effort {
        /// Effort tier to pin
        #[arg(value_enum)]
        level: crate::profile::EffortLevel,
        /// Target a named profile's default instead of the active one
        #[arg(long)]
        profile: Option<String>,
        /// Target this directory's project/workspace binding, not the profile default
        #[arg(long)]
        local: bool,
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
