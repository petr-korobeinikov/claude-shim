use std::fs;
use std::path::Path;

use tempfile::TempDir;

use super::{LinePreset, SessionFacts, render, session_profile};
use crate::profile::profile_dir;

fn sgr(code: &str, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

const EVERY_FIELD: &str =
    r#"{"model":{"id":"claude-opus-5-5","display_name":"Opus 5.5"},"effort":{"level":"high"}}"#;

#[test]
fn from_stdin_reads_every_field() {
    let facts = SessionFacts::from_stdin(EVERY_FIELD, Some("work".to_string()));
    assert_eq!(facts.profile.as_deref(), Some("work"));
    assert_eq!(facts.model_id.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(facts.model_name.as_deref(), Some("Opus 5.5"));
    assert_eq!(facts.effort.as_deref(), Some("high"));
}

#[test]
fn from_stdin_without_effort_leaves_it_none() {
    let facts = SessionFacts::from_stdin(r#"{"model":{"display_name":"Haiku 5.5"}}"#, None);
    assert_eq!(facts.model_name.as_deref(), Some("Haiku 5.5"));
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_reads_garbage_as_empty() {
    let facts = SessionFacts::from_stdin("not json", Some("work".to_string()));
    assert_eq!(facts.profile.as_deref(), Some("work"));
    assert_eq!(facts.model_id, None);
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_reads_a_non_object_as_empty() {
    let facts = SessionFacts::from_stdin(r#"["Opus 5.5"]"#, None);
    assert_eq!(facts.model_id, None);
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_ignores_a_mistyped_field() {
    let facts = SessionFacts::from_stdin(
        r#"{"model":{"display_name":5},"effort":{"level":"max"}}"#,
        None,
    );
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort.as_deref(), Some("max"));
}

fn facts(profile: Option<&str>, model_name: Option<&str>, effort: Option<&str>) -> SessionFacts {
    SessionFacts {
        profile: profile.map(str::to_string),
        model_id: None,
        model_name: model_name.map(str::to_string),
        effort: effort.map(str::to_string),
    }
}

#[test]
fn compact_prints_every_segment() {
    let every_segment = facts(Some("work"), Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Compact, &every_segment),
        "work · Opus 5.5 · high"
    );
}

#[test]
fn labeled_prints_every_segment() {
    let every_segment = facts(Some("work"), Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Labeled, &every_segment),
        "Profile: work · Model: Opus 5.5 (high)"
    );
}

#[test]
fn compact_without_effort_leaves_it_out() {
    let no_effort = facts(Some("work"), Some("Haiku 5.5"), None);
    assert_eq!(render(LinePreset::Compact, &no_effort), "work · Haiku 5.5");
}

#[test]
fn labeled_without_effort_leaves_out_the_parentheses() {
    let no_effort = facts(Some("work"), Some("Haiku 5.5"), None);
    assert_eq!(
        render(LinePreset::Labeled, &no_effort),
        "Profile: work · Model: Haiku 5.5"
    );
}

#[test]
fn compact_without_profile_starts_at_the_model() {
    let no_profile = facts(None, Some("Opus 5.5"), Some("high"));
    assert_eq!(render(LinePreset::Compact, &no_profile), "Opus 5.5 · high");
}

#[test]
fn labeled_without_profile_starts_at_the_model() {
    let no_profile = facts(None, Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Labeled, &no_profile),
        "Model: Opus 5.5 (high)"
    );
}

#[test]
fn compact_without_model_keeps_the_effort() {
    let no_model = facts(Some("work"), None, Some("high"));
    assert_eq!(render(LinePreset::Compact, &no_model), "work · high");
}

#[test]
fn labeled_without_model_leaves_out_the_effort() {
    let no_model = facts(Some("work"), None, Some("high"));
    assert_eq!(render(LinePreset::Labeled, &no_model), "Profile: work");
}

#[test]
fn every_preset_prints_nothing_without_facts() {
    let nothing = facts(None, None, None);
    assert_eq!(render(LinePreset::Compact, &nothing), "");
    assert_eq!(render(LinePreset::Labeled, &nothing), "");
}

fn colored_model(model_id: Option<&str>, model_name: &str) -> String {
    let model_only = SessionFacts {
        profile: None,
        model_id: model_id.map(str::to_string),
        model_name: Some(model_name.to_string()),
        effort: None,
    };
    render(LinePreset::Colored, &model_only)
}

fn colored_effort(effort: &str) -> String {
    render(
        LinePreset::Colored,
        &facts(None, Some("Sonnet 5.5"), Some(effort)),
    )
}

fn with_effort(effort_run: &str) -> String {
    format!(
        "{} Sonnet 5.5 {}{effort_run}{}",
        sgr("2", "Model:"),
        sgr("2", "("),
        sgr("2", ")")
    )
}

#[test]
fn colored_paints_fable_bold_orange() {
    assert_eq!(
        colored_model(Some("claude-fable-5-1"), "Fable 5.1"),
        format!("{} {}", sgr("2", "Model:"), sgr("1;38;5;208", "Fable 5.1"))
    );
}

#[test]
fn colored_paints_opus_yellow() {
    assert_eq!(
        colored_model(Some("claude-opus-5-5"), "Opus 5.5"),
        format!("{} {}", sgr("2", "Model:"), sgr("33", "Opus 5.5"))
    );
}

#[test]
fn colored_leaves_sonnet_plain() {
    assert_eq!(
        colored_model(Some("claude-sonnet-5-5"), "Sonnet 5.5"),
        format!("{} Sonnet 5.5", sgr("2", "Model:"))
    );
}

#[test]
fn colored_leaves_an_unknown_model_plain() {
    assert_eq!(
        colored_model(Some("gpt-9"), "Mystery 1"),
        format!("{} Mystery 1", sgr("2", "Model:"))
    );
}

#[test]
fn colored_paints_haiku_dim() {
    assert_eq!(
        colored_model(Some("claude-haiku-5-5"), "Haiku 5.5"),
        format!("{} {}", sgr("2", "Model:"), sgr("2", "Haiku 5.5"))
    );
}

#[test]
fn colored_checks_the_id_before_the_display_name() {
    assert_eq!(
        colored_model(Some("claude-haiku-5-5"), "Opus 5.5"),
        format!("{} {}", sgr("2", "Model:"), sgr("2", "Opus 5.5"))
    );
}

#[test]
fn colored_falls_back_to_the_display_name_ignoring_case() {
    assert_eq!(
        colored_model(None, "FABLE 5.1"),
        format!("{} {}", sgr("2", "Model:"), sgr("1;38;5;208", "FABLE 5.1"))
    );
}

#[test]
fn colored_paints_max_bold_red() {
    assert_eq!(colored_effort("max"), with_effort(&sgr("1;31", "max")));
}

#[test]
fn colored_paints_xhigh_yellow() {
    assert_eq!(colored_effort("xhigh"), with_effort(&sgr("33", "xhigh")));
}

#[test]
fn colored_paints_high_green() {
    assert_eq!(colored_effort("high"), with_effort(&sgr("32", "high")));
}

#[test]
fn colored_paints_any_other_effort_dim() {
    for effort in ["medium", "low", "auto"] {
        assert_eq!(colored_effort(effort), with_effort(&sgr("2", effort)));
    }
}

#[test]
fn colored_prints_fable_at_max_in_full() {
    let input = r#"{"model":{"id":"claude-fable-5-1","display_name":"Fable 5.1"},"effort":{"level":"max"}}"#;
    let fable_at_max = SessionFacts::from_stdin(input, Some("work".to_string()));
    assert_eq!(
        render(LinePreset::Colored, &fable_at_max),
        format!(
            "{} work{}{} {} {}{}{}",
            sgr("2", "Profile:"),
            sgr("2", " · "),
            sgr("2", "Model:"),
            sgr("1;38;5;208", "Fable 5.1"),
            sgr("2", "("),
            sgr("1;31", "max"),
            sgr("2", ")")
        )
    );
}

#[test]
fn effective_turns_colored_into_labeled_under_no_color() {
    assert_eq!(LinePreset::Colored.effective(true), LinePreset::Labeled);
}

#[test]
fn effective_keeps_colored_without_no_color() {
    assert_eq!(LinePreset::Colored.effective(false), LinePreset::Colored);
}

#[test]
fn effective_keeps_compact_under_no_color() {
    assert_eq!(LinePreset::Compact.effective(true), LinePreset::Compact);
}

fn data_dir_with(name: &str) -> TempDir {
    let data_dir = TempDir::new().unwrap();
    fs::create_dir_all(profile_dir(data_dir.path(), name)).unwrap();
    data_dir
}

#[test]
fn session_profile_names_a_profile_directory() {
    let data_dir = data_dir_with("work");
    let config_dir = profile_dir(data_dir.path(), "work");
    assert_eq!(
        session_profile(Some(&config_dir), data_dir.path()).as_deref(),
        Some("work")
    );
}

#[test]
fn session_profile_ignores_a_foreign_directory() {
    let data_dir = data_dir_with("work");
    let foreign = TempDir::new().unwrap();
    let config_dir = foreign.path().join("work");
    fs::create_dir(&config_dir).unwrap();
    assert_eq!(session_profile(Some(&config_dir), data_dir.path()), None);
}

#[test]
fn session_profile_ignores_an_invalid_name() {
    let data_dir = data_dir_with("bad\\name");
    let config_dir = profile_dir(data_dir.path(), "bad\\name");
    assert!(config_dir.is_dir());
    assert_eq!(session_profile(Some(&config_dir), data_dir.path()), None);
}

#[test]
fn session_profile_ignores_the_profiles_directory_itself() {
    let data_dir = data_dir_with("work");
    let config_dir = profile_dir(data_dir.path(), "work").join("..");
    assert_eq!(session_profile(Some(&config_dir), data_dir.path()), None);
}

#[test]
fn session_profile_ignores_a_missing_profile_directory() {
    let data_dir = data_dir_with("work");
    let config_dir = profile_dir(data_dir.path(), "gone");
    assert_eq!(session_profile(Some(&config_dir), data_dir.path()), None);
}

#[test]
fn session_profile_is_none_when_unset() {
    let data_dir = data_dir_with("work");
    assert_eq!(session_profile(None, data_dir.path()), None);
}

#[test]
fn session_profile_accepts_a_trailing_slash() {
    let data_dir = data_dir_with("work");
    let config_dir = format!("{}/", profile_dir(data_dir.path(), "work").display());
    assert_eq!(
        session_profile(Some(Path::new(&config_dir)), data_dir.path()).as_deref(),
        Some("work")
    );
}
