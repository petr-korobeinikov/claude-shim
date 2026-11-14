## 1. Assumption

- [ ] 1.1 The operator sets a profile's `statusLine.command` to `printf %s "$CLAUDE_CONFIG_DIR"` in a session started through the shim and reports the line and `claude --version`. The profile directory → proceed; blank → stop and replan the profile source (design, Decisions).

## 2. Rendering core

- [ ] 2.1 Failing tests first in `src/statusline/tests.rs` for `SessionFacts::from_stdin`: every field, missing `effort`, `not json`, a non-object. Then `src/statusline/mod.rs` per design. `cargo test` green.
- [ ] 2.2 Failing tests for `render` with `compact` and `labeled` on the spec's "Every segment" and "No effort" inputs, no profile, and no model; then the code. `cargo test` green.
- [ ] 2.3 Failing tests for `colored`: one per row of the spec's table for model and effort, `id` checked before `display_name`, and `LinePreset::effective(true)` giving `Labeled`; then the code. `cargo test` green.
- [ ] 2.4 Failing tests for `session_profile` on a `tempfile` data directory: a profile directory, a foreign directory, an invalid name, unset; then the code, `active_profile` made `pub(crate)`. `cargo test` green.

## 3. Command

- [ ] 3.1 A failing `src/cli/tests.rs` case for `statusline colored` and one for `statusline fancy`; then `Command::Statusline`, `src/statusline/dispatch.rs` and `src/lib.rs` wiring. `cargo test` and `cargo clippy --all-targets -- -D warnings` green.
- [ ] 3.2 `e2e/statusline.bats` pins each scenario of the spec's first three requirements and "Fable at max" and "NO_COLOR set", JSON piped in. `cargo build && mise run e2e` green.

## 4. Installing presets and docs

- [ ] 4.1 Failing `src/profile/tests.rs` cases that `compact`, `labeled` and `colored` write `claude-shim statusline <preset>`; then the `StatusLinePreset` variants. `cargo test` green.
- [ ] 4.2 `e2e/profile_statusline.bats` pins "Installing colored". `cargo build && mise run e2e` green.
- [ ] 4.3 The operator installs `colored` on a profile with the built binary on `PATH` and checks Opus and Fable at `high` and `max` on dark and light themes; a palette change goes back to the spec as a replan.
- [ ] 4.4 `docs/guide/statusline.md` documents the three presets with sample lines, the color table, `NO_COLOR`, `claude-shim statusline <preset>`, the `PATH` requirement and switching with `--force`; `README.md` and `docs/guide/quick-start.md` are read against it. `npm run docs:build` green.

## 5. Integration

- [ ] 5.1 `cargo test`, `cargo clippy --all-targets -- -D warnings` and `mise run e2e` green together; `mise run coverage` shows `src/statusline/mod.rs` covered.

## Workflow follow-up

- `openspec archive add-statusline-presets -y` within the change's one commit on `develop`, subject `feat: add compact, labeled and colored statusline presets`.
- Remove the prototype: `$CLAUDE_CONFIG_DIR/statusline-test.sh` of profile `personal`, and set its preset with `--force`.
