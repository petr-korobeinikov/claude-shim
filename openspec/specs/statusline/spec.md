# statusline Specification

## Purpose
Shows the profile, model and effort of a session in Claude Code's status bar, through presets the shim installs and renders.

## Requirements

### Requirement: Rendering a statusLine preset

`claude-shim statusline <preset>`, for `compact`, `labeled` or `colored`, SHALL read the statusLine JSON on stdin, print one line and exit 0. The model is `model.display_name`, the effort `effort.level` ([docs](https://code.claude.com/docs/en/statusline)); a segment without data SHALL be left out. Stdin that is not a JSON object SHALL be read as `{}`.

#### Scenario: Garbage on stdin
- **WHEN** stdin is `not json` and no profile is found
- **THEN** the command prints an empty line and exits 0

#### Scenario: Unknown preset
- **WHEN** the preset is `fancy`
- **THEN** the command exits 2

### Requirement: Profile shown

The profile SHALL be the last component of `CLAUDE_CONFIG_DIR` when it names a directory under the shim's profiles directory, else the profile `claude-shim profile current` resolves here.

#### Scenario: Session profile wins
- **WHEN** `CLAUDE_CONFIG_DIR` is profile `work`'s directory and the current directory is bound to `home`
- **THEN** the line names `work`

#### Scenario: Foreign config dir
- **WHEN** `CLAUDE_CONFIG_DIR` is `/tmp/other` and the current directory is bound to `home`
- **THEN** the line names `home`

### Requirement: Compact and labeled lines

`compact` SHALL print the present segments of profile, model and effort joined by ` · `. `labeled` SHALL print `Profile: <profile>` and `Model: <model> (<effort>)` joined by ` · `, leaving out ` (<effort>)` without an effort, and the effort without a model.

#### Scenario: Every segment
- **WHEN** profile is `work`, `display_name` `Opus 5.5`, `level` `high`
- **THEN** `compact` prints `work · Opus 5.5 · high`
- **AND** `labeled` prints `Profile: work · Model: Opus 5.5 (high)`

#### Scenario: No effort
- **WHEN** profile is `work`, `display_name` `Haiku 5.5`, no `effort`
- **THEN** `compact` prints `work · Haiku 5.5`
- **AND** `labeled` prints `Profile: work · Model: Haiku 5.5`

### Requirement: Colored line

`colored` SHALL print the `labeled` line in SGR codes, each run closed by `0`: labels, ` · ` and parentheses in `2`; the model by the first family found, case ignored, in `model.id`, then `display_name`; the effort by level.

| Model family | SGR | Effort | SGR |
| --- | --- | --- | --- |
| `fable` | `1;38;5;208` | `max` | `1;31` |
| `opus` | `33` | `xhigh` | `33` |
| `sonnet`, unknown | none | `high` | `32` |
| `haiku` | `2` | any other | `2` |

#### Scenario: Fable at max
- **WHEN** `model.id` is `claude-fable-5-1`, `display_name` `Fable 5.1`, `level` `max`
- **THEN** `Fable 5.1` is wrapped in `ESC[1;38;5;208m` and `ESC[0m`, `max` in `ESC[1;31m` and `ESC[0m`

### Requirement: NO_COLOR

With `NO_COLOR` set and non-empty ([no-color.org](https://no-color.org)), `colored` SHALL print the `labeled` line.

#### Scenario: NO_COLOR set
- **WHEN** `colored` runs with `NO_COLOR=1`
- **THEN** the output equals that of `labeled` and holds no `ESC`

### Requirement: Installing a preset

`claude-shim profile statusline --preset` SHALL accept `compact`, `labeled` and `colored` beside `profile-indicator`, writing `claude-shim statusline <preset>` as the profile's `statusLine.command` under the existing `--force` rule. `profile new --statusline` SHALL keep installing `profile-indicator`.

#### Scenario: Installing colored
- **WHEN** `claude-shim profile statusline --profile p --preset colored` runs
- **THEN** `p`'s `settings.json` holds `"command": "claude-shim statusline colored"`
