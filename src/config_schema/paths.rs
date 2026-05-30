//! Section and field path discovery from `confique` metadata and schema markers.

use std::path::{Path, PathBuf};

use confique::meta::{FieldKind, Meta};
use serde_json::Value;

use crate::config::ConfigSchema;

use super::{
    marker::{ENV_ONLY_SCHEMA_EXTENSION, TREE_SPLIT_SCHEMA_EXTENSION},
    reference::resolve_schema_reference,
};

/// Resolves the output path for a split section schema.
///
/// # Arguments
///
/// - `root_schema_path`: Output path for the root schema.
/// - `section_path`: Nested section field path.
///
/// # Returns
///
/// Returns the generated schema path for `section_path`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub fn schema_path_for_section(root_schema_path: &Path, section_path: &[&str]) -> PathBuf {
    let Some((last, parents)) = section_path.split_last() else {
        return root_schema_path.to_path_buf();
    };

    let mut path = root_schema_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    for parent in parents {
        path.push(*parent);
    }

    path.push(format!("{}.schema.json", *last));
    path
}
/// Collects every nested `confique` section path from schema metadata.
///
/// # Arguments
///
/// - `meta`: Root `confique` metadata to traverse.
///
/// # Returns
///
/// Returns nested section paths in metadata traversal order.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub fn nested_section_paths(meta: &'static Meta) -> Vec<Vec<&'static str>> {
    let mut paths = Vec::new();
    collect_nested_section_paths(meta, &mut Vec::new(), &mut paths);
    paths
}

/// Finds nested sections whose field schema opts into template/schema splitting.
///
/// # Type Parameters
///
/// - `S`: Config schema type whose metadata supplies nested section paths.
///
/// # Arguments
///
/// - `full_schema`: Full root schema containing `x-tree-split` markers.
///
/// # Returns
///
/// Returns nested section paths that should be split.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub fn split_section_paths<S>(full_schema: &Value) -> Vec<Vec<&'static str>>
where
    S: ConfigSchema,
{
    nested_section_paths(&S::META)
        .into_iter()
        .filter(|section_path| section_has_tree_split_marker(full_schema, section_path))
        .collect()
}

/// Finds leaf fields whose schema opts out of template and schema output.
///
/// # Type Parameters
///
/// - `S`: Config schema type whose metadata supplies field paths.
///
/// # Arguments
///
/// - `full_schema`: Full root schema containing `x-env-only` markers.
///
/// # Returns
///
/// Returns leaf field paths marked with `x-env-only = true`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub fn env_only_field_paths<S>(full_schema: &Value) -> Vec<Vec<&'static str>>
where
    S: ConfigSchema,
{
    let mut paths = Vec::new();
    collect_env_only_field_paths(&S::META, full_schema, &mut Vec::new(), &mut paths);
    paths
}

/// Checks whether a section property carries the split marker extension.
///
/// # Arguments
///
/// - `root_schema`: Full root schema to inspect.
/// - `section_path`: Nested section field path to check.
///
/// # Returns
///
/// Returns `true` when the section schema carries `x-tree-split = true`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn section_has_tree_split_marker(root_schema: &Value, section_path: &[&str]) -> bool {
    property_schema_for_path(root_schema, section_path)
        .and_then(|schema| schema.get(TREE_SPLIT_SCHEMA_EXTENSION))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Checks whether a field property carries the env-only marker extension.
///
/// # Arguments
///
/// - `root_schema`: Full root schema to inspect.
/// - `field_path`: Field path to check.
///
/// # Returns
///
/// Returns `true` when the field schema carries `x-env-only = true`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn field_has_env_only_marker(root_schema: &Value, field_path: &[&str]) -> bool {
    property_schema_for_path(root_schema, field_path)
        .and_then(|schema| schema.get(ENV_ONLY_SCHEMA_EXTENSION))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Returns the original property schema for a field path.
///
/// # Arguments
///
/// - `root_schema`: Full root schema to traverse.
/// - `path`: Field path to locate.
///
/// # Returns
///
/// Returns the original property schema when the section path exists.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn property_schema_for_path<'a>(root_schema: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = root_schema;

    for (index, section) in path.iter().enumerate() {
        let property = current.get("properties")?.get(*section)?;
        if index + 1 == path.len() {
            return Some(property);
        }

        current = resolve_schema_reference(root_schema, property).unwrap_or(property);
    }

    None
}

/// Recursively appends nested section paths to `paths`.
///
/// # Arguments
///
/// - `meta`: Current `confique` metadata node.
/// - `prefix`: Mutable section path prefix for `meta`.
/// - `paths`: Output list receiving discovered nested section paths.
///
/// # Returns
///
/// Returns no value; `paths` and `prefix` are updated during traversal.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_nested_section_paths(
    meta: &'static Meta,
    prefix: &mut Vec<&'static str>,
    paths: &mut Vec<Vec<&'static str>>,
) {
    for field in meta.fields {
        if let FieldKind::Nested { meta } = field.kind {
            prefix.push(field.name);
            paths.push(prefix.clone());
            collect_nested_section_paths(meta, prefix, paths);
            prefix.pop();
        }
    }
}

/// Recursively appends env-only leaf field paths to `paths`.
///
/// # Arguments
///
/// - `meta`: Current `confique` metadata node.
/// - `root_schema`: Full root schema containing marker extensions.
/// - `prefix`: Mutable field path prefix for `meta`.
/// - `paths`: Output list receiving discovered leaf paths.
///
/// # Returns
///
/// Returns no value; `paths` and `prefix` are updated during traversal.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_env_only_field_paths(
    meta: &'static Meta,
    root_schema: &Value,
    prefix: &mut Vec<&'static str>,
    paths: &mut Vec<Vec<&'static str>>,
) {
    for field in meta.fields {
        prefix.push(field.name);

        match field.kind {
            FieldKind::Leaf { .. } => {
                if field_has_env_only_marker(root_schema, prefix) {
                    paths.push(prefix.clone());
                }
            }
            FieldKind::Nested { meta } => {
                collect_env_only_field_paths(meta, root_schema, prefix, paths);
            }
        }

        prefix.pop();
    }
}

/// Returns split sections that are direct children of `section_path`.
///
/// # Arguments
///
/// - `section_path`: Parent section path to match.
/// - `split_paths`: All split section paths.
///
/// # Returns
///
/// Returns split paths whose parent is exactly `section_path`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub fn direct_child_split_section_paths(
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) -> Vec<Vec<&'static str>> {
    split_paths
        .iter()
        .filter(|path| path.len() == section_path.len() + 1 && path.starts_with(section_path))
        .cloned()
        .collect()
}
