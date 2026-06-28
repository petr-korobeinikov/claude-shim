use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use directories::BaseDirs;
use serde_json::{Map, Value, json};

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
    pub statusline_settings: Option<PathBuf>,
}

pub(crate) enum NewError {
    InvalidName,
    AlreadyExists(PathBuf),
    Io(PathBuf, io::Error),
    Statusline(StatuslineError),
}

/// A statusLine to install into a profile's `settings.json`.
enum StatusLine {
    Preset(StatusLinePreset),
    Custom(String),
}

/// Built-in statusLine presets selectable via `--preset`.
#[derive(Copy, Clone, clap::ValueEnum)]
pub(crate) enum StatusLinePreset {
    /// Render `Current profile: <name>`, resolving the active profile.
    ProfileIndicator,
}

pub(crate) enum StatuslineError {
    AlreadySet(PathBuf),
    NotAnObject(PathBuf),
    Parse(PathBuf, serde_json::Error),
    Serialize(serde_json::Error),
    Io(PathBuf, io::Error),
}

// The shell command behind `StatusLinePreset::ProfileIndicator`. It prints
// `Current profile: <name>`, resolving the active profile via the shim and
// falling back to the CLAUDE_CONFIG_DIR basename when the binary or a marker
// is unavailable at render time. This is a shell snippet (a value), not JSON —
// the surrounding settings.json is built with serde_json, never hand-written.
const PROFILE_INDICATOR_COMMAND: &str = "p=$(claude-shim profile current 2>/dev/null); [ -n \"$p\" ] || p=\"${CLAUDE_CONFIG_DIR##*/}\"; echo \"Current profile: $p\"";

impl StatusLinePreset {
    fn command(self) -> &'static str {
        match self {
            StatusLinePreset::ProfileIndicator => PROFILE_INDICATOR_COMMAND,
        }
    }
}

