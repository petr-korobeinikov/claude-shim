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
    let marker = claude.join("claude-shim.json");
    fs::write(&marker, project_body(name, None)).unwrap();
    marker
}

fn write_project_marker_with_effort(dir: &Path, name: &str, effort: &str) -> PathBuf {
    let claude = dir.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    let marker = claude.join("claude-shim.json");
    let body = serde_json::to_string_pretty(&serde_json::json!({ "name": name, "effort": effort }))
        .unwrap();
    fs::write(&marker, body).unwrap();
    marker
}

#[test]
fn find_project_marker_returns_none_when_absent() {
    let dir = TempDir::new().unwrap();
    assert!(find_project_marker(dir.path(), None).is_none());
}

#[test]
fn find_project_marker_reads_name_and_returns_path() {
    let dir = TempDir::new().unwrap();
    let marker = write_project_marker(dir.path(), "myprofile");
    let hit = find_project_marker(dir.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "myprofile");
    assert_eq!(hit.path, marker);
}

#[test]
fn find_project_marker_trims_whitespace() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "  trimmed  ");
    assert_eq!(
        find_project_marker(dir.path(), None).unwrap().unwrap().name,
        "trimmed"
    );
}

#[test]
fn find_project_marker_blank_name_is_fault() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "");
    assert!(matches!(
        find_project_marker(dir.path(), None),
        Some(Err(_))
    ));
}

#[test]
fn find_project_marker_walks_up_from_nested_dir() {
    let dir = TempDir::new().unwrap();
    let outer_marker = write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("a/b/c");
    fs::create_dir_all(&nested).unwrap();
    let hit = find_project_marker(&nested, None).unwrap().unwrap();
    assert_eq!(hit.name, "outer");
    assert_eq!(hit.path, outer_marker);
}

#[test]
fn find_project_marker_takes_nearest_match() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("inner");
    let near_marker = write_project_marker(&nested, "nearest");
    let hit = find_project_marker(&nested, None).unwrap().unwrap();
    assert_eq!(hit.name, "nearest");
    assert_eq!(hit.path, near_marker);
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
    let marker = dir.join(".claude-shim.json");
    fs::write(&marker, project_body(name, None)).unwrap();
    marker
}

#[test]
fn find_project_marker_reads_workspace_marker() {
    let dir = TempDir::new().unwrap();
    let marker = write_workspace_marker(dir.path(), "ws");
    let hit = find_project_marker(dir.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "ws");
    assert_eq!(hit.path, marker);
}

#[test]
fn find_project_marker_workspace_walks_up_from_nested_dir() {
    let dir = TempDir::new().unwrap();
    let outer = write_workspace_marker(dir.path(), "ws");
    let nested = dir.path().join("proj/sub");
    fs::create_dir_all(&nested).unwrap();
    let hit = find_project_marker(&nested, None).unwrap().unwrap();
    assert_eq!(hit.name, "ws");
    assert_eq!(hit.path, outer);
}

#[test]
fn find_project_marker_project_wins_over_workspace_at_same_dir() {
    let dir = TempDir::new().unwrap();
    let proj = write_project_marker(dir.path(), "proj");
    write_workspace_marker(dir.path(), "ws");
    let hit = find_project_marker(dir.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "proj");
    assert_eq!(hit.path, proj);
}

#[test]
fn find_project_marker_nearest_workspace_wins_over_higher_project() {
    let dir = TempDir::new().unwrap();
    write_project_marker(dir.path(), "outer");
    let nested = dir.path().join("inner");
    fs::create_dir_all(&nested).unwrap();
    let near = write_workspace_marker(&nested, "inner-ws");
    let hit = find_project_marker(&nested, None).unwrap().unwrap();
    assert_eq!(hit.name, "inner-ws");
    assert_eq!(hit.path, near);
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

// ---- #19: JSON markers, effort resolution ----

#[test]
fn find_project_marker_empty_file_is_fault() {
    let dir = TempDir::new().unwrap();
    let claude = dir.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("claude-shim.json"), "").unwrap();
    assert!(matches!(
        find_project_marker(dir.path(), None),
        Some(Err(_))
    ));
}

