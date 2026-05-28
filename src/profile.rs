use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use directories::BaseDirs;

pub fn current() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(p) => p,
        Err(_) => return ExitCode::SUCCESS,
    };

    let name = match find_profile_name(&cwd) {
        Some(n) => n,
        None => return ExitCode::SUCCESS,
    };

    if !is_valid_profile_name(&name) {
        eprintln!("claudectl: invalid profile name '{name}'");
        return ExitCode::from(2);
    }

    let profile_dir = match profile_path(&name) {
        Some(p) => p,
        None => {
            eprintln!("claudectl: cannot resolve data dir");
            return ExitCode::from(2);
        }
    };

    if profile_dir.is_dir() {
        println!("{name}");
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "claudectl: profile '{name}' is referenced but {} does not exist",
            profile_dir.display()
        );
        ExitCode::from(2)
    }
}

fn find_profile_name(start: &Path) -> Option<String> {
    let home = BaseDirs::new().map(|b| b.home_dir().to_path_buf());
    find_profile_name_bounded(start, home.as_deref())
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

fn profile_path(name: &str) -> Option<PathBuf> {
    let base = BaseDirs::new()?;
    Some(
        base.data_dir()
            .join("claudectl")
            .join("profiles")
            .join(name),
    )
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
        assert!(find_profile_name(dir.path()).is_none());
    }

    #[test]
    fn find_profile_name_reads_first_line() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "myprofile\nignored");
        assert_eq!(find_profile_name(dir.path()).as_deref(), Some("myprofile"));
    }

    #[test]
    fn find_profile_name_trims_whitespace() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "  trimmed  ");
        assert_eq!(find_profile_name(dir.path()).as_deref(), Some("trimmed"));
    }

    #[test]
    fn find_profile_name_skips_empty_file() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "");
        assert!(find_profile_name(dir.path()).is_none());
    }

    #[test]
    fn find_profile_name_walks_up_from_nested_dir() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "outer");
        let nested = dir.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(find_profile_name(&nested).as_deref(), Some("outer"));
    }

    #[test]
    fn find_profile_name_takes_nearest_match() {
        let dir = TempDir::new().unwrap();
        write_profile(dir.path(), "outer");
        let nested = dir.path().join("inner");
        write_profile(&nested, "nearest");
        assert_eq!(find_profile_name(&nested).as_deref(), Some("nearest"));
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
}
