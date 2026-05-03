//! Mapping generated template paths back to schema section paths.

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use confique::meta::{FieldKind, Meta};

use crate::normalize_lexical;

use crate::config::ConfigSchema;

/// Resolves the split section represented by a generated template target.
pub(super) fn section_path_for_target<S>(
    root_base_dir: &Path,
    target_path: &Path,
    split_paths: &[Vec<&'static str>],
) -> Option<Vec<&'static str>>
where
    S: ConfigSchema,
{
    section_path_for_target_candidates::<S>(root_base_dir, target_path, split_paths)
}

/// Resolves a target path against a caller-provided set of section candidates.
pub(super) fn section_path_for_target_candidates<S>(
    root_base_dir: &Path,
    target_path: &Path,
    candidates: &[Vec<&'static str>],
) -> Option<Vec<&'static str>>
where
    S: ConfigSchema,
{
    let normalized_target = normalize_lexical(target_path);

    for section_path in candidates {
        let section_target =
            normalize_lexical(root_base_dir.join(template_path_for_section::<S>(section_path)));
        if section_target == normalized_target {
            return Some(section_path.clone());
        }
    }

    infer_section_path_from_path(target_path, candidates)
}

/// Returns the generated template path for a split section path.
pub(super) fn template_path_for_section<S>(section_path: &[&str]) -> PathBuf
where
    S: ConfigSchema,
{
    if let Some(path) = S::template_path_for_section(section_path) {
        return path;
    }

    let Some((last, parent_path)) = section_path.split_last() else {
        return PathBuf::new();
    };

    if parent_path.is_empty() {
        return PathBuf::from("config").join(format!("{last}.yaml"));
    }

    let parent_template_path = template_path_for_section::<S>(parent_path);
    parent_template_path
        .with_extension("")
        .join(format!("{last}.yaml"))
}

/// Returns `path` relative to `base` when it is inside that base directory.
pub(super) fn path_relative_to(path: &Path, base: &Path) -> PathBuf {
    match path.strip_prefix(base) {
        Ok(relative) if !relative.as_os_str().is_empty() => relative.to_path_buf(),
        _ => path.to_path_buf(),
    }
}

/// Infers the closest section path from a user-customized template path.
fn infer_section_path_from_path(
    path: &Path,
    candidates: &[Vec<&'static str>],
) -> Option<Vec<&'static str>> {
    let path_tokens = normalized_path_tokens(path);
    let file_token = path
        .file_stem()
        .and_then(OsStr::to_str)
        .map(normalize_token)
        .unwrap_or_default();

    candidates
        .iter()
        .filter_map(|section_path| {
            let score = section_path_score(section_path, &path_tokens, &file_token);
            (score > 0).then_some((score, section_path.clone()))
        })
        .max_by_key(|(score, section_path)| (*score, section_path.len()))
        .map(|(_, section_path)| section_path)
}

/// Normalizes every path component into a comparable section token.
fn normalized_path_tokens(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(|component| {
            Path::new(component)
                .file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or(component)
        })
        .map(normalize_token)
        .filter(|component| !component.is_empty())
        .collect()
}

/// Normalizes a path or section token for fuzzy section matching.
fn normalize_token(token: &str) -> String {
    token
        .chars()
        .filter_map(|character| match character {
            '-' | ' ' => Some('_'),
            '_' => Some('_'),
            character if character.is_ascii_alphanumeric() => Some(character.to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

/// Scores how well a section path matches a template path.
fn section_path_score(section_path: &[&str], path_tokens: &[String], file_token: &str) -> usize {
    let section_tokens = section_path
        .iter()
        .map(|segment| normalize_token(segment))
        .collect::<Vec<_>>();

    if path_tokens.ends_with(&section_tokens) {
        return 1_000 + section_tokens.len();
    }

    let Some(last_section_token) = section_tokens.last() else {
        return 0;
    };

    if file_token == last_section_token {
        return 500 + section_tokens.len();
    }

    if file_token.starts_with(last_section_token) || last_section_token.starts_with(file_token) {
        return 100 + last_section_token.len().min(file_token.len());
    }

    0
}

/// Looks up nested `confique` metadata for a section path.
pub(super) fn meta_at_path(meta: &'static Meta, section_path: &[&str]) -> Option<&'static Meta> {
    let mut current_meta = meta;
    for section in section_path {
        current_meta = current_meta.fields.iter().find_map(|field| {
            if field.name != *section {
                return None;
            }

            match field.kind {
                FieldKind::Nested { meta } => Some(meta),
                FieldKind::Leaf { .. } => None,
            }
        })?;
    }

    Some(current_meta)
}
