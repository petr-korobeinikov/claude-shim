use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::profile::{is_valid_profile_name, profile_dir};

mod dispatch;
mod usage;
pub(crate) use dispatch::print_line;
use usage::{countdown_text, session_tokens, token_text};

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum LinePreset {
    /// `work · Opus 5.5 · high`
    Compact,
    /// `Profile: work · Model: Opus 5.5 (high)`
    Labeled,
    /// The labeled line, colored by cost; plain under `NO_COLOR`
    Colored,
    /// `Profile: work · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Week: 41%`, colored
    Full,
    /// The full line and `Week resets: in 2d 5h`
    FullWeek,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Palette {
    Plain,
    Sgr,
}

impl Palette {
    fn paint(self, text: &str, sgr: Option<&str>) -> String {
        match (self, sgr) {
            (Palette::Sgr, Some(sgr)) => format!("\x1b[{sgr}m{text}\x1b[0m"),
            _ => text.to_string(),
        }
    }
}

#[derive(Default)]
pub(crate) struct SessionFacts {
    pub(crate) profile: Option<String>,
    pub(crate) model_id: Option<String>,
    pub(crate) model_name: Option<String>,
    pub(crate) effort: Option<String>,
    pub(crate) transcript_path: Option<PathBuf>,
    pub(crate) session_percent: Option<u64>,
    pub(crate) week_percent: Option<u64>,
    pub(crate) week_resets_in: Option<u64>,
}

impl SessionFacts {
    pub(crate) fn from_stdin(text: &str, profile: Option<String>, now: u64) -> Self {
        let input = serde_json::from_str(text).unwrap_or(Value::Null);
        let field = |pointer: &str| {
            input
                .pointer(pointer)
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        let number = |pointer: &str| input.pointer(pointer).and_then(Value::as_f64);
        let percent = |pointer: &str| number(pointer).map(f64::round).map(whole_number);
        Self {
            profile,
            model_id: field("/model/id"),
            model_name: field("/model/display_name"),
            effort: field("/effort/level"),
            transcript_path: field("/transcript_path").map(PathBuf::from),
            session_percent: percent("/rate_limits/five_hour/used_percentage"),
            week_percent: percent("/rate_limits/seven_day/used_percentage"),
            week_resets_in: number("/rate_limits/seven_day/resets_at")
                .map(f64::floor)
                .map(whole_number)
                .and_then(|resets_at| resets_at.checked_sub(now))
                .filter(|seconds_left| *seconds_left > 0),
        }
    }
}

// `as` saturates rather than wraps: a negative value reads 0.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole_number(value: f64) -> u64 {
    value as u64
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

pub(crate) fn render(preset: LinePreset, facts: &SessionFacts, palette: Palette) -> String {
    match preset {
        LinePreset::Compact => compact(facts),
        LinePreset::Labeled => labeled(facts, Palette::Plain),
        LinePreset::Colored => labeled(facts, palette),
        LinePreset::Full => join(&full_segment_list(facts, palette), palette),
        LinePreset::FullWeek => join(&full_week_segment_list(facts, palette), palette),
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

fn labeled(facts: &SessionFacts, palette: Palette) -> String {
    let paint = |text, sgr| palette.paint(text, sgr);
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

fn join(segment_list: &[String], palette: Palette) -> String {
    segment_list.join(&palette.paint(SEPARATOR, Some(DIM)))
}

fn segment(label: &str, value: &str, palette: Palette) -> String {
    format!("{} {value}", palette.paint(label, Some(DIM)))
}

fn full_segment_list(facts: &SessionFacts, palette: Palette) -> Vec<String> {
    let mut segment_list = identity_segment_list(facts, palette);
    segment_list.extend(spending_segment_list(facts, palette));
    segment_list
}

fn full_week_segment_list(facts: &SessionFacts, palette: Palette) -> Vec<String> {
    let mut segment_list = full_segment_list(facts, palette);
    segment_list.extend(facts.week_resets_in.map(|seconds_left| {
        let countdown = format!("in {}", countdown_text(seconds_left));
        segment("Week resets:", &countdown, palette)
    }));
    segment_list
}

fn identity_segment_list(facts: &SessionFacts, palette: Palette) -> Vec<String> {
    let mut segment_list = Vec::new();
    if let Some(profile) = &facts.profile {
        segment_list.push(segment("Profile:", profile, palette));
    }
    if let Some(model_name) = &facts.model_name {
        let family = ModelFamily::of(facts.model_id.as_deref(), Some(model_name));
        let model = palette.paint(model_name, family.sgr());
        segment_list.push(segment("Model:", &model, palette));
        if let Some(effort) = &facts.effort {
            let effort = palette.paint(effort, Some(effort_sgr(effort)));
            segment_list.push(segment("Effort:", &effort, palette));
        }
    }
    segment_list
}

fn spending_segment_list(facts: &SessionFacts, palette: Palette) -> Vec<String> {
    let tokens = facts
        .transcript_path
        .as_deref()
        .and_then(session_tokens)
        .map(|count| format!("~{}", token_text(count)));
    let percent_text = |percent: Option<u64>| percent.map(|percent| format!("{percent}%"));
    [
        ("Tokens:", tokens),
        ("Session:", percent_text(facts.session_percent)),
        ("Week:", percent_text(facts.week_percent)),
    ]
    .into_iter()
    .filter_map(|(label, value)| Some(segment(label, &value?, palette)))
    .collect()
}

#[cfg(test)]
mod tests;