#[test]
fn find_project_marker_malformed_is_fault() {
    let dir = TempDir::new().unwrap();
    let claude = dir.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    let marker = claude.join("claude-shim.json");
    fs::write(&marker, "{ not json").unwrap();
    match find_project_marker(dir.path(), None) {
        Some(Err(fault)) => assert_eq!(fault.path, marker),
        _ => panic!("expected a fault"),
    }
}

#[test]
fn find_project_marker_captures_effort_override() {
    let dir = TempDir::new().unwrap();
    write_project_marker_with_effort(dir.path(), "proj", "max");
    let hit = find_project_marker(dir.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "proj");
    assert_eq!(hit.effort, Some(EffortLevel::Max));
}

#[test]
fn resolve_malformed_project_marker_is_malformed() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let claude = project.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("claude-shim.json"), "{ bad").unwrap();

    assert!(matches!(
        resolve(project.path(), home.path(), config.path()),
        Resolution::Malformed(_)
    ));
}

#[test]
fn resolve_carries_project_effort_override() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    write_project_marker_with_effort(project.path(), "proj", "high");

    match resolve(project.path(), home.path(), config.path()) {
        Resolution::Profile(p) => {
            assert_eq!(p.name, "proj");
            assert_eq!(p.effort_override, Some(EffortLevel::High));
        }
        _ => panic!("expected Project"),
    }
}

#[test]
fn active_profile_is_none_on_malformed_marker() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let claude = project.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("claude-shim.json"), "{ bad").unwrap();
    assert!(active_profile(project.path(), home.path(), config.path()).is_none());
}

#[test]
fn find_project_marker_rejects_traversal_name() {
    let dir = TempDir::new().unwrap();
    let claude = dir.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(
        claude.join("claude-shim.json"),
        serde_json::json!({ "name": "../../../etc" }).to_string(),
    )
    .unwrap();
    assert!(matches!(
        find_project_marker(dir.path(), None),
        Some(Err(_))
    ));
}

#[test]
fn resolve_rejects_traversal_name_in_project_marker() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let claude = project.path().join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(
        claude.join("claude-shim.json"),
        serde_json::json!({ "name": ".." }).to_string(),
    )
    .unwrap();
    assert!(matches!(
        resolve(project.path(), home.path(), config.path()),
        Resolution::Malformed(_)
    ));
}

#[test]
fn resolve_ignores_invalid_default_marker_name() {
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    write_default_marker(config.path(), "../evil");
    assert!(matches!(
        resolve(project.path(), home.path(), config.path()),
        Resolution::None
    ));
}

// resolve_effort: project override > profile default > unset

fn project_ref(name: &str, marker: PathBuf, effort_override: Option<EffortLevel>) -> ProfileRef {
    ProfileRef {
        name: name.to_owned(),
        source: ProfileSource::Project,
        marker,
        effort_override,
        warnings: Vec::new(),
    }
}

#[test]
fn resolve_effort_override_wins_without_reading_profile_default() {
    let data = TempDir::new().unwrap();
    // No profile-default file exists; the override alone must decide.
    let pref = project_ref("p", PathBuf::from("/marker"), Some(EffortLevel::High));
    let got = resolve_effort(data.path(), &pref);
    assert_eq!(got.level, Some(EffortLevel::High));
    assert!(got.warnings.is_empty());
}

#[test]
fn resolve_effort_reads_profile_default_when_no_override() {
    let data = TempDir::new().unwrap();
    let dir = profile_dir(data.path(), "p");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("claude-shim.json"), r#"{"effort":"max"}"#).unwrap();

    let pref = project_ref("p", PathBuf::from("/marker"), None);
    assert_eq!(
        resolve_effort(data.path(), &pref).level,
        Some(EffortLevel::Max)
    );
}

#[test]
fn resolve_effort_unset_when_no_override_and_no_default() {
    let data = TempDir::new().unwrap();
    let pref = project_ref("p", PathBuf::from("/marker"), None);
    let got = resolve_effort(data.path(), &pref);
    assert_eq!(got.level, None);
    assert!(got.warnings.is_empty());
}

