## MODIFIED Requirements

### Requirement: Rendering a statusLine preset

`claude-shim statusline <preset>`, for `compact`, `labeled`, `colored`, `full` or `full-week`, SHALL read the statusLine JSON on stdin, print one line and exit 0. The model is `model.display_name`, the effort `effort.level` ([docs](https://code.claude.com/docs/en/statusline)); a segment without data SHALL be left out. Stdin that is not a JSON object SHALL be read as `{}`.

#### Scenario: Garbage on stdin
- **WHEN** stdin is `not json` and no profile is found
- **THEN** the command prints an empty line and exits 0

#### Scenario: Unknown preset
- **WHEN** the preset is `fancy`
- **THEN** the command exits 2

### Requirement: NO_COLOR

With `NO_COLOR` set and non-empty ([no-color.org](https://no-color.org)), `colored` SHALL print the `labeled` line, and `full` and `full-week` their lines without SGR codes.

#### Scenario: NO_COLOR set
- **WHEN** `colored` runs with `NO_COLOR=1`
- **THEN** the output equals that of `labeled` and holds no `ESC`

#### Scenario: NO_COLOR on full
- **WHEN** `full` runs with `NO_COLOR=1`
- **THEN** the output holds no `ESC`

### Requirement: Installing a preset

`claude-shim profile statusline --preset` SHALL accept `compact`, `labeled`, `colored`, `full` and `full-week` beside `profile-indicator`, writing `claude-shim statusline <preset>` as the profile's `statusLine.command` under the existing `--force` rule. `profile new --statusline` SHALL keep installing `profile-indicator`.

#### Scenario: Installing colored
- **WHEN** `claude-shim profile statusline --profile p --preset colored` runs
- **THEN** `p`'s `settings.json` holds `"command": "claude-shim statusline colored"`

#### Scenario: Installing full-week
- **WHEN** `claude-shim profile statusline --profile p --preset full-week` runs
- **THEN** `p`'s `settings.json` holds `"command": "claude-shim statusline full-week"`

## ADDED Requirements

### Requirement: Full line

`full` SHALL print the present segments `Profile: <profile>`, `Model: <model>`, `Effort: <effort>`, `Tokens: ~<tokens>` as `session-usage` counts them, `Session: <n>%` and `Week: <n>%` joined by ` · `, leaving out the effort without a model. `Session` is `rate_limits.five_hour.used_percentage`, `Week` is `rate_limits.seven_day.used_percentage`, each rounded to a whole number ([docs](https://code.claude.com/docs/en/statusline)).

#### Scenario: Every segment
- **WHEN** profile is `work`, `display_name` `Opus 5.5`, `level` `high`, the tokens 66900, `five_hour` 23.5, `seven_day` 41.2, under `NO_COLOR=1`
- **THEN** `full` prints `Profile: work · Model: Opus 5.5 · Effort: high · Tokens: ~66.9k · Session: 24% · Week: 41%`

#### Scenario: Weekly window only
- **WHEN** `rate_limits` holds only `seven_day`, at 41
- **THEN** the line ends in `· Week: 41%` and holds no `Session:`

### Requirement: Full line colors

`full` and `full-week` SHALL paint labels and ` · ` in `2`, the model and the effort as `colored` does, and other values plain.

#### Scenario: Colored segments
- **WHEN** `model.id` is `claude-opus-5-5`, `level` `high`, without `NO_COLOR`
- **THEN** `Effort:` is wrapped in `ESC[2m` and `ESC[0m`, `Opus 5.5` in `ESC[33m`, `high` in `ESC[32m`

### Requirement: Full-week line

`full-week` SHALL print the `full` line, then ` · Week resets: in <time>` with the countdown `session-usage` gives, leaving the segment out without one.

#### Scenario: Days left
- **WHEN** `rate_limits.seven_day.resets_at` is now plus 2 days, 5 hours and 30 seconds
- **THEN** the line ends in `· Week resets: in 2d 5h`
