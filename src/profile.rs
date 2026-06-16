use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use directories::BaseDirs;

pub(crate) enum Resolution {
    Profile(ProfileRef),
    Legacy,
    None,
}

pub(crate) struct ProfileRef {
    pub name: String,
    source: ProfileSource,
    pub marker: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ProfileSource {
    Project,
    Default,
}

struct ProjectMarker {
    name: String,
    path: PathBuf,
}

pub(crate) struct Created {
    pub profile_dir: PathBuf,
    pub default_marker: Option<PathBuf>,
}

pub(crate) enum NewError {
    InvalidName,
    AlreadyExists(PathBuf),
    Io(PathBuf, io::Error),
}

pub(crate) struct Applied {
    pub marker_path: PathBuf,
}

pub(crate) enum UseError {
    InvalidName,
    ProfileNotFound(PathBuf),
    MarkerAlreadyExists(PathBuf),
    Io(PathBuf, io::Error),
}

pub(crate) fn current() -> ExitCode {
    let Ok(cwd) = env::current_dir() else {
        return ExitCode::SUCCESS;
    };
    let Some(base) = BaseDirs::new() else {
        return ExitCode::SUCCESS;
    };

    match resolve(&cwd, base.home_dir(), base.config_dir()) {
        Resolution::Profile(p) => emit(&p.name, base.data_dir(), p.source),
        Resolution::Legacy | Resolution::None => ExitCode::SUCCESS,
    }
}

pub(crate) fn new(name: &str, set_default: bool) -> ExitCode {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: unable to determine base directories");
        return ExitCode::from(2);
    };
    match create(base.data_dir(), base.config_dir(), name, set_default) {
        Ok(c) => {
            println!("created profile '{name}' at {}", c.profile_dir.display());
            if let Some(marker) = c.default_marker {
                println!("set '{name}' as the global default ({})", marker.display());
            }
            ExitCode::SUCCESS
        }
        Err(NewError::InvalidName) => {
            eprintln!("claude-shim: invalid profile name '{name}'");
            ExitCode::from(2)
        }
        Err(NewError::AlreadyExists(dir)) => {
            eprintln!(
                "claude-shim: profile '{name}' already exists at {}",
                dir.display()
            );
            ExitCode::from(2)
        }
        Err(NewError::Io(path, e)) => {
            eprintln!("claude-shim: I/O error at {}: {e}", path.display());
            ExitCode::from(2)
        }
    }
}

pub(crate) fn use_profile(name: &str, workspace: bool) -> ExitCode {
    let Ok(cwd) = env::current_dir() else {
        eprintln!("claude-shim: unable to read current directory");
        return ExitCode::from(2);
    };
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: unable to determine base directories");
        return ExitCode::from(2);
    };
    match apply(&cwd, base.data_dir(), name, workspace) {
        Ok(a) => {
            println!("set profile '{name}' at {}", a.marker_path.display());
            ExitCode::SUCCESS
        }
        Err(UseError::InvalidName) => {
            eprintln!("claude-shim: invalid profile name '{name}'");
            ExitCode::from(2)
        }
        Err(UseError::ProfileNotFound(p)) => {
            eprintln!(
                "claude-shim: profile '{name}' does not exist at {}",
                p.display()
            );
            eprintln!("hint: create it first with `claude-shim profile new {name}`");
            ExitCode::from(2)
        }
        Err(UseError::MarkerAlreadyExists(p)) => {
            eprintln!(
                "claude-shim: marker already exists at {} — remove it first to switch profiles",
                p.display()
            );
            ExitCode::from(2)
        }
        Err(UseError::Io(path, e)) => {
            eprintln!("claude-shim: I/O error at {}: {e}", path.display());
            ExitCode::from(2)
        }
    }
}

pub(crate) fn apply(
    cwd: &Path,
    data_dir: &Path,
    name: &str,
    workspace: bool,
) -> Result<Applied, UseError> {
    if !is_valid_profile_name(name) {
        return Err(UseError::InvalidName);
    }
    let profile = profile_dir(data_dir, name);
    if !profile.is_dir() {
        return Err(UseError::ProfileNotFound(profile));
    }
    let marker = if workspace {
        cwd.join(".claude-shim-profile")
    } else {
        cwd.join(".claude").join("claude-shim-profile")
    };
    if marker.exists() {
        return Err(UseError::MarkerAlreadyExists(marker));
    }
    if let Some(parent) = marker.parent() {
        std::fs::create_dir_all(parent).map_err(|e| UseError::Io(parent.to_path_buf(), e))?;
    }
    std::fs::write(&marker, format!("{name}\n")).map_err(|e| UseError::Io(marker.clone(), e))?;
    Ok(Applied {
        marker_path: marker,
    })
}