#[test]
fn resolve_effort_surfaces_profile_default_warning_with_its_path() {
    let data = TempDir::new().unwrap();
    let dir = profile_dir(data.path(), "p");
    fs::create_dir_all(&dir).unwrap();
    let config = dir.join("claude-shim.json");
    fs::write(&config, r#"{"effort":"ultra"}"#).unwrap();

    let pref = project_ref("p", PathBuf::from("/marker"), None);
    let got = resolve_effort(data.path(), &pref);
    assert_eq!(got.level, None);
    assert_eq!(got.warnings.len(), 1);
    assert_eq!(got.warnings[0].0, config);
}

#[test]
fn resolve_effort_carries_project_marker_warning_paired_with_marker() {
    let data = TempDir::new().unwrap();
    // A bad project-marker effort leaves no override, so we also consult the
    // (absent) profile default — but the marker's own warning must survive,
    // paired with the marker path, not the profile-default path.
    let marker = PathBuf::from("/some/.claude/claude-shim.json");
    let pref = ProfileRef {
        name: "p".to_owned(),
        source: ProfileSource::Project,
        marker: marker.clone(),
        effort_override: None,
        warnings: vec![MarkerWarning::InvalidEffort("ultra".into())],
    };
    let got = resolve_effort(data.path(), &pref);
    assert_eq!(got.level, None);
    assert_eq!(got.warnings[0].0, marker);
    assert_eq!(
        got.warnings[0].1,
        MarkerWarning::InvalidEffort("ultra".into())
    );
}

// phase 4: writing effort via the CLI paths

#[test]
fn create_with_effort_writes_profile_default_config() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(
        data.path(),
        config.path(),
        "personal",
        false,
        false,
        Some(EffortLevel::High),
    )
    .unwrap_or_else(|_| panic!("expected Ok"));
    let path = c.effort_config.expect("effort config expected");
    assert_eq!(path, c.profile_dir.join("claude-shim.json"));
    let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["effort"], "high");
    assert!(v.get("name").is_none());
}

#[test]
fn create_without_effort_omits_config() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, false, None)
        .unwrap_or_else(|_| panic!("expected Ok"));
    assert!(c.effort_config.is_none());
    assert!(!c.profile_dir.join("claude-shim.json").exists());
}

#[test]
fn apply_with_effort_writes_name_and_effort() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    apply(
        cwd.path(),
        data.path(),
        "work",
        false,
        Some(EffortLevel::Max),
    )
    .unwrap_or_else(|_| panic!("expected Ok"));
    let hit = find_project_marker(cwd.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "work");
    assert_eq!(hit.effort, Some(EffortLevel::Max));
}

fn test_dirs<'a>(data: &'a TempDir, config: &'a TempDir, home: &'a TempDir) -> Dirs<'a> {
    Dirs {
        data_dir: data.path(),
        config_dir: config.path(),
        home: home.path(),
    }
}

#[test]
fn effort_at_sets_named_profile() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    make_profile(data.path(), "work");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, None, EffortLevel::High, Some("work"), false),
        ExitCode::SUCCESS
    );
    let config_file = profile_dir(data.path(), "work").join("claude-shim.json");
    let v: Value = serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
    assert_eq!(v["effort"], "high");
}

#[test]
fn effort_at_fails_for_missing_profile() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, None, EffortLevel::High, Some("ghost"), false),
        ExitCode::from(2)
    );
}

#[test]
fn effort_at_rejects_invalid_name() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, None, EffortLevel::High, Some("a/b"), false),
        ExitCode::from(2)
    );
}

#[test]
fn effort_at_fails_without_active_profile_or_flag() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("work");
    fs::create_dir_all(&cwd).unwrap();
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::High, None, false),
        ExitCode::from(2)
    );
}

#[test]
fn effort_at_sets_active_profile_without_flag() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    make_profile(data.path(), "work");
    let cwd = home.path().join("proj");
    fs::create_dir_all(&cwd).unwrap();
    write_project_marker(&cwd, "work");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::Max, None, false),
        ExitCode::SUCCESS
    );
    let cfg = profile_dir(data.path(), "work").join("claude-shim.json");
    let v: Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    assert_eq!(v["effort"], "max");
}

#[test]
fn effort_at_overwrites_existing_default() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    make_profile(data.path(), "work");
    let dirs = test_dirs(&data, &config, &home);
    effort_at(&dirs, None, EffortLevel::Low, Some("work"), false);
    assert_eq!(
        effort_at(&dirs, None, EffortLevel::Max, Some("work"), false),
        ExitCode::SUCCESS
    );
    let cfg = profile_dir(data.path(), "work").join("claude-shim.json");
    let v: Value = serde_json::from_str(&fs::read_to_string(&cfg).unwrap()).unwrap();
    assert_eq!(v["effort"], "max");
}

