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
                    effort,
                },
        } => {
            assert_eq!(name, "personal");
            assert!(!default);
            assert!(!statusline);
            assert!(effort.is_none());
        }
        _ => panic!("expected Profile::New"),
    }
}

#[test]
fn parses_profile_new_with_default_flag() {
    let cli =
        Cli::try_parse_from(["claude-shim", "profile", "new", "personal", "--default"]).unwrap();
    match cli.command {
        Command::Profile {
            action:
                ProfileAction::New {
                    name,
                    default,
                    statusline,
                    ..
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
        Cli::try_parse_from(["claude-shim", "profile", "new", "personal", "--statusline"]).unwrap();
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
            action: ProfileAction::Statusline {
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
    let cli = Cli::try_parse_from(["claude-shim", "profile", "statusline", "echo hi", "--force"])
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
            action:
                ProfileAction::Use {
                    name,
                    workspace,
                    effort,
                },
        } => {
            assert_eq!(name, "work");
            assert!(!workspace);
            assert!(effort.is_none());
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
            action: ProfileAction::Use {
                name, workspace, ..
            },
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

#[test]
fn parses_profile_new_with_effort() {
    let cli = Cli::try_parse_from([
        "claude-shim",
        "profile",
        "new",
        "personal",
        "--effort",
        "high",
    ])
    .unwrap();
    match cli.command {
        Command::Profile {
            action: ProfileAction::New { effort, .. },
        } => assert!(matches!(effort, Some(crate::profile::EffortLevel::High))),
        _ => panic!("expected Profile::New"),
    }
}

#[test]
fn parses_profile_use_with_effort() {
    let cli =
        Cli::try_parse_from(["claude-shim", "profile", "use", "work", "--effort", "max"]).unwrap();
    match cli.command {
        Command::Profile {
            action: ProfileAction::Use { effort, .. },
        } => assert!(matches!(effort, Some(crate::profile::EffortLevel::Max))),
        _ => panic!("expected Profile::Use"),
    }
}

#[test]
fn parses_profile_effort() {
    let cli = Cli::try_parse_from(["claude-shim", "profile", "effort", "xhigh"]).unwrap();
    match cli.command {
        Command::Profile {
            action:
                ProfileAction::Effort {
                    level,
                    profile,
                    local,
                },
        } => {
            assert!(matches!(level, crate::profile::EffortLevel::Xhigh));
            assert!(profile.is_none());
            assert!(!local);
        }
        _ => panic!("expected Profile::Effort"),
    }
}

#[test]
fn parses_profile_effort_with_profile() {
    let cli = Cli::try_parse_from([
        "claude-shim",
        "profile",
        "effort",
        "low",
        "--profile",
        "work",
    ])
    .unwrap();
    match cli.command {
        Command::Profile {
            action: ProfileAction::Effort { level, profile, .. },
        } => {
            assert!(matches!(level, crate::profile::EffortLevel::Low));
            assert_eq!(profile.as_deref(), Some("work"));
        }
        _ => panic!("expected Profile::Effort"),
    }
}

#[test]
fn rejects_profile_effort_without_level() {
    assert!(Cli::try_parse_from(["claude-shim", "profile", "effort"]).is_err());
}

#[test]
fn rejects_profile_effort_invalid_level() {
    assert!(Cli::try_parse_from(["claude-shim", "profile", "effort", "ultra"]).is_err());
}

#[test]
fn parses_profile_effort_local() {
    let cli = Cli::try_parse_from(["claude-shim", "profile", "effort", "high", "--local"]).unwrap();
    match cli.command {
        Command::Profile {
            action: ProfileAction::Effort { local, profile, .. },
        } => {
            assert!(local);
            assert!(profile.is_none());
        }
        _ => panic!("expected Profile::Effort"),
    }
}

#[test]
fn rejects_profile_effort_local_with_profile() {
    assert!(
        Cli::try_parse_from([
            "claude-shim",
            "profile",
            "effort",
            "high",
            "--local",
            "--profile",
            "work",
        ])
        .is_err()
    );
}
