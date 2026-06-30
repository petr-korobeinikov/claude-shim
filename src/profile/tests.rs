use super::*;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
fn is_valid_accepts_normal_names() {
    assert!(is_valid_profile_name("foo"));
    assert!(is_valid_profile_name("client-acme"));
    assert!(is_valid_profile_name("foo_bar.123"));
}

#[test]
fn is_valid_rejects_empty() {
    assert!(!is_valid_profile_name(""));
}

#[test]
fn is_valid_rejects_dot_segments() {
    assert!(!is_valid_profile_name("."));
    assert!(!is_valid_profile_name(".."));
}

#[test]
fn is_valid_rejects_path_separators() {
    assert!(!is_valid_profile_name("a/b"));
    assert!(!is_valid_profile_name("a\\b"));
}

#[test]
fn is_valid_rejects_nul_byte() {
    assert!(!is_valid_profile_name("a\0b"));
}

fn write_project_marker(dir: &Path, name: &str) -> PathBuf {
    let claude = dir.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    let marker = claude.join("claude-shim-profile");
    fs::write(&marker, name).unwrap();
    marker
}

#[test]
fn find_project_marker_returns_none_when_absent() {
    let dir = TempDir::new().unwrap();
    assert!(find_project_marker(dir.path(), None).is_none());
}

#[test]
fn find_project_marker_reads_first_line_and_returns_path() {
    let dir = TempDir::new().unwrap();
    let marker = write_project_marker(dir.path(), "myprofile\nignored");
    let got = find_project_marker(dir.path(), None).unwrap();
    assert_eq!(got.name, "myprofile");
    assert_eq!(got.path, marker);
}

#[test]
fn find_project_marker_trims_whitespace() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "  trimmed  ");
    assert_eq!(
        find_project_marker(dir.path(), None).unwrap().name,
        "trimmed"
    );
}

#[test]
fn find_project_marker_skips_empty_file() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "");
    assert!(find_project_marker(dir.path(), None).is_none());
}

#[test]
fn find_project_marker_walks_up_from_nested_dir() {
    let dir = TempDir::new().unwrap();
    let outer_marker = write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    let got = find_project_marker(&nested, None).unwrap();
    assert_eq!(got.name, "outer");
    assert_eq!(got.path, outer_marker);
}

#[test]
fn find_project_marker_takes_nearest_match() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("inner");
    let near_marker = write_project_marker(&nested, "nearest");
    let got = find_project_marker(&nested, None).unwrap();
    assert_eq!(got.name, "nearest");
    assert_eq!(got.path, near_marker);
}

#[test]
fn find_project_marker_bounded_stops_before_bound() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "outside-bound");
    let nested = dir.path().join("inner");
    fs::create_dir_all(&nested).unwrap();

    assert!(find_project_marker(&nested, Some(dir.path())).is_none());
    assert!(find_project_marker(&nested, None).is_some());
}

fn write_workspace_marker(dir: &Path, name: &str) -> PathBuf {
    let marker = dir.join(".claude-shim-profile");
    fs::write(&marker, name).unwrap();
    marker
}

#[test]
fn find_project_marker_reads_workspace_marker() {
    let dir = TempDir::new().unwrap();
    let marker = write_workspace_marker(dir.path(), "ws");
    let got = find_project_marker(dir.path(), None).unwrap();
    assert_eq!(got.name, "ws");
    assert_eq!(got.path, marker);
}

#[test]
fn find_project_marker_workspace_walks_up_from_nested_dir() {
    let dir = TempDir::new().unwrap();
    let outer = write_workspace_marker(dir.path(), "ws");
    let nested = dir.path().join("proj/sub");
    fs::create_dir_all(&nested).unwrap();
    let got = find_project_marker(&nested, None).unwrap();
    assert_eq!(got.name, "ws");
    assert_eq!(got.path, outer);
}

#[test]
fn find_project_marker_project_wins_over_workspace_at_same_dir() {
    let dir = TempDir::new().unwrap();
    let proj = write_project_marker(dir.path(), "proj");
    write_workspace_marker(dir.path(), "ws");
    let got = find_project_marker(dir.path(), None).unwrap();
    assert_eq!(got.name, "proj");
    assert_eq!(got.path, proj);
}

#[test]
fn find_project_marker_nearest_workspace_wins_over_higher_project() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("inner");
    fs::create_dir_all(&nested).unwrap();
    let near = write_workspace_marker(&nested, "inner-ws");
    let got = find_project_marker(&nested, None).unwrap();
    assert_eq!(got.name, "inner-ws");
    assert_eq!(got.path, near);
}

