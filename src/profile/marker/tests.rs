use super::*;
use clap::ValueEnum;

// --- EffortLevel ---

#[test]
fn as_token_covers_every_tier() {
    assert_eq!(EffortLevel::Low.as_token(), "low");
    assert_eq!(EffortLevel::Medium.as_token(), "medium");
    assert_eq!(EffortLevel::High.as_token(), "high");
    assert_eq!(EffortLevel::Xhigh.as_token(), "xhigh");
    assert_eq!(EffortLevel::Max.as_token(), "max");
    assert_eq!(EffortLevel::Auto.as_token(), "auto");
}

#[test]
fn from_token_round_trips_every_variant() {
    for level in EffortLevel::value_variants() {
        assert_eq!(EffortLevel::from_token(level.as_token()), Some(*level));
    }
}

#[test]
fn from_token_rejects_unknown_and_is_case_sensitive() {
    assert_eq!(EffortLevel::from_token("ultra"), None);
    assert_eq!(EffortLevel::from_token(""), None);
    assert_eq!(EffortLevel::from_token("HIGH"), None);
    assert_eq!(EffortLevel::from_token("high "), None);
}

/// The token we write and inject, the token we read back, and the token the
/// `--effort` flag accepts must all be the same spelling. Locks them together
/// so a rename of any one surfaces here.
#[test]
fn cli_name_matches_wire_token() {
    for level in EffortLevel::value_variants() {
        let cli_name = level.to_possible_value().unwrap();
        assert_eq!(cli_name.get_name(), level.as_token());
    }
}

// --- parse_project_marker ---

