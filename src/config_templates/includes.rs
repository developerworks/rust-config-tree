//! Include discovery and schema-derived include planning for template targets.

use std::path::{Path, PathBuf};

use crate::normalize_lexical;

use super::section::{
    path_relative_to, section_path_for_target, section_path_for_target_candidates,
    template_path_for_section,
};
use crate::{
    config::{ConfigResult, ConfigSchema},
    config_load::{figment_for_file, load_layer},
    config_schema::direct_child_split_section_paths,
};

/// Reads include paths from an existing template source when possible.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to parse the source layer.
///
/// # Arguments
///
/// - `path`: Existing or planned template source path.
///
/// # Returns
///
/// Returns include paths read from `path`, or an empty list when it is missing.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn template_source_include_paths<S>(path: &Path) -> ConfigResult<Vec<PathBuf>>
where
    S: ConfigSchema,
{
    if !path.exists() {
        return Ok(Vec::new());
    }

    match load_layer::<S>(path) {
        Ok(layer) => Ok(S::include_paths(&layer)),
        Err(_) => load_include_paths_only(path),
    }
}

/// Falls back to reading only the top-level include list from a partial file.
///
/// # Arguments
///
/// - `path`: Partial template file to inspect.
///
/// # Returns
///
/// Returns the top-level include list when present.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn load_include_paths_only(path: &Path) -> ConfigResult<Vec<PathBuf>> {
    // Existing template files can be partial or contain commented schema
    // defaults. If the full schema layer cannot load, still preserve an
    // explicit top-level `include` list when one exists.
    match figment_for_file(path).extract_inner::<Vec<PathBuf>>("include") {
        Ok(paths) => Ok(paths),
        Err(error) if error.missing() => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

/// Computes schema-derived child include paths for a split section template.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to map section paths to template paths.
///
/// # Arguments
///
/// - `root_source_path`: Root template source path for path mapping.
/// - `node_source_path`: Current template source path.
/// - `split_paths`: All split section paths.
///
/// # Returns
///
/// Returns default include paths for direct split child sections.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn default_child_include_paths<S>(
    root_source_path: &Path,
    node_source_path: &Path,
    split_paths: &[Vec<&'static str>],
) -> Vec<PathBuf>
where
    S: ConfigSchema,
{
    let root_base_dir = root_source_path.parent().unwrap_or_else(|| Path::new("."));
    let section_path = section_path_for_target::<S>(root_base_dir, node_source_path, split_paths)
        .unwrap_or_default();
    let source_base_dir = node_source_path.parent().unwrap_or_else(|| Path::new("."));

    direct_child_split_section_paths(&section_path, split_paths)
        .into_iter()
        .map(|child_section_path| {
            let child_path =
                root_base_dir.join(template_path_for_section::<S>(&child_section_path));
            path_relative_to(&child_path, source_base_dir)
        })
        .collect()
}

/// Keeps source includes that still point at generated split sections.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to map include targets to sections.
///
/// # Arguments
///
/// - `root_source_path`: Root template source path for path mapping.
/// - `node_source_path`: Current template source path.
/// - `include_paths`: Source include paths to filter.
/// - `all_section_paths`: All nested section paths known from metadata.
/// - `split_paths`: Section paths that are actually split into templates.
///
/// # Returns
///
/// Returns include paths that still refer to split sections or unknown files.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn retain_split_include_paths<S>(
    root_source_path: &Path,
    node_source_path: &Path,
    include_paths: Vec<PathBuf>,
    all_section_paths: &[Vec<&'static str>],
    split_paths: &[Vec<&'static str>],
) -> Vec<PathBuf>
where
    S: ConfigSchema,
{
    let root_base_dir = root_source_path.parent().unwrap_or_else(|| Path::new("."));
    let source_base_dir = node_source_path.parent().unwrap_or_else(|| Path::new("."));

    include_paths
        .into_iter()
        .filter(|include_path| {
            let target_path = if include_path.is_absolute() {
                include_path.clone()
            } else {
                source_base_dir.join(include_path)
            };
            let target_path = normalize_lexical(target_path);

            match section_path_for_target_candidates::<S>(
                root_base_dir,
                &target_path,
                all_section_paths,
            ) {
                Some(section_path) => split_paths.contains(&section_path),
                None => true,
            }
        })
        .collect()
}

/// Appends default include paths without duplicating existing source entries.
///
/// # Arguments
///
/// - `include_paths`: Include list to update.
/// - `defaults`: Default include paths to append when missing.
///
/// # Returns
///
/// Returns no value; `include_paths` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn append_missing_include_paths(
    include_paths: &mut Vec<PathBuf>,
    defaults: Vec<PathBuf>,
) {
    for default_path in defaults {
        if !include_paths.contains(&default_path) {
            include_paths.push(default_path);
        }
    }
}
