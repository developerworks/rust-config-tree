//! Lexical path normalization and include path resolution.
//!
//! These helpers normalize paths without consulting the file system. They are
//! used by both tree loading and template target collection.

use std::path::{Component, Path, PathBuf};

use crate::{ConfigTreeError, Result};

/// Converts a path to an absolute path and normalizes it lexically.
///
/// The path does not need to exist. `.` and `..` components are simplified
/// without resolving symbolic links.
///
/// # Arguments
///
/// - `path`: Path to convert to an absolute normalized path.
///
/// # Returns
///
/// Returns the normalized absolute path.
pub fn absolutize_lexical(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| ConfigTreeError::CurrentDir { source })?
            .join(path)
    };

    Ok(normalize_lexical(path))
}

/// Resolves an include path relative to the file that declared it.
///
/// Absolute include paths are only normalized. Relative include paths are joined
/// to the parent directory of `parent_path` and then normalized.
///
/// # Arguments
///
/// - `parent_path`: Path of the config file that declared the include.
/// - `include_path`: Include path declared by `parent_path`.
///
/// # Returns
///
/// Returns the normalized resolved include path.
pub fn resolve_include_path(
    parent_path: impl AsRef<Path>,
    include_path: impl AsRef<Path>,
) -> PathBuf {
    let parent_path = parent_path.as_ref();
    let include_path = include_path.as_ref();

    if include_path.is_absolute() {
        return normalize_lexical(include_path);
    }

    let base_dir = parent_path.parent().unwrap_or_else(|| Path::new("."));
    normalize_lexical(base_dir.join(include_path))
}

/// Normalizes a path by removing lexical `.` and `..` components.
///
/// This function does not touch the file system and does not resolve symbolic
/// links.
///
/// # Arguments
///
/// - `path`: Path to normalize.
///
/// # Returns
///
/// Returns `path` with lexical current-directory and parent-directory
/// components simplified.
pub fn normalize_lexical(path: impl AsRef<Path>) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.as_ref().components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

#[cfg(test)]
#[path = "unit_tests/path.rs"]
mod unit_tests;