#[test]
fn apply_workspace_with_effort_round_trips() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    apply(
        cwd.path(),
        data.path(),
        "work",
        true,
        Some(EffortLevel::High),
    )
    .unwrap_or_else(|_| panic!("expected Ok"));
    let hit = find_project_marker(cwd.path(), None).unwrap().unwrap();
    assert_eq!(hit.name, "work");
    assert_eq!(hit.effort, Some(EffortLevel::High));
    assert_eq!(hit.path, cwd.path().join(".claude-shim.json"));
}

// effort_at --local: update the resolved project/workspace binding, keep its name

#[test]
fn effort_at_local_updates_project_marker() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("proj");
    fs::create_dir_all(&cwd).unwrap();
    let marker = write_project_marker(&cwd, "personal");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::High, None, true),
        ExitCode::SUCCESS
    );
    let hit = find_project_marker(&cwd, Some(home.path()))
        .unwrap()
        .unwrap();
    assert_eq!(hit.name, "personal");
    assert_eq!(hit.effort, Some(EffortLevel::High));
    assert_eq!(hit.path, marker);
}

#[test]
fn effort_at_local_updates_workspace_marker() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("ws");
    fs::create_dir_all(&cwd).unwrap();
    let marker = write_workspace_marker(&cwd, "team");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::Max, None, true),
        ExitCode::SUCCESS
    );
    let hit = find_project_marker(&cwd, Some(home.path()))
        .unwrap()
        .unwrap();
    assert_eq!(hit.name, "team");
    assert_eq!(hit.effort, Some(EffortLevel::Max));
    assert_eq!(hit.path, marker);
}

#[test]
fn effort_at_local_updates_nearest_binding_up_the_tree() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let ws = home.path().join("ws");
    let cwd = ws.join("repo/sub");
    fs::create_dir_all(&cwd).unwrap();
    let marker = write_workspace_marker(&ws, "team");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::High, None, true),
        ExitCode::SUCCESS
    );
    let v: Value = serde_json::from_str(&fs::read_to_string(&marker).unwrap()).unwrap();
    assert_eq!(v["name"], "team");
    assert_eq!(v["effort"], "high");
}

#[test]
fn effort_at_local_fails_without_binding() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("bare");
    fs::create_dir_all(&cwd).unwrap();
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::High, None, true),
        ExitCode::from(2)
    );
}

#[test]
fn effort_at_local_refuses_malformed_marker_without_clobbering() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("proj");
    let claude = cwd.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    let marker = claude.join("claude-shim.json");
    fs::write(&marker, "{ not json").unwrap();
    let before = fs::read_to_string(&marker).unwrap();
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::High, None, true),
        ExitCode::from(2)
    );
    assert_eq!(fs::read_to_string(&marker).unwrap(), before);
}

