use std::path::Path;

use serde_json::Value;

use crate::profile::{is_valid_profile_name, profile_dir};

mod dispatch;
pub(crate) use dispatch::print_line;

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum LinePreset {
    /// `work · Opus 5.5 · high`
    Compact,
    /// `Profile: work · Model: Opus 5.5 (high)`
    Labeled,
    /// The labeled line, colored by cost; plain under `NO_COLOR`
    Colored,
}

impl LinePreset {
    pub(crate) fn effective(self, no_color: bool) -> Self {
        match self {
            LinePreset::Colored if no_color => LinePreset::Labeled,
            preset => preset,
        }
    }
}

pub(crate) struct SessionFacts {
    pub(crate) profile: Option<String>,
    pub(crate) model_id: Option<String>,
    pub(crate) model_name: Option<String>,
    pub(crate) effort: Option<String>,
}

impl SessionFacts {
    pub(crate) fn from_stdin(text: &str, profile: Option<String>) -> Self {
        let input = serde_json::from_str(text).unwrap_or(Value::Null);
        let field = |pointer: &str| {
            input
                .pointer(pointer)
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        Self {
            profile,
            model_id: field("/model/id"),
            model_name: field("/model/display_name"),
            effort: field("/effort/level"),
        }
    }
}

pub(crate) fn session_profile(config_dir_env: Option<&Path>, data_dir: &Path) -> Option<String> {
    let config_dir = config_dir_env?;
    let name = config_dir.file_name()?.to_str()?;
    let is_profile_dir = is_valid_profile_name(name)
        && profile_dir(data_dir, name) == config_dir
        && config_dir.is_dir();
    is_profile_dir.then(|| name.to_string())
}

const SEPARATOR: &str = " · ";

const DIM: &str = "2";
const BOLD_ORANGE: &str = "1;38;5;208";
const YELLOW: &str = "33";
const GREEN: &str = "32";
const BOLD_RED: &str = "1;31";

#[derive(Copy, Clone)]
enum ModelFamily {
    Fable,
    Opus,
    Sonnet,
    Haiku,
    Unknown,
}

impl ModelFamily {
    const NAMED: [(&'static str, ModelFamily); 4] = [
        ("fable", ModelFamily::Fable),
        ("opus", ModelFamily::Opus),
        ("sonnet", ModelFamily::Sonnet),
        ("haiku", ModelFamily::Haiku),
    ];

    fn of(model_id: Option<&str>, model_name: Option<&str>) -> Self {
        [model_id, model_name]
            .into_iter()
            .flatten()
            .find_map(Self::named_in)
            .unwrap_or(ModelFamily::Unknown)
    }

    fn named_in(text: &str) -> Option<Self> {
        let text = text.to_lowercase();
        Self::NAMED
            .into_iter()
            .find(|(name, _)| text.contains(name))
            .map(|(_, family)| family)
    }

    fn sgr(self) -> Option<&'static str> {
        match self {
            ModelFamily::Fable => Some(BOLD_ORANGE),
            ModelFamily::Opus => Some(YELLOW),
            ModelFamily::Sonnet | ModelFamily::Unknown => None,
            ModelFamily::Haiku => Some(DIM),
        }
    }
}

fn effort_sgr(level: &str) -> &'static str {
    match level {
        "max" => BOLD_RED,
        "xhigh" => YELLOW,
        "high" => GREEN,
        _ => DIM,
    }
}

pub(crate) fn render(preset: LinePreset, facts: &SessionFacts) -> String {
    match preset {
        LinePreset::Compact => compact(facts),
        LinePreset::Labeled => labeled(facts, |text, _| text.to_string()),
        LinePreset::Colored => labeled(facts, paint),
    }
}

fn paint(text: &str, sgr: Option<&str>) -> String {
    match sgr {
        Some(sgr) => format!("\x1b[{sgr}m{text}\x1b[0m"),
        None => text.to_string(),
    }
}

fn compact(facts: &SessionFacts) -> String {
    [&facts.profile, &facts.model_name, &facts.effort]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(SEPARATOR)
}

fn labeled(facts: &SessionFacts, paint: impl Fn(&str, Option<&str>) -> String) -> String {
    let dim = |text| paint(text, Some(DIM));
    let mut segment_list = Vec::new();
    if let Some(profile) = &facts.profile {
        segment_list.push(format!("{} {profile}", dim("Profile:")));
    }
    if let Some(model_name) = &facts.model_name {
        let family = ModelFamily::of(facts.model_id.as_deref(), Some(model_name));
        let effort_run = facts.effort.as_deref().map_or_else(String::new, |effort| {
            let effort = paint(effort, Some(effort_sgr(effort)));
            format!(" {}{effort}{}", dim("("), dim(")"))
        });
        let model_run = paint(model_name, family.sgr());
        segment_list.push(format!("{} {model_run}{effort_run}", dim("Model:")));
    }
    segment_list.join(&dim(SEPARATOR))
}

#[cfg(test)]
mod tests;
