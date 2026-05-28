use std::env;
use std::path::Path;
use std::process::ExitCode;

use directories::BaseDirs;

pub fn current() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return ExitCode::SUCCESS,
    };
    let base = match BaseDirs::new() {
        Some(b) => b,
        None => return ExitCode::SUCCESS,
    };

    match resolve(&cwd, base.home_dir(), base.config_dir()) {
        Resolved::Project(name) => emit(&name, base.data_dir(), Severity::Loud),
        Resolved::Default(name) => emit(&name, base.data_dir(), Severity::Silent),
        Resolved::None => ExitCode::SUCCESS,
    }
}

enum Resolved {
    Project(String),
    Default(String),
    None,
}

enum Severity {
    Loud,
    Silent,
}

fn resolve(cwd: &Path, home: &Path, config_dir: &Path) -> Resolved {
    if let Some(name) = find_profile_name_bounded(cwd, Some(home)) {
        return Resolved::Project(name);
    }
    let default_marker = config_dir.join("claudectl").join("default-profile");
    if let Some(name) = read_default_marker(&default_marker) {
        return Resolved::Default(name);
    }
    Resolved::None
}

fn read_default_marker(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let name = content.lines().next().unwrap_or("").trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn emit(name: &str, data_dir: &Path, severity: Severity) -> ExitCode {
    if !is_valid_profile_name(name) {
        return match severity {
            Severity::Loud => {
                eprintln!("claudectl: invalid profile name '{name}'");
                ExitCode::from(2)
            }
            Severity::Silent => ExitCode::SUCCESS,
        };
    }
    let profile_dir = data_dir.join("claudectl").join("profiles").join(name);
    if profile_dir.is_dir() {
        println!("{name}");
        ExitCode::SUCCESS
    } else {
        match severity {
            Severity::Loud => {
                eprintln!(
                    "claudectl: profile '{name}' is referenced but {} does not exist",
                    profile_dir.display()
                );
                ExitCode::from(2)
            }
            Severity::Silent => ExitCode::SUCCESS,
        }
    }
}

pub(crate) fn find_profile_name_bounded(start: &Path, stop_at: Option<&Path>) -> Option<String> {
    for dir in start.ancestors() {
        if matches!(stop_at, Some(s) if dir == s) {
            break;
        }
        let candidate = dir.join(".claude").join("claudectl-profile");
        if candidate.is_file()
            && let Ok(content) = std::fs::read_to_string(&candidate)
        {
            let name = content.lines().next().unwrap_or("").trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
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

    fn write_profile(dir: &Path, name: &str) {
        let claude = dir.join(".claude");
        fs::create_dir_all(&claude).unwrap();
        fs::write(claude.join("claudectl-profile"), name).unwrap();
    }

    #[test]
    fn find_profile_name_returns_none_when_absent() {
        let dir = TempDir::new().unwrap();
        assert!(find_profile_name_bounded(dir.path(), None).is_none());
    }

    #[test]
    fn find_profile_name_reads_first_line() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "myprofile\nignored");
        assert_eq!(
            find_profile_name_bounded(dir.path(), None).as_deref(),
            Some("myprofile")
        );
    }

    #[test]
    fn find_profile_name_trims_whitespace() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "  trimmed  ");
        assert_eq!(
            find_profile_name_bounded(dir.path(), None).as_deref(),
            Some("trimmed")
        );
    }

    #[test]
    fn find_profile_name_skips_empty_file() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "");
        assert!(find_profile_name_bounded(dir.path(), None).is_none());
    }

    #[test]
    fn find_profile_name_walks_up_from_nested_dir() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "outer");
        let nested = dir.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(
            find_profile_name_bounded(&nested, None).as_deref(),
            Some("outer")
        );
    }

    #[test]
    fn find_profile_name_takes_nearest_match() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "outer");
        let nested = dir.path().join("inner");
        write_profile(&nested, "nearest");
        assert_eq!(
            find_profile_name_bounded(&nested, None).as_deref(),
            Some("nearest")
        );
    }

    #[test]
    fn find_profile_name_bounded_stops_before_bound() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "outside-bound");
        let nested = dir.path().join("inner");
        fs::create_dir_all(&nested).unwrap();

        assert!(find_profile_name_bounded(&nested, Some(dir.path())).is_none());
        assert_eq!(
            find_profile_name_bounded(&nested, None).as_deref(),
            Some("outside-bound")
        );
    }

    fn write_default_marker(config: &Path, name: &str) {
        let dir = config.join("claudectl");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("default-profile"), name).unwrap();
    }

    #[test]
    fn resolve_project_marker_wins_over_default() {
        let home = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        write_profile(project.path(), "proj");
        write_default_marker(config.path(), "global");

        assert!(matches!(
            resolve(project.path(), home.path(), config.path()),
            Resolved::Project(ref n) if n == "proj"
        ));
    }

    #[test]
    fn resolve_falls_back_to_default_marker() {
        let home = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();
        write_default_marker(config.path(), "global");

        assert!(matches!(
            resolve(project.path(), home.path(), config.path()),
            Resolved::Default(ref n) if n == "global"
        ));
    }

    #[test]
    fn resolve_returns_none_when_nothing_in_scope() {
        let home = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        assert!(matches!(
            resolve(project.path(), home.path(), config.path()),
            Resolved::None
        ));
    }

    #[test]
    fn read_default_marker_returns_none_for_missing_file() {
        let dir = TempDir::new().unwrap();
        assert!(read_default_marker(&dir.path().join("absent")).is_none());
    }

    #[test]
    fn read_default_marker_trims_and_takes_first_line() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("default-profile");
        fs::write(&p, "  spaced  \nignored").unwrap();
        assert_eq!(read_default_marker(&p).as_deref(), Some("spaced"));
    }

    #[test]
    fn read_default_marker_rejects_empty() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("default-profile");
        fs::write(&p, "").unwrap();
        assert!(read_default_marker(&p).is_none());
    }
}