#[test]
fn effort_at_local_overwrites_existing_effort() {
    let (data, home, config) = (
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
        TempDir::new().unwrap(),
    );
    let cwd = home.path().join("proj");
    fs::create_dir_all(&cwd).unwrap();
    write_project_marker_with_effort(&cwd, "personal", "low");
    let dirs = test_dirs(&data, &config, &home);
    assert_eq!(
        effort_at(&dirs, Some(&cwd), EffortLevel::Max, None, true),
        ExitCode::SUCCESS
    );
    let hit = find_project_marker(&cwd, Some(home.path()))
        .unwrap()
        .unwrap();
    assert_eq!(hit.name, "personal");
    assert_eq!(hit.effort, Some(EffortLevel::Max));
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
    let c =
        create(data.path(), config.path(), "personal", false, false, None).unwrap_or_else(|_| {
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
    let c =
        create(data.path(), config.path(), "personal", false, false, None).unwrap_or_else(|_| {
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
    let c =
        create(data.path(), config.path(), "personal", true, false, None).unwrap_or_else(|_| {
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
        create(data.path(), config.path(), "a/b", false, false, None),
        Err(NewError::InvalidName)
    ));
}

#[test]
fn create_fails_when_profile_already_exists() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let existing = profile_dir(data.path(), "personal");
    fs::create_dir_all(&existing).unwrap();

    match create(data.path(), config.path(), "personal", false, false, None) {
        Err(NewError::AlreadyExists(p)) => assert_eq!(p, existing),
        _ => panic!("expected AlreadyExists"),
    }
}

#[test]
fn create_does_not_touch_default_when_profile_exists() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    fs::create_dir_all(profile_dir(data.path(), "personal")).unwrap();

    let _ = create(data.path(), config.path(), "personal", true, false, None);
    assert!(!config.path().join("claude-shim").exists());
}

#[test]
fn create_with_statusline_writes_settings_json() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let c = create(data.path(), config.path(), "personal", false, true, None)
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
    let c = create(data.path(), config.path(), "personal", false, false, None)
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

    let a = apply(cwd.path(), data.path(), "work", false, None).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert_eq!(
        a.marker_path,
        cwd.path().join(".claude").join("claude-shim.json")
    );
    // What `apply` writes must be what the resolver reads back.
    assert_eq!(
        find_project_marker(cwd.path(), None).unwrap().unwrap().name,
        "work"
    );
}

#[test]
fn apply_creates_dot_claude_when_missing() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");
    assert!(!cwd.path().join(".claude").exists());

    apply(cwd.path(), data.path(), "work", false, None).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert!(cwd.path().join(".claude").is_dir());
}

#[test]
fn apply_writes_workspace_marker_with_flag() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "work");

    let a = apply(cwd.path(), data.path(), "work", true, None).unwrap_or_else(|_| {
        panic!("expected Ok");
    });
    assert_eq!(a.marker_path, cwd.path().join(".claude-shim.json"));
    assert_eq!(
        find_project_marker(cwd.path(), None).unwrap().unwrap().name,
        "work"
    );
    assert!(!cwd.path().join(".claude").exists());
}

#[test]
fn apply_rejects_invalid_name() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    assert!(matches!(
        apply(cwd.path(), data.path(), "a/b", false, None),
        Err(UseError::InvalidName)
    ));
}

#[test]
fn apply_fails_when_profile_missing() {
    let cwd = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    match apply(cwd.path(), data.path(), "ghost", false, None) {
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

    match apply(cwd.path(), data.path(), "work", false, None) {
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

    match apply(cwd.path(), data.path(), "work", true, None) {
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

// ---- command wrappers (the `…_at` inner fns) ----

// `cwd` is nested under `home` so resolve()'s stop_at(home) bounds the walk-up
// to the temp tree — it can't reach a stray marker on the real filesystem, and
// the boundary is exercised rather than dead.
fn workdir(home: &Path) -> PathBuf {
    let cwd = home.join("work");
    fs::create_dir_all(&cwd).unwrap();
    cwd
}

fn dirs<'a>(data: &'a TempDir, config: &'a TempDir, home: &'a TempDir) -> Dirs<'a> {
    Dirs {
        data_dir: data.path(),
        config_dir: config.path(),
        home: home.path(),
    }
}

#[test]
fn current_at_prints_active_project_profile() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    make_profile(data.path(), "foo");
    write_project_marker(&cwd, "foo");

    let mut out = Vec::new();
    let code = current_at(&dirs(&data, &config, &home), &cwd, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(String::from_utf8(out).unwrap(), "foo\n");
}

#[test]
fn current_at_is_loud_when_project_profile_missing() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    write_project_marker(&cwd, "ghost"); // referenced, but no profile dir exists

    let mut out = Vec::new();
    let code = current_at(&dirs(&data, &config, &home), &cwd, &mut out);
    assert_eq!(code, ExitCode::from(2));
    assert!(out.is_empty(), "nothing should reach stdout on error");
}

#[test]
fn current_at_prints_profile_from_default_marker() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    make_profile(data.path(), "foo");
    write_default_marker(config.path(), "foo");

    let mut out = Vec::new();
    let code = current_at(&dirs(&data, &config, &home), &cwd, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    // The printed name distinguishes "resolved via default marker" from
    // "nothing in scope" — the latter would print nothing (next test).
    assert_eq!(String::from_utf8(out).unwrap(), "foo\n");
}

#[test]
fn current_at_prints_nothing_when_no_profile_in_scope() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());

    let mut out = Vec::new();
    let code = current_at(&dirs(&data, &config, &home), &cwd, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert!(out.is_empty(), "no profile in scope → no output");
}

