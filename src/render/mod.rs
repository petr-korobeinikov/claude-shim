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

    /// Wrap `path` for short `Display` under these anchors.
    pub(crate) fn show(self, path: &'a Path) -> ShortPath<'a> {
        ShortPath { path, ctx: self }
    }
}

/// A path paired with the anchors to shorten it against; see the module docs for
/// the precedence. Produced by [`PathCtx::show`] and only ever used inline in a
/// format string, so it borrows rather than owning anything.
pub(crate) struct ShortPath<'a> {
    path: &'a Path,
    ctx: PathCtx<'a>,
}

impl fmt::Display for ShortPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(cwd) = self.ctx.cwd
            && let Ok(rel) = self.path.strip_prefix(cwd)
            && !rel.as_os_str().is_empty()
        {
            return write!(f, "{}", rel.display());
        }
        if let Some(home) = self.ctx.home
            && let Ok(rel) = self.path.strip_prefix(home)
        {
            return if rel.as_os_str().is_empty() {
                write!(f, "~")
            } else {
                write!(f, "~{MAIN_SEPARATOR}{}", rel.display())
            };
        }
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
mod tests;
