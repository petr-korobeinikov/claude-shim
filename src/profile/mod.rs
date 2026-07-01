use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Map, Value, json};

mod dispatch;
mod marker;
pub(crate) use dispatch::{current, list, new, statusline, use_profile};
pub(crate) use marker::{EffortLevel, MarkerWarning, project_body};

pub(crate) enum Resolution {
    Profile(ProfileRef),
    Legacy,
    None,
    Malformed(MarkerFault),
}

pub(crate) struct ProfileRef {
    pub name: String,
    source: ProfileSource,
    pub marker: PathBuf,
    effort_override: Option<EffortLevel>,
    warnings: Vec<MarkerWarning>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ProfileSource {
    Project,
    Default,
}

struct MarkerHit {
    name: String,
    effort: Option<EffortLevel>,
    path: PathBuf,
    warnings: Vec<MarkerWarning>,
}

#[derive(Debug)]
pub(crate) struct MarkerFault {
    pub path: PathBuf,
    pub reason: String,
}

pub(crate) struct EffortResolution {
    pub level: Option<EffortLevel>,
    pub warnings: Vec<(PathBuf, MarkerWarning)>,
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

/// The three base directories a command resolves against, bundled so call sites
/// name each one — preventing transposition of the otherwise-interchangeable
/// `&Path` arguments. `cwd` stays a separate argument: it is required for some
/// entry points and optional for others.
#[derive(Copy, Clone)]
pub(crate) struct Dirs<'a> {
    pub data_dir: &'a Path,
    pub config_dir: &'a Path,
    pub home: &'a Path,
}

fn current_at(dirs: &Dirs, cwd: &Path, out: &mut impl Write) -> ExitCode {
    match resolve(cwd, dirs.home, dirs.config_dir) {
        Resolution::Profile(p) => emit(&p.name, dirs.data_dir, p.source, out),
        Resolution::Legacy | Resolution::None | Resolution::Malformed(_) => ExitCode::SUCCESS,
    }
}

fn new_at(
    data_dir: &Path,
    config_dir: &Path,
    name: &str,
    set_default: bool,
    statusline: bool,
) -> ExitCode {
    match create(data_dir, config_dir, name, set_default, statusline) {
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
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn statusline_at(
    dirs: &Dirs,
    cwd: Option<&Path>,
    profile: Option<&str>,
    preset: Option<StatusLinePreset>,
    command: Option<String>,
    force: bool,
) -> ExitCode {
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
        let Some(cwd) = cwd else {
            eprintln!("claude-shim: unable to read current directory");
            return ExitCode::from(2);
        };
        let Some(name) = active_profile(cwd, dirs.home, dirs.config_dir) else {
            eprintln!("claude-shim: no active profile here — pass --profile <name>");
            return ExitCode::from(2);
        };
        name
    };
    if !is_valid_profile_name(&name) {
        eprintln!("claude-shim: invalid profile name '{name}'");
        return ExitCode::from(2);
    }
    let dir = profile_dir(dirs.data_dir, &name);
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
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn use_profile_at(cwd: &Path, data_dir: &Path, name: &str, workspace: bool) -> ExitCode {
    match apply(cwd, data_dir, name, workspace) {
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

fn list_at(dirs: &Dirs, cwd: Option<&Path>, out: &mut impl Write) -> ExitCode {
    let default_name =
        read_marker_file(&dirs.config_dir.join("claude-shim").join("default-profile"));
    let active_name = cwd.and_then(|cwd| {
        active_profile(cwd, dirs.home, dirs.config_dir)
            .filter(|name| profile_dir(dirs.data_dir, name).is_dir())
    });
    match collect(
        dirs.data_dir,
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
                let line = if tags.is_empty() {
                    p.name.clone()
                } else {
                    format!("{} ({})", p.name, tags.join(", "))
                };
                if let Err(e) = writeln!(out, "{line}") {
                    return on_write_error(&e);
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
        cwd.join(".claude-shim.json")
    } else {
        cwd.join(".claude").join("claude-shim.json")
    };
    if marker.exists() {
        return Err(UseError::MarkerAlreadyExists(marker));
    }
    if let Some(parent) = marker.parent() {
        std::fs::create_dir_all(parent).map_err(|e| UseError::Io(parent.to_path_buf(), e))?;
    }
    std::fs::write(&marker, project_body(name)).map_err(|e| UseError::Io(marker.clone(), e))?;
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

impl fmt::Display for StatuslineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadySet(p) => write!(
                f,
                "claude-shim: statusLine already set in {} — pass --force to overwrite",
                p.display()
            ),
            Self::NotAnObject(p) => write!(f, "claude-shim: {} is not a JSON object", p.display()),
            Self::Parse(p, err) => write!(f, "claude-shim: failed to parse {}: {err}", p.display()),
            Self::Serialize(err) => write!(f, "claude-shim: failed to serialize settings: {err}"),
            Self::Io(p, err) => write!(f, "claude-shim: I/O error at {}: {err}", p.display()),
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
    match find_project_marker(cwd, Some(home)) {
        Some(Ok(hit)) => {
            return Resolution::Profile(ProfileRef {
                name: hit.name,
                source: ProfileSource::Project,
                marker: hit.path,
                effort_override: hit.effort,
                warnings: hit.warnings,
            });
        }
        Some(Err(fault)) => return Resolution::Malformed(fault),
        None => {}
    }
    let default_marker = config_dir.join("claude-shim").join("default-profile");
    if let Some(name) = read_marker_file(&default_marker).filter(|name| is_valid_profile_name(name))
    {
        return Resolution::Profile(ProfileRef {
            name,
            source: ProfileSource::Default,
            marker: default_marker,
            effort_override: None,
            warnings: Vec::new(),
        });
    }
    if home.join(".claude").is_dir() {
        return Resolution::Legacy;
    }
    Resolution::None
}

pub(crate) fn resolve_effort(data_dir: &Path, profile: &ProfileRef) -> EffortResolution {
    let mut warnings: Vec<(PathBuf, MarkerWarning)> = profile
        .warnings
        .iter()
        .cloned()
        .map(|w| (profile.marker.clone(), w))
        .collect();
    if let Some(level) = profile.effort_override {
        return EffortResolution {
            level: Some(level),
            warnings,
        };
    }
    let config_path = profile_dir(data_dir, &profile.name).join("claude-shim.json");
    let Ok(text) = std::fs::read_to_string(&config_path) else {
        return EffortResolution {
            level: None,
            warnings,
        };
    };
    let config = marker::parse_profile_config(&text);
    warnings.extend(
        config
            .warnings
            .into_iter()
            .map(|w| (config_path.clone(), w)),
    );
    EffortResolution {
        level: config.effort,
        warnings,
    }
}

/// The profile active in `cwd` — the one `profile statusline` targets without
/// `--profile`. Same resolution as the shim, mapped to its name; `None` when no
/// profile is in scope.
fn active_profile(cwd: &Path, home: &Path, config_dir: &Path) -> Option<String> {
    match resolve(cwd, home, config_dir) {
        Resolution::Profile(p) => Some(p.name),
        Resolution::Legacy | Resolution::None | Resolution::Malformed(_) => None,
    }
}

fn find_project_marker(
    start: &Path,
    stop_at: Option<&Path>,
) -> Option<Result<MarkerHit, MarkerFault>> {
    for dir in start.ancestors() {
        if matches!(stop_at, Some(s) if dir == s) {
            break;
        }
        for path in [
            dir.join(".claude").join("claude-shim.json"),
            dir.join(".claude-shim.json"),
        ] {
            if path.is_file() {
                return Some(load_project_marker(path));
            }
        }
    }
    None
}

fn load_project_marker(path: PathBuf) -> Result<MarkerHit, MarkerFault> {
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            return Err(MarkerFault {
                reason: format!("cannot read: {e}"),
                path,
            });
        }
    };
    let parsed = match marker::parse_project_marker(&text) {
        Ok(parsed) => parsed,
        Err(e) => {
            return Err(MarkerFault {
                reason: e.to_string(),
                path,
            });
        }
    };
    if !is_valid_profile_name(&parsed.name) {
        return Err(MarkerFault {
            reason: format!("invalid profile name {:?}", parsed.name),
            path,
        });
    }
    Ok(MarkerHit {
        name: parsed.name,
        effort: parsed.effort,
        path,
        warnings: parsed.warnings,
    })
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

/// Map a stdout write failure to an exit code: a closed pipe (e.g. piping into
/// `head`) is a clean stop reported as success; any other write error is
/// surfaced and fails. Used by the output commands (`current`/`list`); the
/// post-action confirmation messages elsewhere keep plain `println!`.
fn on_write_error(e: &io::Error) -> ExitCode {
    if e.kind() == io::ErrorKind::BrokenPipe {
        ExitCode::SUCCESS
    } else {
        eprintln!("claude-shim: failed to write output: {e}");
        ExitCode::from(2)
    }
}

fn emit(name: &str, data_dir: &Path, source: ProfileSource, out: &mut impl Write) -> ExitCode {
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
        match writeln!(out, "{name}") {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => on_write_error(&e),
        }
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
mod tests;