#[test]
fn find_project_marker_workspace_bounded_stops_before_bound() {
    let dir = TempDir::new().unwrap();
    write_workspace_marker(dir.path(), "outside-bound");
    let nested = dir.path().join("inner");
    fs::create_dir_all(&nested).unwrap();

    assert!(find_project_marker(&nested, Some(dir.path())).is_none());
    assert!(find_project_marker(&nested, None).is_some());
}

fn write_default_marker(config: &Path, name: &str) -> PathBuf {
    let dir = config.join("claude-shim");
    fs::create_dir_all(&dir).unwrap();
    let marker = dir.join("default-profile");
    fs::write(&marker, name).unwrap();
    marker
}

#[test]
fn resolve_project_marker_wins_over_default() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let proj_marker = write_project_marker(project.path(), "proj");
    write_default_marker(config.path(), "global");

    match resolve(project.path(), home.path(), config.path()) {
        Resolution::Profile(p) => {
            assert_eq!(p.name, "proj");
            assert_eq!(p.source, ProfileSource::Project);
            assert_eq!(p.marker, proj_marker);
        }
        other => panic!(
            "expected Project, got {:?}",
            matches!(other, Resolution::Profile(_))
        ),
    }
}

#[test]
fn resolve_falls_back_to_default_marker() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let def_marker = write_default_marker(config.path(), "global");

    match resolve(project.path(), home.path(), config.path()) {
        Resolution::Profile(p) => {
            assert_eq!(p.name, "global");
            assert_eq!(p.source, ProfileSource::Default);
            assert_eq!(p.marker, def_marker);
        }
        _ => panic!("expected Default profile"),
    }
}

#[test]
fn resolve_returns_legacy_when_home_claude_exists() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    fs::create_dir_all(home.path().join(".claude")).unwrap();

    assert!(matches!(
        resolve(project.path(), home.path(), config.path()),
        Resolution::Legacy
    ));
}

#[test]
fn resolve_returns_none_when_nothing_in_scope() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    assert!(matches!(
        resolve(project.path(), home.path(), config.path()),
        Resolution::None
    ));
}

#[test]
fn active_profile_uses_default_marker() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    write_default_marker(config.path(), "personal");
    assert_eq!(
        active_profile(project.path(), home.path(), config.path()).as_deref(),
        Some("personal")
    );
}

#[test]
fn active_profile_prefers_project_marker_over_default() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    write_project_marker(project.path(), "proj");
    write_default_marker(config.path(), "global");
    assert_eq!(
        active_profile(project.path(), home.path(), config.path()).as_deref(),
        Some("proj")
    );
}

#[test]
fn active_profile_is_none_without_any_marker() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    assert!(active_profile(project.path(), home.path(), config.path()).is_none());
}

#[test]
fn read_marker_file_returns_none_for_missing_file() {
    let dir = TempDir::new().unwrap();
    assert!(read_marker_file(&dir.path().join("absent")).is_none());
}

#[test]
fn read_marker_file_trims_and_takes_first_line() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("marker");
    fs::write(&p, "  spaced  \nignored").unwrap();
    assert_eq!(read_marker_file(&p).as_deref(), Some("spaced"));
}

#[test]
fn read_marker_file_rejects_empty() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("marker");
    fs::write(&p, "").unwrap();
    assert!(read_marker_file(&p).is_none());
}

#[test]
fn create_makes_profile_directory() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, false).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert!(c.profile_dir.is_dir());
    assert_eq!(c.profile_dir, profile_dir(data.path(), "personal"));
    assert!(c.default_marker.is_none());
    assert!(!config.path().join("claude-shim").exists());
}

#[test]
fn create_seeds_claude_md_in_profile_dir() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, false).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    let claude_md = c.profile_dir.join("CLAUDE.md");
    assert!(claude_md.is_file());
    assert_eq!(fs::read_to_string(&claude_md).unwrap(), PROFILE_CLAUDE_MD);
}

#[test]
fn create_with_default_writes_default_marker() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", true, false).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    let marker = c.default_marker.expect("default marker expected");
    assert_eq!(
        marker,
        config.path().join("claude-shim").join("default-profile")
    );
    assert_eq!(fs::read_to_string(&marker).unwrap(), "personal\n");
}

