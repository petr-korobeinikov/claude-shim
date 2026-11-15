use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json::Value;

// Cache reads are left out: every call re-reads the whole context, so
// counting them inflates a 70k session to over a million.
const COUNTED_USAGE_FIELD_LIST: [&str; 3] = [
    "input_tokens",
    "cache_creation_input_tokens",
    "output_tokens",
];

pub(crate) fn session_tokens(path: &Path) -> Option<u64> {
    let file = File::open(path).ok()?;
    let count = count_session_tokens(BufReader::new(file));
    (count > 0).then_some(count)
}

// Claude Code writes one call over several assistant lines that share
// `message.id`; the last line's usage stands for the call.
pub(crate) fn count_session_tokens(reader: impl BufRead) -> u64 {
    let mut call_tokens = HashMap::new();
    for line in reader.lines().map_while(Result::ok) {
        let Ok(entry) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if let Some((id, tokens)) = call_usage(&entry) {
            call_tokens.insert(id, tokens);
        }
    }
    call_tokens.values().sum()
}

fn call_usage(entry: &Value) -> Option<(String, u64)> {
    if entry.get("type")?.as_str()? != "assistant" {
        return None;
    }
    let message = entry.get("message")?;
    let id = message.get("id")?.as_str()?.to_string();
    let usage = message.get("usage");
    let tokens = COUNTED_USAGE_FIELD_LIST
        .into_iter()
        .filter_map(|field| usage?.get(field)?.as_u64())
        .sum();
    Some((id, tokens))
}

// Integer tenths, not `{:.1}` on an `f64`: no binary-float ties, and the
// unit is picked after rounding, so 999950 reads `1.0M`, not `1000.0k`.
pub(crate) fn token_text(count: u64) -> String {
    if count < 1000 {
        return count.to_string();
    }
    let thousand_tenths = (count + 50) / 100;
    if thousand_tenths < 10_000 {
        return tenths_text(thousand_tenths, "k");
    }
    tenths_text((count + 50_000) / 100_000, "M")
}

fn tenths_text(tenths: u64, unit: &str) -> String {
    format!("{}.{}{unit}", tenths / 10, tenths % 10)
}

const SECONDS_PER_MINUTE: u64 = 60;
const SECONDS_PER_HOUR: u64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: u64 = 24 * SECONDS_PER_HOUR;

pub(crate) fn countdown_text(seconds: u64) -> String {
    let days = seconds / SECONDS_PER_DAY;
    let hours = seconds % SECONDS_PER_DAY / SECONDS_PER_HOUR;
    let minutes = seconds % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests;
