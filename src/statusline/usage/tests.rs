use std::fs;

use tempfile::TempDir;

use super::{count_session_tokens, countdown_text, session_tokens, token_text};

const CALL_LINE: &str = r#"{"type":"assistant","message":{"id":"msg_1","usage":{"input_tokens":10,"cache_creation_input_tokens":900,"cache_read_input_tokens":50000,"output_tokens":90}}}"#;

fn transcript(line_list: &[&str]) -> String {
    line_list.join("\n")
}

#[test]
fn count_session_tokens_counts_one_call_over_two_lines_once() {
    let text = transcript(&[CALL_LINE, CALL_LINE]);
    assert_eq!(count_session_tokens(text.as_bytes()), 1000);
}

#[test]
fn count_session_tokens_sums_distinct_calls() {
    let second_call = r#"{"type":"assistant","message":{"id":"msg_2","usage":{"input_tokens":1,"cache_creation_input_tokens":2,"output_tokens":3}}}"#;
    let text = transcript(&[CALL_LINE, second_call]);
    assert_eq!(count_session_tokens(text.as_bytes()), 1006);
}

#[test]
fn count_session_tokens_takes_the_last_line_of_a_call() {
    let first = r#"{"type":"assistant","message":{"id":"msg_1","usage":{"output_tokens":5}}}"#;
    let last = r#"{"type":"assistant","message":{"id":"msg_1","usage":{"output_tokens":40}}}"#;
    let text = transcript(&[first, last]);
    assert_eq!(count_session_tokens(text.as_bytes()), 40);
}

#[test]
fn count_session_tokens_skips_a_non_assistant_line() {
    let user_line = r#"{"type":"user","message":{"id":"msg_9","usage":{"input_tokens":7000}}}"#;
    let text = transcript(&[user_line, CALL_LINE]);
    assert_eq!(count_session_tokens(text.as_bytes()), 1000);
}

#[test]
fn count_session_tokens_skips_a_line_that_is_not_json() {
    let text = transcript(&["not json", CALL_LINE, "{\"type\":"]);
    assert_eq!(count_session_tokens(text.as_bytes()), 1000);
}

#[test]
fn count_session_tokens_counts_a_missing_usage_field_as_zero() {
    let output_only =
        r#"{"type":"assistant","message":{"id":"msg_1","usage":{"output_tokens":90}}}"#;
    assert_eq!(count_session_tokens(output_only.as_bytes()), 90);
}

#[test]
fn count_session_tokens_skips_a_line_without_an_id() {
    let no_id = r#"{"type":"assistant","message":{"usage":{"output_tokens":90}}}"#;
    assert_eq!(count_session_tokens(no_id.as_bytes()), 0);
}

#[test]
fn session_tokens_counts_a_transcript() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("session.jsonl");
    fs::write(&path, transcript(&[CALL_LINE, CALL_LINE])).unwrap();
    assert_eq!(session_tokens(&path), Some(1000));
}

#[test]
fn session_tokens_is_none_without_a_file() {
    let directory = TempDir::new().unwrap();
    assert_eq!(session_tokens(&directory.path().join("gone.jsonl")), None);
}

#[test]
fn session_tokens_is_none_for_a_count_of_zero() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join("session.jsonl");
    fs::write(&path, r#"{"type":"user"}"#).unwrap();
    assert_eq!(session_tokens(&path), None);
}

#[test]
fn token_text_picks_the_unit_after_rounding() {
    assert_eq!(token_text(812), "812");
    assert_eq!(token_text(66_900), "66.9k");
    assert_eq!(token_text(1_340_000), "1.3M");
    assert_eq!(token_text(999_950), "1.0M");
}

#[test]
fn token_text_prints_999_as_an_integer() {
    assert_eq!(token_text(999), "999");
}

#[test]
fn token_text_prints_1000_in_thousands() {
    assert_eq!(token_text(1000), "1.0k");
}

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

#[test]
fn countdown_text_prints_days_and_hours() {
    assert_eq!(countdown_text(2 * DAY + 5 * HOUR + 30), "2d 5h");
}

#[test]
fn countdown_text_prints_hours_and_minutes() {
    assert_eq!(countdown_text(3 * HOUR + 7 * MINUTE + 59), "3h 7m");
}

#[test]
fn countdown_text_prints_0m_under_a_minute() {
    assert_eq!(countdown_text(30), "0m");
}

#[test]
fn countdown_text_prints_exactly_one_day_in_days() {
    assert_eq!(countdown_text(DAY), "1d 0h");
}