pub(crate) fn create(
    data_dir: &Path,
    config_dir: &Path,
    name: &str,
    set_default: bool,
) -> Result<Created, NewError> {
    if !is_valid_profile_name(name) {
        return Err(NewError::InvalidName);
    }
    let dir = profile_dir(data_dir, name);
    if dir.exists() {
        return Err(NewError::AlreadyExists(dir));
    }
    std::fs::create_dir_all(&dir).map_err(|e| NewError::Io(dir.clone(), e))?;
    let default_marker = if set_default {
        let marker = config_dir.join("claude-shim").join("default-profile");
        if let Some(parent) = marker.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| NewError::Io(parent.to_path_buf(), e))?;
        }
        std::fs::write(&marker, format!("{name}\n"))
            .map_err(|e| NewError::Io(marker.clone(), e))?;
        Some(marker)
    } else {
        None
    };
    Ok(Created {
        profile_dir: dir,
        default_marker,
    })
}

pub(crate) fn resolve(cwd: &Path, home: &Path, config_dir: &Path) -> Resolution {
    if let Some(m) = find_project_marker(cwd, Some(home)) {
        return Resolution::Profile(ProfileRef {
            name: m.name,
            source: ProfileSource::Project,
            marker: m.path,
        });
    }
    let default_marker = config_dir.join("claude-shim").join("default-profile");
    if let Some(name) = read_marker_file(&default_marker) {
        return Resolution::Profile(ProfileRef {
            name,
            source: ProfileSource::Default,
            marker: default_marker,
        });
    }
    if home.join(".claude").is_dir() {
        return Resolution::Legacy;
    }
    Resolution::None
}

fn find_project_marker(start: &Path, stop_at: Option<&Path>) -> Option<ProjectMarker> {
    for dir in start.ancestors() {
        if matches!(stop_at, Some(s) if dir == s) {
            break;
        }
        let project = dir.join(".claude").join("claude-shim-profile");
        if project.is_file()
            && let Some(name) = read_marker_file(&project)
        {
            return Some(ProjectMarker {
                name,
                path: project,
            });
        }
        let workspace = dir.join(".claude-shim-profile");
        if workspace.is_file()
            && let Some(name) = read_marker_file(&workspace)
        {
            return Some(ProjectMarker {
                name,
                path: workspace,
            });
        }
    }
    None
}

pub(crate) fn profile_dir(data_dir: &Path, name: &str) -> PathBuf {
    data_dir.join("claude-shim").join("profiles").join(name)
}

fn read_marker_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let name = content.lines().next().unwrap_or("").trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn emit(name: &str, data_dir: &Path, source: ProfileSource) -> ExitCode {
    let loud = matches!(source, ProfileSource::Project);
    if !is_valid_profile_name(name) {
        if loud {
            eprintln!("claude-shim: invalid profile name '{name}'");
            return ExitCode::from(2);
        }
        return ExitCode::SUCCESS;
    }
    let dir = profile_dir(data_dir, name);
    if dir.is_dir() {
        println!("{name}");
        ExitCode::SUCCESS
    } else if loud {
        eprintln!(
            "claude-shim: profile '{name}' is referenced but {} does not exist",
            dir.display()
        );
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

fn is_valid_profile_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;
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
            other => panic!("expected Project, got {:?}", matches!(other, Resolution::Profile(_))),
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
        let c = create(data.path(), config.path(), "personal", false).unwrap_or_else(|_| {
            panic!("expected Ok");
        });
        assert!(c.profile_dir.is_dir());
        assert_eq!(c.profile_dir, profile_dir(data.path(), "personal"));
        assert!(c.default_marker.is_none());
        assert!(!config.path().join("claude-shim").exists());
    }

    #[test]
    fn create_with_default_writes_default_marker() {
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let c = create(data.path(), config.path(), "personal", true).unwrap_or_else(|_| {
            panic!("expected Ok");
        });
        let marker = c.default_marker.expect("default marker expected");
        assert_eq!(marker, config.path().join("claude-shim").join("default-profile"));
        assert_eq!(fs::read_to_string(&marker).unwrap(), "personal\n");
    }

    #[test]
    fn create_rejects_invalid_name() {
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        assert!(matches!(
            create(data.path(), config.path(), "a/b", false),
            Err(NewError::InvalidName)
        ));
    }

    #[test]
    fn create_fails_when_profile_already_exists() {
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let existing = profile_dir(data.path(), "personal");
        fs::create_dir_all(&existing).unwrap();

        match create(data.path(), config.path(), "personal", false) {
            Err(NewError::AlreadyExists(p)) => assert_eq!(p, existing),
            _ => panic!("expected AlreadyExists"),
        }
    }

    #[test]
    fn create_does_not_touch_default_when_profile_exists() {
        let data = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        fs::create_dir_all(profile_dir(data.path(), "personal")).unwrap();

        let _ = create(data.path(), config.path(), "personal", true);
        assert!(!config.path().join("claude-shim").exists());
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
        assert_eq!(a.marker_path, cwd.path().join(".claude").join("claude-shim-profile"));
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
}