#[test]
fn project_marker_full() {
    let m = parse_project_marker(r#"{"name":"personal","effort":"max"}"#).unwrap();
    assert_eq!(m.name, "personal");
    assert_eq!(m.effort, Some(EffortLevel::Max));
    assert!(m.warnings.is_empty());
}

#[test]
fn project_marker_name_only() {
    let m = parse_project_marker(r#"{"name":"personal"}"#).unwrap();
    assert_eq!(m.name, "personal");
    assert_eq!(m.effort, None);
    assert!(m.warnings.is_empty());
}

#[test]
fn project_marker_trims_name() {
    let m = parse_project_marker(r#"{"name":"  personal  "}"#).unwrap();
    assert_eq!(m.name, "personal");
}

#[test]
fn project_marker_invalid_effort_degrades_with_warning() {
    let m = parse_project_marker(r#"{"name":"personal","effort":"ultra"}"#).unwrap();
    assert_eq!(m.effort, None);
    assert_eq!(
        m.warnings,
        vec![MarkerWarning::InvalidEffort("ultra".into())]
    );
}

#[test]
fn project_marker_non_string_effort_degrades_with_warning() {
    let m = parse_project_marker(r#"{"name":"personal","effort":5}"#).unwrap();
    assert_eq!(m.effort, None);
    assert_eq!(m.warnings, vec![MarkerWarning::InvalidEffort("5".into())]);
}

#[test]
fn project_marker_unknown_keys_tolerated() {
    let m = parse_project_marker(r#"{"name":"personal","future":true}"#).unwrap();
    assert_eq!(m.name, "personal");
    assert!(m.warnings.is_empty());
}

#[test]
fn project_marker_missing_name_is_fatal() {
    assert!(matches!(
        parse_project_marker(r#"{"effort":"max"}"#),
        Err(MarkerError::MissingName)
    ));
}

#[test]
fn project_marker_empty_or_blank_name_is_fatal() {
    assert!(matches!(
        parse_project_marker(r#"{"name":""}"#),
        Err(MarkerError::MissingName)
    ));
    assert!(matches!(
        parse_project_marker(r#"{"name":"   "}"#),
        Err(MarkerError::MissingName)
    ));
}

#[test]
fn project_marker_non_string_name_is_fatal() {
    assert!(matches!(
        parse_project_marker(r#"{"name":5}"#),
        Err(MarkerError::MissingName)
    ));
}

#[test]
fn project_marker_non_object_is_fatal() {
    assert!(matches!(
        parse_project_marker("[1,2]"),
        Err(MarkerError::NotAnObject)
    ));
    assert!(matches!(
        parse_project_marker(r#""hi""#),
        Err(MarkerError::NotAnObject)
    ));
}

#[test]
fn project_marker_malformed_is_fatal() {
    assert!(matches!(
        parse_project_marker("{not json"),
        Err(MarkerError::Malformed(_))
    ));
}

// --- parse_profile_config ---

#[test]
fn profile_config_valid() {
    let c = parse_profile_config(r#"{"effort":"high"}"#);
    assert_eq!(c.effort, Some(EffortLevel::High));
    assert!(c.warnings.is_empty());
}

#[test]
fn profile_config_empty_object() {
    let c = parse_profile_config("{}");
    assert_eq!(c.effort, None);
    assert!(c.warnings.is_empty());
}

#[test]
fn profile_config_invalid_effort_degrades_with_warning() {
    let c = parse_profile_config(r#"{"effort":"ultra"}"#);
    assert_eq!(c.effort, None);
    assert_eq!(
        c.warnings,
        vec![MarkerWarning::InvalidEffort("ultra".into())]
    );
}

#[test]
fn profile_config_name_is_ignored_with_warning() {
    let c = parse_profile_config(r#"{"name":"x","effort":"high"}"#);
    assert_eq!(c.effort, Some(EffortLevel::High));
    assert_eq!(c.warnings, vec![MarkerWarning::UnexpectedName]);
}

#[test]
fn profile_config_collects_both_warnings_in_order() {
    let c = parse_profile_config(r#"{"name":"x","effort":"ultra"}"#);
    assert_eq!(c.effort, None);
    assert_eq!(
        c.warnings,
        vec![
            MarkerWarning::UnexpectedName,
            MarkerWarning::InvalidEffort("ultra".into())
        ]
    );
}

#[test]
fn profile_config_malformed_degrades_to_unusable() {
    let c = parse_profile_config("{oops");
    assert_eq!(c.effort, None);
    assert_eq!(c.warnings, vec![MarkerWarning::Unusable]);
}

#[test]
fn profile_config_non_object_degrades_to_unusable() {
    let c = parse_profile_config("[1]");
    assert_eq!(c.effort, None);
    assert_eq!(c.warnings, vec![MarkerWarning::Unusable]);
}

#[test]
fn project_marker_null_effort_is_unset_without_warning() {
    let m = parse_project_marker(r#"{"name":"personal","effort":null}"#).unwrap();
    assert_eq!(m.effort, None);
    assert!(m.warnings.is_empty());
}

#[test]
fn profile_config_null_effort_is_unset_without_warning() {
    let c = parse_profile_config(r#"{"effort":null}"#);
    assert_eq!(c.effort, None);
    assert!(c.warnings.is_empty());
}

// --- Display (the caller renders these to stderr) ---

#[test]
fn warnings_render_the_offending_detail() {
    assert!(
        MarkerWarning::InvalidEffort("ultra".into())
            .to_string()
            .contains("ultra")
    );
    assert!(MarkerWarning::UnexpectedName.to_string().contains("name"));
    assert!(MarkerWarning::Unusable.to_string().contains("ignored"));
}

#[test]
fn errors_render_a_reason() {
    assert!(MarkerError::NotAnObject.to_string().contains("object"));
    assert!(MarkerError::MissingName.to_string().contains("name"));
}

#[test]
fn project_body_round_trips_name_only() {
    let m = parse_project_marker(&project_body("personal", None)).unwrap();
    assert_eq!(m.name, "personal");
    assert_eq!(m.effort, None);
}

#[test]
fn project_body_round_trips_with_effort() {
    let m = parse_project_marker(&project_body("personal", Some(EffortLevel::Max))).unwrap();
    assert_eq!(m.name, "personal");
    assert_eq!(m.effort, Some(EffortLevel::Max));
}

#[test]
fn profile_default_body_round_trips() {
    let c = parse_profile_config(&profile_default_body(EffortLevel::High));
    assert_eq!(c.effort, Some(EffortLevel::High));
    assert!(c.warnings.is_empty());
}

#[test]
fn bodies_are_pretty_and_newline_terminated() {
    let body = project_body("personal", None);
    assert!(body.contains("\n  \"name\""));
    assert!(body.ends_with("}\n"));
}
