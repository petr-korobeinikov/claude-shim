## Context

`StatusLinePreset` in `src/profile/mod.rs` maps a preset to the shell command written into `settings.json`; `set_statusline` merges it. Nothing renders a line in Rust today: `profile-indicator` is a shell snippet calling `claude-shim profile current`. `src/profile/mod.rs` runs to 992 lines. `serde_json` has no `serde` derive beside it.

## Goals / Non-Goals

**Goals:** rendering lives in the core, as pure functions over plain data, so every row of the spec's color table is a unit test.

**Non-Goals:** user-defined palettes or formats; moving `profile-indicator` onto the new command.

## Decisions

- **A new module, `src/statusline/`,** over growing `src/profile/mod.rs`: rendering a line is its own subject, and the profile module is already past two screens.
- **The render command takes its own enum, `LinePreset { Compact, Labeled, Colored }`,** over reusing `StatusLinePreset`: `profile-indicator` is not rendered by `claude-shim statusline`, and one enum would admit `claude-shim statusline profile-indicator`. `StatusLinePreset` gains `Compact`, `Labeled`, `Colored`, whose commands are `claude-shim statusline compact` and its siblings as `&'static str` constants.
- **Stdin is read through `serde_json::Value` with `pointer`** (`/model/id`, `/model/display_name`, `/effort/level`) over a derived struct: no `serde` derive dependency, and a missing or mistyped field is just `None`.
- **`colored` under `NO_COLOR` is decided in the core,** `LinePreset::effective(no_color)` returning `Labeled`; the glue only reads the variable.
- **The profile check compares paths, not names:** `CLAUDE_CONFIG_DIR`'s parent must equal `profile_dir(data_dir, "")`'s parent, the shim's profiles directory, and its last component must pass `is_valid_profile_name`; otherwise the fallback is `active_profile`, made `pub(crate)`.

Types, one job each:

| Type | Job |
| --- | --- |
| `SessionFacts { profile, model_id, model_name, effort }` | Holds what a line shows, each an `Option<String>`. |
| `LinePreset` | Names the line format the command prints. |
| `ModelFamily { Fable, Opus, Sonnet, Haiku, Unknown }` | Classifies a model for its color. |

Functions in `src/statusline/mod.rs`:

| Function | Does |
| --- | --- |
| `SessionFacts::from_stdin(text, profile)` | Parses stdin; not an object → every model and effort field `None`. |
| `session_profile(config_dir_env, data_dir)` | Returns the profile `CLAUDE_CONFIG_DIR` names, or `None`. |
| `render(preset, &facts) -> String` | Builds the line for a preset. |
| `ModelFamily::of(model_id, model_name)` | First family found, case-insensitive, `id` before name. |

The SGR codes live as constants beside `ModelFamily::sgr` and an effort match, one table as in the spec.

Layers:

- **Core:** `src/statusline/mod.rs`, plus the new `StatusLinePreset` variants.
- **Glue:** `src/statusline/dispatch.rs` reads stdin to a string (a read error is an empty string), `CLAUDE_CONFIG_DIR`, `NO_COLOR`, the cwd and `BaseDirs` (either missing skips the fallback), prints `render`'s line and returns `ExitCode::SUCCESS`. Its name matches the coverage task's `dispatch` exclusion. `src/cli/mod.rs` gains `Command::Statusline { preset: LinePreset }`; `src/lib.rs` wires it.

Testing:

| Part | Test | Seam |
| --- | --- | --- |
| Parsing, rendering, colors, `NO_COLOR` | `src/statusline/tests.rs` | pure functions over strings |
| `session_profile` and fallback | `src/statusline/tests.rs` | paths from a `tempfile` directory |
| Argument parsing | `src/cli/tests.rs` | `Cli::try_parse_from` |
| New preset commands | `src/profile/tests.rs` | `set_statusline` on a `tempfile` settings path |
| Command end to end | `e2e/statusline.bats` | real binary, JSON piped in, `CLAUDE_CONFIG_DIR` set per test |
| Installing a preset | `e2e/profile_statusline.bats` | real binary, `settings.json` read back |

## Risks / Trade-offs

- `claude-shim` missing from the statusLine shell's `PATH` → the line is blank; `profile-indicator` shares the dependency, and the docs name it.
- Claude Code stops passing `CLAUDE_CONFIG_DIR` to the command → the fallback resolves the profile by cwd; task 1.1 checks the current behavior.
- Claude Code adds an effort level, such as `auto` → it is printed as given, dim.
- Yellow `33` reads poorly on a light theme → the palette is one table; the operator checks both themes in task 4.3.
- The command runs on every assistant message, debounced at 300 ms → a Rust binary starts in milliseconds; nothing is cached.

## Migration Plan

None: existing `statusLine` values stay; a profile opts in with `profile statusline --preset <preset> --force`.
