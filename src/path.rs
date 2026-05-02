use std::path::{Component, Path, PathBuf};

use crate::{ConfigTreeError, Result};

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