#[test]
fn new_at_creates_profile() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        new_at(data.path(), config.path(), "foo", false, false, None),
        ExitCode::SUCCESS
    );
    assert!(profile_dir(data.path(), "foo").is_dir());
}

#[test]
fn new_at_rejects_invalid_name() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        new_at(data.path(), config.path(), "a/b", false, false, None),
        ExitCode::from(2)
    );
}

#[test]
fn new_at_fails_when_profile_exists() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    assert_eq!(
        new_at(data.path(), config.path(), "foo", false, false, None),
        ExitCode::from(2)
    );
}

#[test]
fn new_at_with_default_and_statusline_writes_both() {
    let data = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        new_at(data.path(), config.path(), "foo", true, true, None),
        ExitCode::SUCCESS
    );
    assert!(
        config
            .path()
            .join("claude-shim")
            .join("default-profile")
            .is_file()
    );
    assert!(
        profile_dir(data.path(), "foo")
            .join("settings.json")
            .is_file()
    );
}

#[test]
fn statusline_at_sets_preset_on_named_profile() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    assert_eq!(
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            Some("foo"),
            Some(StatusLinePreset::ProfileIndicator),
            None,
            false,
        ),
        ExitCode::SUCCESS
    );
    assert!(
        profile_dir(data.path(), "foo")
            .join("settings.json")
            .is_file()
    );
}

#[test]
fn statusline_at_rejects_preset_and_command_together() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            Some("foo"),
            Some(StatusLinePreset::ProfileIndicator),
            Some("echo hi".to_owned()),
            false,
        ),
        ExitCode::from(2)
    );
}

#[test]
fn statusline_at_rejects_neither_preset_nor_command() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            Some("foo"),
            None,
            None,
            false,
        ),
        ExitCode::from(2)
    );
}

#[test]
fn statusline_at_fails_when_profile_missing() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            Some("ghost"),
            Some(StatusLinePreset::ProfileIndicator),
            None,
            false,
        ),
        ExitCode::from(2)
    );
}

#[test]
fn statusline_at_resolves_active_profile_from_cwd() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    make_profile(data.path(), "foo");
    write_project_marker(&cwd, "foo");

    let code = statusline_at(
        &dirs(&data, &config, &home),
        Some(&cwd),
        None,
        Some(StatusLinePreset::ProfileIndicator),
        None,
        false,
    );
    assert_eq!(code, ExitCode::SUCCESS);
    // Resolution must have targeted the active profile 'foo': its settings.json
    // now exists with a statusLine. Without resolving, nothing would be written.
    let settings = profile_dir(data.path(), "foo").join("settings.json");
    assert!(
        settings.is_file(),
        "active profile's settings.json should be written"
    );
    assert!(
        fs::read_to_string(&settings)
            .unwrap()
            .contains("statusLine"),
        "settings.json should contain the statusLine key"
    );
}

#[test]
fn statusline_at_fails_when_no_active_profile() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path()); // no marker in scope → no active profile

    let code = statusline_at(
        &dirs(&data, &config, &home),
        Some(&cwd),
        None,
        Some(StatusLinePreset::ProfileIndicator),
        None,
        false,
    );
    assert_eq!(code, ExitCode::from(2));
}

#[test]
fn statusline_at_fails_when_cwd_unavailable() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    assert_eq!(
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            None,
            Some(StatusLinePreset::ProfileIndicator),
            None,
            false,
        ),
        ExitCode::from(2)
    );
}

#[test]
fn statusline_at_already_set_without_force_then_force() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let set = |force| {
        statusline_at(
            &dirs(&data, &config, &home),
            None,
            Some("foo"),
            Some(StatusLinePreset::ProfileIndicator),
            None,
            force,
        )
    };
    assert_eq!(set(false), ExitCode::SUCCESS);
    assert_eq!(set(false), ExitCode::from(2));
    assert_eq!(set(true), ExitCode::SUCCESS);
}

