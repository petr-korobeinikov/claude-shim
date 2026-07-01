use std::fmt;

use clap::ValueEnum;
use serde_json::{Map, Value};

#[cfg(test)]
mod tests;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum EffortLevel {
    Low,
    Medium,
    High,
    #[value(name = "xhigh")]
    Xhigh,
    Max,
    Auto,
}

impl EffortLevel {
    #[must_use]
    pub(crate) fn as_token(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
            Self::Auto => "auto",
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        Self::value_variants()
            .iter()
            .copied()
            .find(|level| level.as_token() == token)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProjectMarker {
    pub name: String,
    pub effort: Option<EffortLevel>,
    pub warnings: Vec<MarkerWarning>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ProfileConfig {
    pub effort: Option<EffortLevel>,
    pub warnings: Vec<MarkerWarning>,
}

#[derive(Debug)]
pub(crate) enum MarkerError {
    Malformed(serde_json::Error),
    NotAnObject,
    MissingName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MarkerWarning {
    Unusable,
    InvalidEffort(String),
    UnexpectedName,
}

pub(crate) fn parse_project_marker(text: &str) -> Result<ProjectMarker, MarkerError> {
    let obj = parse_object(text)?;
    let name = match obj.get("name") {
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_owned(),
        _ => return Err(MarkerError::MissingName),
    };
    let (effort, warnings) = read_effort(&obj);
    Ok(ProjectMarker {
        name,
        effort,
        warnings,
    })
}

#[must_use]
pub(crate) fn parse_profile_config(text: &str) -> ProfileConfig {
    let Ok(obj) = parse_object(text) else {
        return ProfileConfig {
            effort: None,
            warnings: vec![MarkerWarning::Unusable],
        };
    };
    let mut warnings = Vec::new();
    if obj.contains_key("name") {
        warnings.push(MarkerWarning::UnexpectedName);
    }
    let (effort, effort_warnings) = read_effort(&obj);
    warnings.extend(effort_warnings);
    ProfileConfig { effort, warnings }
}

#[must_use]
pub(crate) fn project_body(name: &str, effort: Option<EffortLevel>) -> String {
    let mut obj = Map::new();
    obj.insert("name".to_owned(), Value::String(name.to_owned()));
    if let Some(level) = effort {
        obj.insert(
            "effort".to_owned(),
            Value::String(level.as_token().to_owned()),
        );
    }
    to_pretty(&Value::Object(obj))
}

#[must_use]
pub(crate) fn profile_default_body(effort: EffortLevel) -> String {
    to_pretty(&serde_json::json!({ "effort": effort.as_token() }))
}

fn to_pretty(value: &Value) -> String {
    let mut body =
        serde_json::to_string_pretty(value).expect("serializing a marker object is infallible");
    body.push('\n');
    body
}

fn read_effort(obj: &Map<String, Value>) -> (Option<EffortLevel>, Vec<MarkerWarning>) {
    match obj.get("effort") {
        None | Some(Value::Null) => (None, Vec::new()),
        Some(Value::String(token)) => match EffortLevel::from_token(token) {
            Some(level) => (Some(level), Vec::new()),
            None => (None, vec![MarkerWarning::InvalidEffort(token.clone())]),
        },
        Some(other) => (None, vec![MarkerWarning::InvalidEffort(other.to_string())]),
    }
}

fn parse_object(text: &str) -> Result<Map<String, Value>, JsonProblem> {
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(JsonProblem::NotAnObject),
        Err(e) => Err(JsonProblem::Malformed(e)),
    }
}

enum JsonProblem {
    Malformed(serde_json::Error),
    NotAnObject,
}

impl From<JsonProblem> for MarkerError {
    fn from(problem: JsonProblem) -> Self {
        match problem {
            JsonProblem::Malformed(e) => Self::Malformed(e),
            JsonProblem::NotAnObject => Self::NotAnObject,
        }
    }
}

impl fmt::Display for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(e) => write!(f, "not valid JSON: {e}"),
            Self::NotAnObject => write!(f, "not a JSON object"),
            Self::MissingName => write!(f, "missing required \"name\" field"),
        }
    }
}

impl fmt::Display for MarkerWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unusable => write!(f, "unparseable config ignored"),
            Self::InvalidEffort(token) => write!(f, "invalid effort {token:?} ignored"),
            Self::UnexpectedName => write!(
                f,
                "\"name\" ignored (a profile default has no name; its directory is its identity)"
            ),
        }
    }
}
