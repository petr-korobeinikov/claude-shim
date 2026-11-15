use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

use super::{LinePreset, Palette, SessionFacts, render, session_profile};
use crate::profile::profile_dir;

fn sgr(code: &str, text: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}

const EVERY_FIELD: &str =
    r#"{"model":{"id":"claude-opus-5-5","display_name":"Opus 5.5"},"effort":{"level":"high"}}"#;

#[test]
fn from_stdin_reads_every_field() {
    let facts = SessionFacts::from_stdin(EVERY_FIELD, Some("work".to_string()), NOW);
    assert_eq!(facts.profile.as_deref(), Some("work"));
    assert_eq!(facts.model_id.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(facts.model_name.as_deref(), Some("Opus 5.5"));
    assert_eq!(facts.effort.as_deref(), Some("high"));
}

#[test]
fn from_stdin_without_effort_leaves_it_none() {
    let facts = SessionFacts::from_stdin(r#"{"model":{"display_name":"Haiku 5.5"}}"#, None, NOW);
    assert_eq!(facts.model_name.as_deref(), Some("Haiku 5.5"));
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_reads_garbage_as_empty() {
    let facts = SessionFacts::from_stdin("not json", Some("work".to_string()), NOW);
    assert_eq!(facts.profile.as_deref(), Some("work"));
    assert_eq!(facts.model_id, None);
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_reads_a_non_object_as_empty() {
    let facts = SessionFacts::from_stdin(r#"["Opus 5.5"]"#, None, NOW);
    assert_eq!(facts.model_id, None);
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort, None);
}

#[test]
fn from_stdin_ignores_a_mistyped_field() {
    let facts = SessionFacts::from_stdin(
        r#"{"model":{"display_name":5},"effort":{"level":"max"}}"#,
        None,
        NOW,
    );
    assert_eq!(facts.model_name, None);
    assert_eq!(facts.effort.as_deref(), Some("max"));
}

const NOW: u64 = 1_800_000_000;

fn facts_from(input: &str) -> SessionFacts {
    SessionFacts::from_stdin(input, None, NOW)
}

#[test]
fn from_stdin_rounds_the_limits() {
    let facts = facts_from(
        r#"{"rate_limits":{"five_hour":{"used_percentage":23.5},"seven_day":{"used_percentage":41.2}}}"#,
    );
    assert_eq!(facts.session_percent, Some(24));
    assert_eq!(facts.week_percent, Some(41));
}

#[test]
fn from_stdin_leaves_an_absent_window_none() {
    let facts = facts_from(r#"{"rate_limits":{"seven_day":{"used_percentage":41}}}"#);
    assert_eq!(facts.session_percent, None);
    assert_eq!(facts.week_percent, Some(41));
}

#[test]
fn from_stdin_ignores_a_limit_that_is_not_a_number() {
    let facts =
        facts_from(r#"{"rate_limits":{"seven_day":{"used_percentage":"41","resets_at":"soon"}}}"#);
    assert_eq!(facts.week_percent, None);
    assert_eq!(facts.week_resets_in, None);
}

#[test]
fn from_stdin_floors_resets_at_and_subtracts_now() {
    let facts = facts_from(r#"{"rate_limits":{"seven_day":{"resets_at":1800000090.9}}}"#);
    assert_eq!(facts.week_resets_in, Some(90));
}

#[test]
fn from_stdin_leaves_a_passed_reset_none() {
    for resets_at in [NOW - 1, NOW] {
        let input = format!(r#"{{"rate_limits":{{"seven_day":{{"resets_at":{resets_at}}}}}}}"#);
        assert_eq!(facts_from(&input).week_resets_in, None);
    }
}

#[test]
fn from_stdin_leaves_an_absent_reset_none() {
    assert_eq!(facts_from("{}").week_resets_in, None);
}

#[test]
fn from_stdin_reads_the_transcript_path() {
    let facts = facts_from(r#"{"transcript_path":"/tmp/session.jsonl"}"#);
    assert_eq!(
        facts.transcript_path.as_deref(),
        Some(Path::new("/tmp/session.jsonl"))
    );
}

fn facts(profile: Option<&str>, model_name: Option<&str>, effort: Option<&str>) -> SessionFacts {
    SessionFacts {
        profile: profile.map(str::to_string),
        model_id: None,
        model_name: model_name.map(str::to_string),
        effort: effort.map(str::to_string),
        ..SessionFacts::default()
    }
}

#[test]
fn compact_prints_every_segment() {
    let every_segment = facts(Some("work"), Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Compact, &every_segment, Palette::Sgr),
        "work · Opus 5.5 · high"
    );
}

#[test]
fn labeled_prints_every_segment() {
    let every_segment = facts(Some("work"), Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Labeled, &every_segment, Palette::Sgr),
        "Profile: work · Model: Opus 5.5 (high)"
    );
}

#[test]
fn compact_without_effort_leaves_it_out() {
    let no_effort = facts(Some("work"), Some("Haiku 5.5"), None);
    assert_eq!(
        render(LinePreset::Compact, &no_effort, Palette::Sgr),
        "work · Haiku 5.5"
    );
}

#[test]
fn labeled_without_effort_leaves_out_the_parentheses() {
    let no_effort = facts(Some("work"), Some("Haiku 5.5"), None);
    assert_eq!(
        render(LinePreset::Labeled, &no_effort, Palette::Sgr),
        "Profile: work · Model: Haiku 5.5"
    );
}

#[test]
fn compact_without_profile_starts_at_the_model() {
    let no_profile = facts(None, Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Compact, &no_profile, Palette::Sgr),
        "Opus 5.5 · high"
    );
}

#[test]
fn labeled_without_profile_starts_at_the_model() {
    let no_profile = facts(None, Some("Opus 5.5"), Some("high"));
    assert_eq!(
        render(LinePreset::Labeled, &no_profile, Palette::Sgr),
        "Model: Opus 5.5 (high)"
    );
}

#[test]
fn compact_without_model_keeps_the_effort() {
    let no_model = facts(Some("work"), None, Some("high"));
    assert_eq!(
        render(LinePreset::Compact, &no_model, Palette::Sgr),
        "work · high"
    );
}

#[test]
fn labeled_without_model_leaves_out_the_effort() {
    let no_model = facts(Some("work"), None, Some("high"));
    assert_eq!(
        render(LinePreset::Labeled, &no_model, Palette::Sgr),
        "Profile: work"
    );
}

#[test]
fn every_preset_prints_nothing_without_facts() {
    let nothing = facts(None, None, None);
    assert_eq!(render(LinePreset::Compact, &nothing, Palette::Sgr), "");
    assert_eq!(render(LinePreset::Labeled, &nothing, Palette::Sgr), "");
}

fn colored_model(model_id: Option<&str>, model_name: &str) -> String {
    let model_only = SessionFacts {
        profile: None,
        model_id: model_id.map(str::to_string),
        model_name: Some(model_name.to_string()),
        effort: None,
        ..SessionFacts::default()
    };
    render(LinePreset::Colored, &model_only, Palette::Sgr)
}

fn colored_effort(effort: &str) -> String {
    render(
        LinePreset::Colored,
        &facts(None, Some("Sonnet 5.5"), Some(effort)),
        Palette::Sgr,
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
    let fable_at_max = SessionFacts::from_stdin(input, Some("work".to_string()), NOW);
    assert_eq!(
        render(LinePreset::Colored, &fable_at_max, Palette::Sgr),
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

fn every_field() -> SessionFacts {
    SessionFacts::from_stdin(EVERY_FIELD, Some("work".to_string()), NOW)
}

#[test]
fn colored_under_plain_prints_the_labeled_line() {
    assert_eq!(
        render(LinePreset::Colored, &every_field(), Palette::Plain),
        render(LinePreset::Labeled, &every_field(), Palette::Sgr)
    );
}

#[test]
fn colored_under_sgr_differs_from_labeled() {
    assert_ne!(
        render(LinePreset::Colored, &every_field(), Palette::Sgr),
        render(LinePreset::Labeled, &every_field(), Palette::Sgr)
    );
}

#[test]
fn compact_is_the_same_under_either_palette() {
    assert_eq!(
        render(LinePreset::Compact, &every_field(), Palette::Plain),
        render(LinePreset::Compact, &every_field(), Palette::Sgr)
    );
}

const CALL_OF_66900: &str = r#"{"type":"assistant","message":{"id":"msg_1","usage":{"input_tokens":900,"cache_creation_input_tokens":60000,"cache_read_input_tokens":500000,"output_tokens":6000}}}"#;

fn full_input(transcript_path: &Path, rate_limits: &Value) -> String {
    json!({
        "model": {"id": "claude-opus-5-5", "display_name": "Opus 5.5"},
        "effort": {"level": "high"},
        "transcript_path": transcript_path,
        "rate_limits": rate_limits,
    })
    .to_string()
}

fn every_segment(directory: &TempDir) -> SessionFacts {
    let transcript_path = directory.path().join("session.jsonl");
    fs::write(&transcript_path, CALL_OF_66900).unwrap();
    let rate_limits = json!({
        "five_hour": {"used_percentage": 23.5},
        "seven_day": {"used_percentage": 41.2},
    });
    let input = full_input(&transcript_path, &rate_limits);
    SessionFacts::from_stdin(&input, Some("work".to_string()), NOW)
}

#[test]
fn full_prints_every_segment() {
    let directory = TempDir::new().unwrap();
    assert_eq!(
        render(LinePreset::Full, &every_segment(&directory), Palette::Plain),
        "Profile: work · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Session: 24% · Week: 41%"
    );
}

#[test]
fn full_with_the_weekly_window_only_holds_no_session() {
    let directory = TempDir::new().unwrap();
    let rate_limits = json!({"seven_day": {"used_percentage": 41}});
    let input = full_input(&directory.path().join("gone.jsonl"), &rate_limits);
    let line = render(
        LinePreset::Full,
        &SessionFacts::from_stdin(&input, None, NOW),
        Palette::Plain,
    );
    assert!(line.ends_with(" · Week: 41%"), "{line}");
    assert!(!line.contains("Session:"), "{line}");
}

#[test]
fn full_without_a_transcript_holds_no_tokens() {
    let directory = TempDir::new().unwrap();
    let input = full_input(&directory.path().join("gone.jsonl"), &json!({}));
    let line = render(
        LinePreset::Full,
        &SessionFacts::from_stdin(&input, None, NOW),
        Palette::Plain,
    );
    assert_eq!(line, "Model: Opus 5.5 · Effort: high");
}

#[test]
fn full_without_model_leaves_out_the_effort() {
    let no_model = facts(Some("work"), None, Some("high"));
    assert_eq!(
        render(LinePreset::Full, &no_model, Palette::Plain),
        "Profile: work"
    );
}

#[test]
fn full_paints_labels_dim_and_the_model_and_effort_by_cost() {
    let directory = TempDir::new().unwrap();
    let line = render(LinePreset::Full, &every_segment(&directory), Palette::Sgr);
    assert!(line.contains(&sgr("2", "Effort:")), "{line:?}");
    assert!(line.contains(&sgr("33", "Opus 5.5")), "{line:?}");
    assert!(line.contains(&sgr("32", "high")), "{line:?}");
    assert!(line.contains(&sgr("2", " · ")), "{line:?}");
    assert!(
        line.ends_with(&format!("{} 41%", sgr("2", "Week:"))),
        "{line:?}"
    );
}

#[test]
fn full_under_plain_holds_no_escape() {
    let directory = TempDir::new().unwrap();
    let line = render(LinePreset::Full, &every_segment(&directory), Palette::Plain);
    assert!(!line.contains('\x1b'), "{line:?}");
}

fn full_week_line(resets_at: u64) -> String {
    let directory = TempDir::new().unwrap();
    let rate_limits = json!({"seven_day": {"used_percentage": 41, "resets_at": resets_at}});
    let input = full_input(&directory.path().join("gone.jsonl"), &rate_limits);
    render(
        LinePreset::FullWeek,
        &SessionFacts::from_stdin(&input, None, NOW),
        Palette::Plain,
    )
}

#[test]
fn full_week_counts_down_to_the_weekly_reset() {
    let two_days_five_hours = 2 * 86_400 + 5 * 3600 + 30;
    assert_eq!(
        full_week_line(NOW + two_days_five_hours),
        "Model: Opus 5.5 · Effort: high · Week: 41% · Week resets: in 2d 5h"
    );
}

#[test]
fn full_week_after_the_reset_prints_the_full_line() {
    assert_eq!(
        full_week_line(NOW - 1),
        "Model: Opus 5.5 · Effort: high · Week: 41%"
    );
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
