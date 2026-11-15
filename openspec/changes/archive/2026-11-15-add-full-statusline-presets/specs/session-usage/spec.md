## Purpose

Measures what a session has spent - its tokens from the transcript and the time left until the plan's weekly limit resets - for the statusLine presets to show.

## ADDED Requirements

### Requirement: Session tokens

The tokens SHALL be read from the JSONL file at `transcript_path` ([docs](https://code.claude.com/docs/en/statusline)): each distinct `message.id` of a line whose `type` is `assistant` adds its last `message.usage`'s `input_tokens + cache_creation_input_tokens + output_tokens`; cache reads SHALL NOT count. A missing or unreadable file, or a count of 0, SHALL give no count.

#### Scenario: One call over two lines
- **WHEN** two assistant lines share id `msg_1`, each with `input_tokens` 10, `cache_creation_input_tokens` 900, `cache_read_input_tokens` 50000, `output_tokens` 90
- **THEN** the count is 1000

#### Scenario: No transcript
- **WHEN** `transcript_path` names no file
- **THEN** there is no count, and `full` holds no `Tokens:`

### Requirement: Token display

A count SHALL print as an integer under 1000, else rounded to one decimal with `k` for thousands or `M` for millions, the unit chosen after rounding.

#### Scenario: Units
- **WHEN** the count is 812, 66900, 1340000 or 999950
- **THEN** it prints `812`, `66.9k`, `1.3M` or `1.0M`

### Requirement: Weekly reset countdown

The countdown SHALL be `rate_limits.seven_day.resets_at`, in epoch seconds floored, minus now: `<d>d <h>h` for a day or more, `<h>h <m>m` for an hour or more, else `<m>m`, each unit floored. Without `resets_at`, or once it is not after now, there SHALL be no countdown.

#### Scenario: Hours left
- **WHEN** `resets_at` is now plus 3 hours, 7 minutes and 59 seconds
- **THEN** the countdown is `3h 7m`

#### Scenario: Under a minute
- **WHEN** `resets_at` is now plus 30 seconds
- **THEN** the countdown is `0m`

#### Scenario: Passed
- **WHEN** `resets_at` is now minus 1 second
- **THEN** there is no countdown, and `full-week` holds no `Week resets:`
