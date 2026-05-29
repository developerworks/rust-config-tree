//! Shared field-path helpers for format-specific template renderers.

use confique::meta::{FieldKind, Meta};

/// Returns whether a leaf field path is marked env-only.
pub(super) fn is_env_only_field(
    current_path: &[&'static str],
    field_name: &'static str,
    env_only_paths: &[Vec<&'static str>],
) -> bool {
    env_only_paths.iter().any(|path| {
        path.len() == current_path.len() + 1
            && path.starts_with(current_path)
            && path.last() == Some(&field_name)
    })
}

/// Returns whether `meta` has any fields that should be rendered.
pub(super) fn has_renderable_fields(
    meta: &'static Meta,
    current_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
    skip_include_field: bool,
) -> bool {
    for field in meta.fields {
        let FieldKind::Leaf { .. } = field.kind else {
            continue;
        };

        if skip_include_field && current_path.is_empty() && field.name == "include" {
            continue;
        }

        if !is_env_only_field(current_path, field.name, env_only_paths) {
            return true;
        }
    }

    for field in meta.fields {
        let FieldKind::Nested { meta } = field.kind else {
            continue;
        };

        let mut child_path = current_path.to_vec();
        child_path.push(field.name);

        let split_exact = split_paths.iter().any(|path| path == &child_path);
        if split_exact {
            continue;
        }

        let split_descendant = split_paths
            .iter()
            .any(|path| path.starts_with(&child_path) && path.len() > child_path.len());
        let child_split_paths = if split_descendant { split_paths } else { &[] };

        if has_renderable_fields(meta, &child_path, child_split_paths, env_only_paths, false) {
            return true;
        }
    }

    false
}