#[test]
fn create_rejects_invalid_name() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert!(matches!(
        create(data.path(), config.path(), "a/b", false, false),
        Err(NewError::InvalidName)
    ));
}

#[test]
fn create_fails_when_profile_already_exists() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let existing = profile_dir(data.path(), "personal");
    fs::create_dir_all(&existing).unwrap();

    match create(data.path(), config.path(), "personal", false, false) {
        Err(NewError::AlreadyExists(p)) => assert_eq!(p, existing),
        _ => panic!("expected AlreadyExists"),
    }
}

#[test]
fn create_does_not_touch_default_when_profile_exists() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    fs::create_dir_all(profile_dir(data.path(), "personal")).unwrap();

    let _ = create(data.path(), config.path(), "personal", true, false);
    assert!(!config.path().join("claude-shim").exists());
}

#[test]
fn create_with_statusline_writes_settings_json() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, true)
        .unwrap_or_else(|_| panic!("expected Ok"));
    let settings = c.profile_dir.join("settings.json");
    assert_eq!(c.statusline_settings.as_deref(), Some(settings.as_path()));
    assert!(settings.is_file());
    let value: Value = serde_json::from_str(&fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(value["statusLine"]["type"], "command");
    assert!(
        value["statusLine"]["command"]
            .as_str()
            .unwrap()
            .contains("Current profile:")
    );
}

#[test]
fn create_without_statusline_omits_settings_json() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, false)
        .unwrap_or_else(|_| panic!("expected Ok"));
    assert!(c.statusline_settings.is_none());
    assert!(!c.profile_dir.join("settings.json").exists());
}

#[test]
fn set_statusline_creates_settings_when_absent() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    set_statusline(
        &path,
        &StatusLine::Preset(StatusLinePreset::ProfileIndicator),
        false,
    )
    .unwrap_or_else(|_| panic!("expected Ok"));
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["statusLine"]["type"], "command");
}

#[test]
fn set_statusline_merges_and_preserves_existing_keys_in_order() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(
        &path,
        "{\n  \"model\": \"opus\",\n  \"theme\": \"dark\"\n}\n",
    )
    .unwrap();
    set_statusline(
        &path,
        &StatusLine::Preset(StatusLinePreset::ProfileIndicator),
        false,
    )
    .unwrap_or_else(|_| panic!("expected Ok"));
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["model"], "opus");
    assert_eq!(value["theme"], "dark");
    assert_eq!(value["statusLine"]["type"], "command");
    let keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, vec!["model", "theme", "statusLine"]);
}

#[test]
fn set_statusline_fails_when_present_without_force() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(
        &path,
        r#"{"statusLine": {"type": "command", "command": "old"}}"#,
    )
    .unwrap();
    match set_statusline(
        &path,
        &StatusLine::Preset(StatusLinePreset::ProfileIndicator),
        false,
    ) {
        Err(StatuslineError::AlreadySet(p)) => assert_eq!(p, path),
        _ => panic!("expected AlreadySet"),
    }
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["statusLine"]["command"], "old");
}

#[test]
fn set_statusline_overwrites_with_force() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(
        &path,
        r#"{"statusLine": {"type": "command", "command": "old"}}"#,
    )
    .unwrap();
    set_statusline(&path, &StatusLine::Custom("echo \"hi\"".to_owned()), true)
        .unwrap_or_else(|_| panic!("expected Ok"));
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["statusLine"]["command"], "echo \"hi\"");
}

#[test]
fn set_statusline_rejects_non_object_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(&path, "[]").unwrap();
    assert!(matches!(
        set_statusline(
            &path,
            &StatusLine::Preset(StatusLinePreset::ProfileIndicator),
            true,
        ),
        Err(StatuslineError::NotAnObject(_))
    ));
}

#[test]
fn set_statusline_custom_command_round_trips_quotes() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    let cmd = "echo \"a\\b\" 'c'";
    set_statusline(&path, &StatusLine::Custom(cmd.to_owned()), false)
        .unwrap_or_else(|_| panic!("expected Ok"));
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["statusLine"]["command"], cmd);
}

#[test]
fn profile_indicator_command_is_the_preset_snippet() {
    assert_eq!(
        StatusLine::Preset(StatusLinePreset::ProfileIndicator).command(),
        PROFILE_INDICATOR_COMMAND
    );
    assert!(PROFILE_INDICATOR_COMMAND.contains("Current profile:"));
}

