## 1. Assumption

- [x] 1.1 With the operator's yes, read the assistant lines of a current transcript in profile `personal` with `jq` and record `claude --version`. `message.id` and `message.usage` with the three fields, one id over several lines → proceed; otherwise stop and replan the count (design, Decisions). Check: the recorded lines.
  - Claude Code 2.1.311: 295 `assistant` lines, 83 distinct `message.id`, up to 5 lines per id, usage equal across an id's lines; none lacks a field.
  - Sample: `{"id":"msg_011Ch4TMD6RjgUS6L683wLHj","input_tokens":2,"cache_creation_input_tokens":32908,"cache_read_input_tokens":29739,"output_tokens":176}`, twice.

## 2. Session usage

- [x] 2.1 Failing tests in `src/statusline/usage/tests.rs` for `count_session_tokens`: "One call over two lines", a non-assistant line, a line that is not JSON, a missing usage field; then the code. `cargo test` green.
- [x] 2.2 Failing tests for `session_tokens` on a `tempfile` path: a transcript, no file, a count of 0; then the code. `cargo test` green.
- [x] 2.3 Failing tests for `token_text` on "Units", 999 and 1000, and `countdown_text` on 2d 5h 30s, "Hours left", "Under a minute" and exactly one day; then the code. `cargo test` green.

## 3. Lines

- [x] 3.1 Failing `from_stdin` tests: limits rounded, `resets_at` floored minus `now`, passed or absent → `None`, `transcript_path`, a limit that is not a number; then the fields. `cargo test` green.
- [x] 3.2 Replace `LinePreset::effective` with `Palette` (design, Decisions): render tests take a palette, the three `effective` tests become `Colored` under `Plain` equal to `labeled`. `cargo test` and `cargo clippy --all-targets -- -D warnings` green, 308 tests as before.
  - 332 tests before and after: the plan's 308 predates the 24 tests of 2.1-3.1. Clippy's only errors are dead code that 3.3 wires.
- [x] 3.3 Failing tests for `full`: "Every segment" with a `tempfile` transcript, "Weekly window only", no model, "Colored segments", "NO_COLOR on full"; then `LinePreset::Full`. `cargo test` green.
- [x] 3.4 Failing tests for `full-week`: "Days left", "Passed"; then `LinePreset::FullWeek`. `cargo test` green.
- [x] 3.5 A `src/cli/tests.rs` case for `statusline full-week`; `e2e/statusline.bats` pins "Every segment", "Weekly window only", "No transcript", "Days left", "Passed" and "NO_COLOR on full". `cargo build && mise run e2e` green.

## 4. Installing and docs

- [x] 4.1 Failing `src/profile/tests.rs` cases that `full` and `full-week` write their commands; then the `StatusLinePreset` variants; `e2e/profile_statusline.bats` pins "Installing full-week". `cargo test` and `mise run e2e` green.
- [x] 4.2 The operator runs `full-week` from the built binary on profile `personal` beside `statusline-test.sh` in one session and compares tokens, limits and countdown; a difference beyond the last call goes back to the spec as a replan. Check: the operator's report.
  - Both lines equal on Opus 5.5 at `high`: `Tokens: ~33.9k · Week: 76% · Week resets: in 10h 18m`.
- [x] 4.3 `docs/guide/statusline.md` documents both presets with sample lines, each segment's source, cache reads left out, `/clear` and subagents, segments absent without `rate_limits`, and the palette and `NO_COLOR` covering them; every sentence there naming three presets names five. `README.md`, `docs/index.md` and `docs/guide/quick-start.md` are read against it. `mise run docs-build` green.
  - `README.md` gains tokens and limits in its statusLine bullet; `docs/index.md` and `docs/guide/quick-start.md` stay true, unchanged.

## 5. Integration

- [x] 5.1 `cargo test`, `cargo clippy --all-targets -- -D warnings` and `mise run e2e` green together, about 338 unit and 76 e2e tests expected; `mise run coverage` shows `src/statusline/` covered; `time claude-shim statusline full` on the largest transcript in profile `personal` recorded (design, Risks).
  - 341 unit, 76 e2e; coverage: `statusline/mod.rs` 99.35% lines, `statusline/usage/mod.rs` 100%.
  - Largest transcript, 64.6 MB and 1120 lines: release build 26.4 ms per run over 20 warm runs, under 50 ms; its count, 324477, equals `jq`'s.

## Workflow follow-up

- Implemented on `feature/add-full-statusline-presets`, started with `git flow feature start add-full-statusline-presets`.
- `openspec archive add-full-statusline-presets -y` within the branch's last commit; the operator reviews `git diff develop...feature/add-full-statusline-presets`; then `git flow feature finish add-full-statusline-presets --squash-message "feat: add full and full-week statusline presets"`.
- Remove the prototype `statusline-test.sh` of profile `personal` and set its preset with `--force`, by the operator's call.
