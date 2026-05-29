use std::env;
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

pub(crate) fn current() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return ExitCode::SUCCESS,
    };
    let base = match BaseDirs::new() {
        Some(b) => b,
        None => return ExitCode::SUCCESS,
    };

    match resolve(&cwd, base.home_dir(), base.config_dir()) {
        Resolution::Profile(p) => emit(&p.name, base.data_dir(), p.source),
        Resolution::Legacy | Resolution::None => ExitCode::SUCCESS,
    }
}

pub(crate) fn resolve(cwd: &Path, home: &Path, config_dir: &Path) -> Resolution {
    if let Some(m) = find_project_marker(cwd, Some(home)) {
        return Resolution::Profile(ProfileRef {
            name: m.name,
            source: ProfileSource::Project,
            marker: m.path,
        });
    }
    let default_marker = config_dir.join("claudectl").join("default-profile");
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
        let candidate = dir.join(".claude").join("claudectl-profile");
        if candidate.is_file()
            && let Some(name) = read_marker_file(&candidate)
        {
            return Some(ProjectMarker {
                name,
                path: candidate,
            });
        }
    }
    None
}

pub(crate) fn profile_dir(data_dir: &Path, name: &str) -> PathBuf {
    data_dir.join("claudectl").join("profiles").join(name)
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
            eprintln!("claudectl: invalid profile name '{name}'");
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
            "claudectl: profile '{name}' is referenced but {} does not exist",
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
        let marker = claude.join("claudectl-profile");
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

    fn write_default_marker(config: &Path, name: &str) -> PathBuf {
        let dir = config.join("claudectl");
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
}
