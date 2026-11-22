//! Where things are on disk. Two roots: the engine's (`ESIM_ROOT`, the current directory by
//! default), which holds the catalog, the cache and the shared tables under `data/events`; and,
//! when a project is being run, the project's own folder (`ESIM_PROJECT`, set by the command
//! line). A project folder may carry `data/events/<name>` of its own, and a name found there is
//! taken from there, so a scenario kept outside the engine's tree brings its record with it.

use std::path::{Path, PathBuf};

/// The engine's root: `ESIM_ROOT`, or the current directory.
pub fn root() -> PathBuf { std::env::var("ESIM_ROOT").ok().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")) }

/// The folder of the project being run, when there is one.
pub fn project() -> Option<PathBuf> { std::env::var("ESIM_PROJECT").ok().filter(|s| !s.is_empty()).map(PathBuf::from) }

/// A table or folder under `data/events`: the project's when its first path segment exists
/// there, the engine's otherwise. The first segment decides, so a record folder is read whole
/// from one place and never half from each.
pub fn event(rel: impl AsRef<Path>) -> PathBuf {
    let rel = rel.as_ref();
    if let (Some(p), Some(first)) = (project(), rel.components().next()) {
        let base = p.join("data/events");
        if base.join(first).exists() { return base.join(rel); }
    }
    root().join("data/events").join(rel)
}
