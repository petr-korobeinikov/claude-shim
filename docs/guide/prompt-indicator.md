# Prompt indicator

Both styles read `CLAUDE_SHIM_ACTIVE_PROFILE`,
which the [shell hook](/guide/installation#shell-integration-zsh) exports —
set that up first.

Pick one of the styles below.

## Plain PS1

```sh
PS1='%n@%m %~ ${CLAUDE_SHIM_ACTIVE_PROFILE:+[$CLAUDE_SHIM_ACTIVE_PROFILE] }%# '
```

## oh-my-posh

Pick a format and inline the segment into your theme:

- **YAML** —
  add as a list item under `segments:` of the target block.
- **TOML** —
  paste right after the `[[blocks]]` table you want the segment to live in;
  `[[blocks.segments]]` attaches to the nearest preceding `[[blocks]]`.
- **JSON** —
  add as an object inside the matching `"segments": [ ... ]` array
  (mind the surrounding commas).

### Minimal

A bare text segment —
no glyphs, no palette entry, no Nerd Font.
Drops into any existing OMP theme as-is.

**YAML**

```yaml
- type: text
  style: plain
  template: "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
```

**TOML**

```toml
[[blocks.segments]]
type = "text"
style = "plain"
template = "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
```

**JSON**

```json
{
  "type": "text",
  "style": "plain",
  "template": "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}[{{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }}] {{ end }}"
}
```

### Powerline diamond

A diamond segment with a powerline cap,
brand colour,
and ✳ glyph.

Requires:

- a Nerd Font for the `\ue0b0` powerline cap
  (✳ is plain Unicode U+2733 and renders with any modern font);
- a `claude` palette entry,
  added to the existing top-level `palette` block of your theme.

Pick your format and apply both pieces —
the palette entry goes inside your existing palette,
the segment goes inside the target `segments` array.

**YAML**

Palette entry, added under your existing `palette:`:

```yaml
claude: "#CC785C"
```

Segment:

```yaml
- type: text
  style: diamond
  leading_diamond: "<transparent,background>\ue0b0</>"
  trailing_diamond: "<background,transparent>\ue0b0</>"
  foreground: p:pure_black
  background: p:claude
  template: "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
```

**TOML**

Palette entry, added under your existing `[palette]` table:

```toml
claude = "#CC785C"
```

Segment:

```toml
[[blocks.segments]]
type = "text"
style = "diamond"
leading_diamond = "<transparent,background>\ue0b0</>"
trailing_diamond = "<background,transparent>\ue0b0</>"
foreground = "p:pure_black"
background = "p:claude"
template = "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
```

**JSON**

Palette entry, added inside your existing `"palette"` object:

```json
"claude": "#CC785C"
```

Segment:

```json
{
  "type": "text",
  "style": "diamond",
  "leading_diamond": "<transparent,background>\ue0b0</>",
  "trailing_diamond": "<background,transparent>\ue0b0</>",
  "foreground": "p:pure_black",
  "background": "p:claude",
  "template": "{{ if .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} ✳ {{ .Env.CLAUDE_SHIM_ACTIVE_PROFILE }} {{ end }}"
}
```