fn make_profile(data: &Path, name: &str) -> PathBuf {
    let dir = profile_dir(data, name);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn apply_writes_project_marker_by_default() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");

    let a = apply(cwd.path(), data.path(), "work", false).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert_eq!(
        a.marker_path,
        cwd.path().join(".claude").join("claude-shim-profile")
    );
    assert_eq!(fs::read_to_string(&a.marker_path).unwrap(), "work\n");
}

#[test]
fn apply_creates_dot_claude_when_missing() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    assert!(!cwd.path().join(".claude").exists());

    apply(cwd.path(), data.path(), "work", false).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert!(cwd.path().join(".claude").is_dir());
}

#[test]
fn apply_writes_workspace_marker_with_flag() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");

    let a = apply(cwd.path(), data.path(), "work", true).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert_eq!(a.marker_path, cwd.path().join(".claude-shim-profile"));
    assert_eq!(fs::read_to_string(&a.marker_path).unwrap(), "work\n");
    assert!(!cwd.path().join(".claude").exists());
}

#[test]
fn apply_rejects_invalid_name() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    assert!(matches!(
        apply(cwd.path(), data.path(), "a/b", false),
        Err(UseError::InvalidName)
    ));
}

#[test]
fn apply_fails_when_profile_missing() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    match apply(cwd.path(), data.path(), "ghost", false) {
        Err(UseError::ProfileNotFound(p)) => assert_eq!(p, profile_dir(data.path(), "ghost")),
        _ => panic!("expected ProfileNotFound"),
    }
}

#[test]
fn apply_fails_when_project_marker_already_exists() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    let existing = write_project_marker(cwd.path(), "old");

    match apply(cwd.path(), data.path(), "work", false) {
        Err(UseError::MarkerAlreadyExists(p)) => assert_eq!(p, existing),
        _ => panic!("expected MarkerAlreadyExists"),
    }
}

#[test]
fn apply_fails_when_workspace_marker_already_exists() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    let existing = write_workspace_marker(cwd.path(), "old");

    match apply(cwd.path(), data.path(), "work", true) {
        Err(UseError::MarkerAlreadyExists(p)) => assert_eq!(p, existing),
        _ => panic!("expected MarkerAlreadyExists"),
    }
}

#[test]
fn collect_returns_empty_when_root_missing() {
    let data = TempDir::new().unwrap();
    let got = collect(data.path(), None, None).unwrap_or_else(|_| panic!("expected Ok"));
    assert!(got.is_empty());
}

#[test]
fn collect_returns_sorted_dir_names_only() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "personal");
    make_profile(data.path(), "client-acme");
    make_profile(data.path(), "default");
    let root = data.path().join("claude-shim").join("profiles");
    fs::write(root.join("README"), "not a profile").unwrap();

    let got = collect(data.path(), None, None).unwrap_or_else(|_| panic!("expected Ok"));
    let names: Vec<_> = got.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["client-acme", "default", "personal"]);
    assert!(got.iter().all(|p| !p.is_default && !p.is_active));
}

#[test]
fn collect_marks_default_only() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "personal");
    make_profile(data.path(), "work");

    let got = collect(data.path(), Some("work"), None).unwrap_or_else(|_| panic!("expected Ok"));
    let work = got.iter().find(|p| p.name == "work").unwrap();
    let personal = got.iter().find(|p| p.name == "personal").unwrap();
    assert!(work.is_default && !work.is_active);
    assert!(!personal.is_default && !personal.is_active);
}

#[test]
fn collect_marks_active_only() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "personal");
    make_profile(data.path(), "work");

    let got =
        collect(data.path(), None, Some("personal")).unwrap_or_else(|_| panic!("expected Ok"));
    let personal = got.iter().find(|p| p.name == "personal").unwrap();
    let work = got.iter().find(|p| p.name == "work").unwrap();
    assert!(!personal.is_default && personal.is_active);
    assert!(!work.is_default && !work.is_active);
}

#[test]
fn collect_marks_default_and_active_on_same_profile() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "solo");

    let got =
        collect(data.path(), Some("solo"), Some("solo")).unwrap_or_else(|_| panic!("expected Ok"));
    assert_eq!(got.len(), 1);
    assert!(got[0].is_default && got[0].is_active);
}

#[test]
fn collect_ignores_marker_names_that_point_at_nothing() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "personal");

    let got = collect(data.path(), Some("missing"), Some("ghost"))
        .unwrap_or_else(|_| panic!("expected Ok"));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name, "personal");
    assert!(!got[0].is_default && !got[0].is_active);
}
