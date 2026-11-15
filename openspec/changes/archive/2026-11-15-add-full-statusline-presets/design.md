## Context

`src/statusline/mod.rs` (166 lines) parses stdin into `SessionFacts` through `serde_json::Value::pointer` and renders `LinePreset { Compact, Labeled, Colored }`; `LinePreset::effective(no_color)` turns `Colored` into `Labeled`. `dispatch.rs` reads stdin, `NO_COLOR` and the profile, and always exits 0. `StatusLinePreset` in `src/profile/mod.rs` maps each preset to its `&'static str` command. 308 unit tests, 36 of them in `src/statusline/tests.rs`; 69 e2e tests.

## Goals / Non-Goals

**Goals:** every spec scenario is a unit test over a string, a number or a `tempfile` transcript; the clock enters as a parameter.

**Non-Goals:** subagent transcripts; caching the count between refreshes; the 5-hour window's reset.

## Decisions

- **Color is a parameter, `Palette { Plain, Sgr }`,** passed to `render(preset, &facts, palette)`, over hidden plain twins of `full` and `full-week` behind `#[value(skip)]`: one value covers three colored presets. `LinePreset::effective` and its three tests go; `Colored` under `Plain` is the `labeled` line by construction.
- **The clock enters at parsing:** `SessionFacts::from_stdin(text, profile, now)` stores `week_resets_in: Option<u64>`, seconds left, over passing `now` to `render`: the countdown is computed once, where `resets_at` is read, and `render` stays clock-free.
- **`render` reads the transcript, only for `full` and `full-week`,** from `facts.transcript_path`, over the glue reading it: which preset needs the file is a decision, and `compact`, `labeled`, `colored` never open it.
- **The transcript is read line by line** through `BufRead::lines` and `serde_json::from_str::<Value>`, skipping a line that does not parse, into a `HashMap<String, u64>` from `message.id` to the latest line's sum; a missing usage field counts 0, as in the prototype's `jq`.
- **Token display uses integer tenths,** `(count + 50) / 100` for `k`, switching to `(count + 50_000) / 100_000` for `M` once the tenths reach 10000, over `format!("{:.1}")` on `f64`: no binary-float ties, and `999950` reads `1.0M`.
- **Limits are `f64`,** `used_percentage.round()` and `resets_at.floor()` cast to `u64`; a value that is not a number is `None`.

Types and functions, one job each:

| Item | Job |
| --- | --- |
| `Palette { Plain, Sgr }` | Names whether a line carries SGR codes. |
| `LinePreset` + `Full`, `FullWeek` | `clap` gives `full`, `full-week`. |
| `SessionFacts` + `transcript_path`, `session_percent`, `week_percent`, `week_resets_in` | Holds what a line shows. |
| `StatusLinePreset` + `Full`, `FullWeek` | Commands `claude-shim statusline full`, `… full-week`. |
| `usage::count_session_tokens(reader) -> u64` | Sums a transcript's calls. |
| `usage::session_tokens(path) -> Option<u64>` | Opens the file; error or 0 → `None`. |
| `usage::token_text(count) -> String` | `812`, `66.9k`, `1.3M`. |
| `usage::countdown_text(seconds) -> String` | `2d 5h`, `3h 7m`, `0m`. |

`src/statusline/usage/` is a module of its own, subject `session-usage`, so `mod.rs` keeps rendering only. The `full` line builds two segment lists - profile, model and effort; tokens and limits - so no function passes 40 lines.

Layers:

- **Core:** `src/statusline/mod.rs`, `src/statusline/usage/mod.rs`, the `StatusLinePreset` variants.
- **Glue:** `dispatch.rs` reads `SystemTime::now()` as epoch seconds (before the epoch → 0) and maps `NO_COLOR` to `Palette`; no branch on the preset.

Testing:

| Part | Test | Seam |
| --- | --- | --- |
| Parsing the new fields, `week_resets_in` | `src/statusline/tests.rs` | `from_stdin` with a fixed `now` |
| `full`, `full-week`, colors, `Plain` | `src/statusline/tests.rs` | `render` over facts; a transcript in a `tempfile` directory |
| Count, display, countdown | `src/statusline/usage/tests.rs` | `&[u8]` readers, numbers, a `tempfile` path |
| Argument parsing | `src/cli/tests.rs` | `Cli::try_parse_from` |
| Preset commands | `src/profile/tests.rs` | `set_statusline` on a `tempfile` path |
| Command end to end | `e2e/statusline.bats` | real binary; transcript under `$BATS_TEST_TMPDIR`; `resets_at` from `date +%s` with 30 s of slack |
| Installing | `e2e/profile_statusline.bats` | `settings.json` read back |

## Risks / Trade-offs

- Claude Code changes the transcript format → the segment drops out or miscounts; task 1.1 checks it, and the docs call the count approximate.
- A long session's transcript is read on every refresh → task 5.1 times the command on the largest transcript in profile `personal`; caching is a later change if it passes 50 ms.
- `rate_limits` is absent for API-key users and before the first response → both limit segments drop out, as the spec requires.
- `/clear` starts a new transcript → the count restarts; the docs say so.

## Migration Plan

None: existing `statusLine` values stay; a profile opts in with `profile statusline --preset full --force`.