#[test]
fn use_profile_at_writes_project_marker() {
    let data = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    assert_eq!(
        use_profile_at(cwd.path(), data.path(), "foo", false, None),
        ExitCode::SUCCESS
    );
    assert!(
        cwd.path()
            .join(".claude")
            .join("claude-shim.json")
            .is_file()
    );
}

#[test]
fn use_profile_at_rejects_invalid_name() {
    let data = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    assert_eq!(
        use_profile_at(cwd.path(), data.path(), "a/b", false, None),
        ExitCode::from(2)
    );
}

#[test]
fn use_profile_at_fails_when_profile_missing() {
    let data = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    assert_eq!(
        use_profile_at(cwd.path(), data.path(), "ghost", false, None),
        ExitCode::from(2)
    );
}

#[test]
fn use_profile_at_fails_when_marker_exists() {
    let data = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    write_project_marker(cwd.path(), "old");
    assert_eq!(
        use_profile_at(cwd.path(), data.path(), "foo", false, None),
        ExitCode::from(2)
    );
    // The guard must not clobber the existing selection from "old" to "foo".
    assert_eq!(
        find_project_marker(cwd.path(), None).unwrap().unwrap().name,
        "old"
    );
}

#[test]
fn list_at_prints_nothing_when_empty() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();

    let mut out = Vec::new();
    let code = list_at(&dirs(&data, &config, &home), None, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert!(out.is_empty(), "no profiles → no lines");
}

#[test]
fn list_at_prints_sorted_names_with_default_and_active_tags() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    make_profile(data.path(), "foo");
    make_profile(data.path(), "bar");
    write_default_marker(config.path(), "foo");
    write_project_marker(&cwd, "bar");

    let mut out = Vec::new();
    let code = list_at(&dirs(&data, &config, &home), Some(&cwd), &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    // Sorted; `bar` is active (cwd project marker), `foo` is the global default.
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "bar (active)\nfoo (default)\n"
    );
}

#[test]
fn list_at_prints_combined_tags_for_default_and_active_profile() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    let cwd = workdir(home.path());
    make_profile(data.path(), "foo");
    make_profile(data.path(), "bar");
    write_default_marker(config.path(), "foo");
    write_project_marker(&cwd, "foo"); // foo is both the global default AND active here

    let mut out = Vec::new();
    let code = list_at(&dirs(&data, &config, &home), Some(&cwd), &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    // foo carries BOTH tags (the `tags.join(", ")` branch); bar is plain; sorted.
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "bar\nfoo (default, active)\n"
    );
}

// ---- emit: prints the profile name to stdout, or stays quiet / loud per source ----

#[test]
fn emit_prints_name_for_existing_project_profile() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let mut out = Vec::new();
    let code = emit("foo", data.path(), ProfileSource::Project, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(String::from_utf8(out).unwrap(), "foo\n");
}

#[test]
fn emit_prints_name_for_existing_default_profile() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let mut out = Vec::new();
    let code = emit("foo", data.path(), ProfileSource::Default, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert_eq!(String::from_utf8(out).unwrap(), "foo\n");
}

#[test]
fn emit_is_loud_on_missing_project_profile() {
    let data = TempDir::new().unwrap();
    let mut out = Vec::new();
    let code = emit("ghost", data.path(), ProfileSource::Project, &mut out);
    assert_eq!(code, ExitCode::from(2));
    assert!(out.is_empty());
}

