//! Project root discovery and user-facing path helpers.

use std::path::{Path, PathBuf};

use crate::error::ProjectNotInitializedError;

/// Locate the nearest initialized Specite project from `start` upward.
pub fn find_project_root(start: Option<&Path>) -> Result<PathBuf, ProjectNotInitializedError> {
    let base = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    for candidate in base.ancestors() {
        let specite = candidate.join(".specite");
        if specite.is_dir() || specite.join("iters.json").exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    Err(ProjectNotInitializedError::new(
        "No Specite project found from the current directory. Run `specite init`.",
    ))
}

/// Make a path absolute relative to the current working directory if it is not already absolute.
fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(path),
            Err(_) => path.to_path_buf(),
        }
    }
}

fn current_dir_or_dot() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Display a path relative to the current working directory, falling back to the absolute path.
///
/// Mirrors Python's `display_path(path)` (with no `start` argument).
pub fn display_path(path: &Path) -> String {
    display_path_with_base(path, &current_dir_or_dot())
}

/// Display a path relative to `base`, falling back to the absolute path.
///
/// Mirrors Python's `display_path(path, start)`.
pub fn display_path_with_base(path: &Path, base: &Path) -> String {
    let target = absolutize(path);
    let base = absolutize(base);
    match pathdiff::diff_paths(&target, &base) {
        Some(rel) => rel.to_string_lossy().replace('\\', "/"),
        None => target.to_string_lossy().replace('\\', "/"),
    }
}