impl StatusLine {
    fn command(&self) -> &str {
        match self {
            StatusLine::Preset(preset) => preset.command(),
            StatusLine::Custom(command) => command,
        }
    }
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

pub(crate) struct ListedProfile {
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
}

pub(crate) enum ListError {
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

pub(crate) fn new(name: &str, set_default: bool, statusline: bool) -> ExitCode {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: unable to determine base directories");
        return ExitCode::from(2);
    };
    match create(
        base.data_dir(),
        base.config_dir(),
        name,
        set_default,
        statusline,
    ) {
        Ok(c) => {
            println!("created profile '{name}' at {}", c.profile_dir.display());
            if let Some(path) = c.statusline_settings {
                println!("enabled statusLine indicator at {}", path.display());
            }
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
        Err(NewError::Statusline(e)) => {
            report_statusline_error(&e);
            ExitCode::from(2)
        }
    }
}

pub(crate) fn statusline(
    profile: Option<&str>,
    preset: Option<StatusLinePreset>,
    command: Option<String>,
    force: bool,
) -> ExitCode {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: unable to determine base directories");
        return ExitCode::from(2);
    };
    let requested = match (preset, command) {
        (Some(preset), None) => StatusLine::Preset(preset),
        (None, Some(command)) => StatusLine::Custom(command),
        (None, None) => {
            eprintln!("claude-shim: pass --preset <preset> or a custom command");
            return ExitCode::from(2);
        }
        (Some(_), Some(_)) => {
            eprintln!("claude-shim: --preset and a custom command are mutually exclusive");
            return ExitCode::from(2);
        }
    };
    let name = if let Some(name) = profile {
        name.to_owned()
    } else {
        let Ok(cwd) = env::current_dir() else {
            eprintln!("claude-shim: unable to read current directory");
            return ExitCode::from(2);
        };
        let Some(name) = active_profile(&cwd, base.home_dir(), base.config_dir()) else {
            eprintln!("claude-shim: no active profile here — pass --profile <name>");
            return ExitCode::from(2);
        };
        name
    };
    if !is_valid_profile_name(&name) {
        eprintln!("claude-shim: invalid profile name '{name}'");
        return ExitCode::from(2);
    }
    let dir = profile_dir(base.data_dir(), &name);
    if !dir.is_dir() {
        eprintln!(
            "claude-shim: profile '{name}' does not exist at {}",
            dir.display()
        );
        return ExitCode::from(2);
    }
    let settings_path = dir.join("settings.json");
    match set_statusline(&settings_path, &requested, force) {
        Ok(()) => {
            println!(
                "set statusLine on profile '{name}' ({})",
                settings_path.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            report_statusline_error(&e);
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

pub(crate) fn list() -> ExitCode {
    let Some(base) = BaseDirs::new() else {
        eprintln!("claude-shim: unable to determine base directories");
        return ExitCode::from(2);
    };
    let default_name = read_marker_file(
        &base
            .config_dir()
            .join("claude-shim")
            .join("default-profile"),
    );
    let active_name = env::current_dir().ok().and_then(|cwd| {
        active_profile(&cwd, base.home_dir(), base.config_dir())
            .filter(|name| profile_dir(base.data_dir(), name).is_dir())
    });
    match collect(
        base.data_dir(),
        default_name.as_deref(),
        active_name.as_deref(),
    ) {
        Ok(items) => {
            for p in items {
                let mut tags: Vec<&str> = Vec::new();
                if p.is_default {
                    tags.push("default");
                }
                if p.is_active {
                    tags.push("active");
                }
                if tags.is_empty() {
                    println!("{}", p.name);
                } else {
                    println!("{} ({})", p.name, tags.join(", "));
                }
            }
            ExitCode::SUCCESS
        }
        Err(ListError::Io(path, e)) => {
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
    statusline: bool,
) -> Result<Created, NewError> {
    if !is_valid_profile_name(name) {
        return Err(NewError::InvalidName);
    }
    let dir = profile_dir(data_dir, name);
    make_profile_dir(&dir)?;
    seed_claude_md(&dir)?;
    let statusline_settings = if statusline {
        let path = dir.join("settings.json");
        // The profile dir was just created, so there is no settings.json to
        // clobber; force is moot here.
        set_statusline(
            &path,
            &StatusLine::Preset(StatusLinePreset::ProfileIndicator),
            true,
        )
        .map_err(NewError::Statusline)?;
        Some(path)
    } else {
        None
    };
    let default_marker = if set_default {
        let marker = config_dir.join("claude-shim").join("default-profile");
        if let Some(parent) = marker.parent() {
            std::fs::create_dir_all(parent).map_err(|e| NewError::Io(parent.to_path_buf(), e))?;
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
        statusline_settings,
    })
}

// Seeded into every new profile dir (which the shim exports as CLAUDE_CONFIG_DIR).
// Claude Code auto-loads $CLAUDE_CONFIG_DIR/CLAUDE.md as user memory, so each
// session learns up front that config lives here, not in the overridden ~/.claude.
const PROFILE_CLAUDE_MD: &str = "\
# claude-shim profile

This Claude Code session runs under a claude-shim profile, so its user-level
configuration lives in this directory (the value of `CLAUDE_CONFIG_DIR`) —
not in `~/.claude`.

`CLAUDE_CONFIG_DIR` overrides the default location, so reads and writes to
`~/.claude` (e.g. `~/.claude/settings.json`) are ignored for this session.
To change user-level settings, memory, or commands, edit the files in this
directory instead.

This applies only to the user config directory. A project's own `.claude/`
directory is not affected — use it normally.
";

fn make_profile_dir(dir: &Path) -> Result<(), NewError> {
    if dir.exists() {
        return Err(NewError::AlreadyExists(dir.to_path_buf()));
    }
    std::fs::create_dir_all(dir).map_err(|e| NewError::Io(dir.to_path_buf(), e))
}

fn seed_claude_md(dir: &Path) -> Result<(), NewError> {
    let path = dir.join("CLAUDE.md");
    std::fs::write(&path, PROFILE_CLAUDE_MD).map_err(|e| NewError::Io(path, e))
}

// The one mechanism for installing a statusLine: merge the `statusLine` key
// into the profile's settings.json, preserving every other key. Used by both
// `profile new --statusline` (fresh file) and `profile statusline` (existing).
fn set_statusline(
    settings_path: &Path,
    statusline: &StatusLine,
    force: bool,
) -> Result<(), StatuslineError> {
    let mut root = read_settings(settings_path)?;
    if !force && root.contains_key("statusLine") {
        return Err(StatuslineError::AlreadySet(settings_path.to_path_buf()));
    }
    root.insert(
        "statusLine".to_owned(),
        statusline_block(statusline.command()),
    );
    write_settings(settings_path, root)
}

fn statusline_block(command: &str) -> Value {
    json!({
        "type": "command",
        "command": command,
    })
}

fn read_settings(path: &Path) -> Result<Map<String, Value>, StatuslineError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(StatuslineError::Io(path.to_path_buf(), e)),
    };
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(StatuslineError::NotAnObject(path.to_path_buf())),
        Err(e) => Err(StatuslineError::Parse(path.to_path_buf(), e)),
    }
}

fn write_settings(path: &Path, root: Map<String, Value>) -> Result<(), StatuslineError> {
    let mut serialized =
        serde_json::to_string_pretty(&Value::Object(root)).map_err(StatuslineError::Serialize)?;
    serialized.push('\n');
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| StatuslineError::Io(parent.to_path_buf(), e))?;
    }
    std::fs::write(path, serialized).map_err(|e| StatuslineError::Io(path.to_path_buf(), e))
}

fn report_statusline_error(e: &StatuslineError) {
    match e {
        StatuslineError::AlreadySet(p) => eprintln!(
            "claude-shim: statusLine already set in {} — pass --force to overwrite",
            p.display()
        ),
        StatuslineError::NotAnObject(p) => {
            eprintln!("claude-shim: {} is not a JSON object", p.display());
        }
        StatuslineError::Parse(p, err) => {
            eprintln!("claude-shim: failed to parse {}: {err}", p.display());
        }
        StatuslineError::Serialize(err) => {
            eprintln!("claude-shim: failed to serialize settings: {err}");
        }
        StatuslineError::Io(p, err) => {
            eprintln!("claude-shim: I/O error at {}: {err}", p.display());
        }
    }
}

pub(crate) fn collect(
    data_dir: &Path,
    default_name: Option<&str>,
    active_name: Option<&str>,
) -> Result<Vec<ListedProfile>, ListError> {
    let root = data_dir.join("claude-shim").join("profiles");
    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(ListError::Io(root, e)),
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| ListError::Io(root.clone(), e))?;
        let file_type = entry
            .file_type()
            .map_err(|e| ListError::Io(entry.path(), e))?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_valid_profile_name(&name) {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names
        .into_iter()
        .map(|name| ListedProfile {
            is_default: default_name == Some(name.as_str()),
            is_active: active_name == Some(name.as_str()),
            name,
        })
        .collect())
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

/// The profile active in `cwd` — the one `profile statusline` targets without
/// `--profile`. Same resolution as the shim, mapped to its name; `None` when no
/// profile is in scope.
fn active_profile(cwd: &Path, home: &Path, config_dir: &Path) -> Option<String> {
    match resolve(cwd, home, config_dir) {
        Resolution::Profile(p) => Some(p.name),
        Resolution::Legacy | Resolution::None => None,
    }
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

        let got =
            collect(data.path(), Some("work"), None).unwrap_or_else(|_| panic!("expected Ok"));
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

        let got = collect(data.path(), Some("solo"), Some("solo"))
            .unwrap_or_else(|_| panic!("expected Ok"));
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
}