#[test]
fn emit_is_silent_on_missing_default_profile() {
    let data = TempDir::new().unwrap();
    let mut out = Vec::new();
    let code = emit("ghost", data.path(), ProfileSource::Default, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert!(out.is_empty());
}

#[test]
fn emit_is_loud_on_invalid_project_name() {
    let data = TempDir::new().unwrap();
    let mut out = Vec::new();
    let code = emit("a/b", data.path(), ProfileSource::Project, &mut out);
    assert_eq!(code, ExitCode::from(2));
    assert!(out.is_empty());
}

#[test]
fn emit_is_silent_on_invalid_default_name() {
    let data = TempDir::new().unwrap();
    let mut out = Vec::new();
    let code = emit("a/b", data.path(), ProfileSource::Default, &mut out);
    assert_eq!(code, ExitCode::SUCCESS);
    assert!(out.is_empty());
}

// A genuine serialize error (not a parse error mislabeled): JSON object keys
// must be strings, so serializing a map with a tuple key fails to serialize.
fn serialize_error() -> serde_json::Error {
    serde_json::to_string(&std::collections::BTreeMap::from([((0_i32, 0_i32), 0_i32)])).unwrap_err()
}

#[test]
fn statusline_error_display_renders_alreadyset() {
    let s = StatuslineError::AlreadySet(PathBuf::from("/x/settings.json")).to_string();
    assert!(s.contains("statusLine already set"), "{s}");
    assert!(s.contains("/x/settings.json"), "{s}");
    assert!(s.contains("--force"), "{s}");
}

#[test]
fn statusline_error_display_renders_not_an_object() {
    let s = StatuslineError::NotAnObject(PathBuf::from("/x/settings.json")).to_string();
    assert!(s.contains("is not a JSON object"), "{s}");
    assert!(s.contains("/x/settings.json"), "{s}");
}

#[test]
fn statusline_error_display_renders_parse() {
    let err = serde_json::from_str::<Value>("{").unwrap_err();
    let s = StatuslineError::Parse(PathBuf::from("/x/settings.json"), err).to_string();
    assert!(s.contains("failed to parse"), "{s}");
    assert!(s.contains("/x/settings.json"), "{s}");
}

#[test]
fn statusline_error_display_renders_serialize() {
    let s = StatuslineError::Serialize(serialize_error()).to_string();
    assert!(s.contains("failed to serialize settings"), "{s}");
}

#[test]
fn statusline_error_display_renders_io() {
    let s = StatuslineError::Io(
        PathBuf::from("/x/settings.json"),
        std::io::Error::other("boom"),
    )
    .to_string();
    assert!(s.contains("I/O error"), "{s}");
    assert!(s.contains("/x/settings.json"), "{s}");
    assert!(s.contains("boom"), "{s}");
}

#[test]
fn set_statusline_treats_whitespace_only_file_as_empty() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(&path, "   \n\t").unwrap();
    assert!(set_statusline(&path, &StatusLine::Custom("echo hi".to_owned()), false).is_ok());
}

#[test]
fn set_statusline_reports_parse_error_on_malformed_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("settings.json");
    fs::write(&path, "{ not json").unwrap();
    let err = set_statusline(&path, &StatusLine::Custom("x".to_owned()), false).unwrap_err();
    assert!(matches!(err, StatuslineError::Parse(..)));
}

#[test]
fn collect_skips_dir_names_that_are_invalid_profiles() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "valid");
    let root = data.path().join("claude-shim").join("profiles");
    fs::create_dir(root.join("a\\b")).unwrap(); // backslash: legal Unix filename, invalid profile name
    let got = collect(data.path(), None, None).unwrap_or_else(|_| panic!("expected Ok"));
    let names: Vec<_> = got.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["valid"]);
}

#[test]
fn collect_errors_when_profiles_path_is_a_file() {
    let data = TempDir::new().unwrap();
    let cs = data.path().join("claude-shim");
    fs::create_dir_all(&cs).unwrap();
    fs::write(cs.join("profiles"), "not a directory").unwrap();
    assert!(matches!(
        collect(data.path(), None, None),
        Err(ListError::Io(..))
    ));
}

// ---- stdout write-error handling (the path an infallible Vec<u8> can't reach) ----

/// A sink that always fails, with a configurable error kind.
struct FailingWriter(std::io::ErrorKind);

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(self.0, "simulated write failure"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn emit_treats_broken_pipe_as_success() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let mut out = FailingWriter(std::io::ErrorKind::BrokenPipe);
    assert_eq!(
        emit("foo", data.path(), ProfileSource::Project, &mut out),
        ExitCode::SUCCESS
    );
}

#[test]
fn emit_reports_other_write_errors_as_failure() {
    let data = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let mut out = FailingWriter(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        emit("foo", data.path(), ProfileSource::Project, &mut out),
        ExitCode::from(2)
    );
}

#[test]
fn list_at_reports_write_failure() {
    let data = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    let config = TempDir::new().unwrap();
    make_profile(data.path(), "foo");
    let mut out = FailingWriter(std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        list_at(&dirs(&data, &config, &home), None, &mut out),
        ExitCode::from(2)
    );
}
