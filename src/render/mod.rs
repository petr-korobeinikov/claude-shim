//! Rendering paths in short, human-facing form for the CLI's stdout/stderr.
//!
//! Every confirmation and error the tool prints routes its paths through
//! [`PathCtx::show`], which yields a [`ShortPath`] whose `Display` applies this
//! precedence:
//!
//! 1. a strict descendant of the current directory → relative to it (no `./`,
//!    and never a `../` escape);
//! 2. otherwise the home directory itself → `~`, or a descendant of it →
//!    `~/<rel>`;
//! 3. otherwise the absolute path.
//!
//! [`PathCtx::show_shell`] yields a [`ShellPath`] with the same precedence but
//! shell-quoted for the copy-paste commands in error hints (`~` stays expandable).
//!
//! Non-UTF-8 components render lossily, exactly as [`std::path::Path::display`]
//! does. Paths written into marker/config files or the shell `init` snippet
//! must stay absolute and deliberately do not go through here.

use std::fmt;
use std::path::{MAIN_SEPARATOR, Path};

/// The anchors a [`ShortPath`] shortens against: the current directory and the
/// home directory. Either may be absent (the environment could not be resolved,
/// or the caller has no cwd) — a missing anchor disables its rule and the path
/// falls through to the next one. `Copy`, so it is passed by value.
#[derive(Copy, Clone)]
pub(crate) struct PathCtx<'a> {
    cwd: Option<&'a Path>,
    home: Option<&'a Path>,
}

impl<'a> PathCtx<'a> {
    /// No anchors: every path renders absolute. Used by tests that assert on the
    /// un-shortened form; production call sites always resolve real anchors.
    #[cfg(test)]
    pub(crate) const EMPTY: PathCtx<'static> = PathCtx {
        cwd: None,
        home: None,
    };

    pub(crate) fn new(cwd: Option<&'a Path>, home: Option<&'a Path>) -> Self {
        Self { cwd, home }
    }

    /// The same anchors with the cwd rule disabled. Used by [`Self::show_shell`],
    /// whose copy-paste commands target fixed locations and so must not be
    /// shortened against the transient current directory.
    fn without_cwd(self) -> Self {
        Self { cwd: None, ..self }
    }

    /// Wrap `path` for short `Display` under these anchors.
    pub(crate) fn show(self, path: &'a Path) -> ShortPath<'a> {
        ShortPath { path, ctx: self }
    }

    /// Wrap `path` for shell-safe `Display` — the copy-paste form used in the
    /// shim's remediation hints. See [`ShellPath`].
    pub(crate) fn show_shell(self, path: &'a Path) -> ShellPath<'a> {
        ShellPath { path, ctx: self }
    }
}

/// A path paired with the anchors to shorten it against; see the module docs for
/// the precedence. Produced by [`PathCtx::show`] and only ever used inline in a
/// format string, so it borrows rather than owning anything.
pub(crate) struct ShortPath<'a> {
    path: &'a Path,
    ctx: PathCtx<'a>,
}

/// The same shortened path as [`ShortPath`], rendered as a single shell word for
/// the copy-paste commands in the shim's remediation hints: the non-`~` part is
/// single-quoted so a space (macOS `~/Library/Application Support/…`) or any
/// other metacharacter survives being pasted verbatim, while a leading `~` stays
/// outside the quotes so the shell still expands it. Anchored to `~`/absolute
/// only — never the transient cwd — since these commands target fixed, global
/// locations that must resolve the same wherever they are pasted. Produced by
/// [`PathCtx::show_shell`].
pub(crate) struct ShellPath<'a> {
    path: &'a Path,
    ctx: PathCtx<'a>,
}

/// The shortened form a path takes under the anchors, chosen once so the plain
/// and shell-quoted renderings share the precedence documented at module level.
enum Form<'a> {
    /// Rendered as-is: a strict descendant of the current directory (relative to
    /// it) or a path under neither anchor (absolute). The shell form quotes it.
    Plain(&'a Path),
    /// The home directory itself → `~`.
    Home,
    /// A descendant of home → `~` followed by this remainder.
    HomeRelative(&'a Path),
}

fn classify<'a>(path: &'a Path, ctx: PathCtx<'a>) -> Form<'a> {
    if let Some(cwd) = ctx.cwd
        && let Ok(rel) = path.strip_prefix(cwd)
        && !rel.as_os_str().is_empty()
    {
        return Form::Plain(rel);
    }
    if let Some(home) = ctx.home
        && let Ok(rel) = path.strip_prefix(home)
    {
        return if rel.as_os_str().is_empty() {
            Form::Home
        } else {
            Form::HomeRelative(rel)
        };
    }
    Form::Plain(path)
}

impl fmt::Display for ShortPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match classify(self.path, self.ctx) {
            Form::Plain(p) => write!(f, "{}", p.display()),
            Form::Home => write!(f, "~"),
            Form::HomeRelative(rel) => write!(f, "~{MAIN_SEPARATOR}{}", rel.display()),
        }
    }
}

impl fmt::Display for ShellPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match classify(self.path, self.ctx.without_cwd()) {
            Form::Plain(p) => write!(f, "{}", quote_path(p)),
            Form::Home => write!(f, "~"),
            Form::HomeRelative(rel) => write!(f, "~{MAIN_SEPARATOR}{}", quote_path(rel)),
        }
    }
}

/// The shell-quoted form of `p`'s lossy string — the per-segment quoting used by
/// [`ShellPath`].
fn quote_path(p: &Path) -> String {
    shell_quote(&p.to_string_lossy())
}

/// Quote `s` as a single POSIX shell word: wrap it in single quotes and escape
/// each embedded single quote as `'\''`. The crate's one shell-quoting
/// primitive — used here for path hints and by the `init` snippet — so a value
/// survives spaces and every shell metacharacter when pasted or evaluated.
pub(crate) fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str(r"'\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests;
