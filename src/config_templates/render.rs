//! Format-specific template rendering and include block injection.

use std::path::{Path, PathBuf};

use super::yaml::render_yaml_template;
use crate::{
    config::{ConfigResult, ConfigSchema},
    config_format::{ConfigFormat, json5_options, toml_options, yaml_options},
};

/// Renders the default template for one path.
///
/// The template format is inferred from the path extension.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to render the template.
///
/// # Arguments
///
/// - `path`: Output path whose extension selects the template format.
///
/// # Returns
///
/// Returns the generated template content.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, template_for_path};
///
/// #[derive(Config)]
/// struct AppConfig {
///     #[config(default = [])]
///     include: Vec<std::path::PathBuf>,
///     #[config(default = "demo")]
///     mode: String,
/// }
///
/// impl ConfigSchema for AppConfig {
///     fn include_paths(layer: &<Self as Config>::Layer) -> Vec<std::path::PathBuf> {
///         layer.include.clone().unwrap_or_default()
///     }
/// }
///
/// let template = template_for_path::<AppConfig>("config.yaml")?;
///
/// assert!(template.contains("mode"));
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn template_for_path<S>(path: impl AsRef<Path>) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    let template = match ConfigFormat::from_path(path.as_ref()) {
        ConfigFormat::Yaml => confique::yaml::template::<S>(yaml_options()),
        ConfigFormat::Toml => confique::toml::template::<S>(toml_options()),
        ConfigFormat::Json => confique::json5::template::<S>(json5_options()),
    };

    Ok(template)
}

/// Renders the template content for one collected template target.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to render fields.
///
/// # Arguments
///
/// - `path`: Target template path whose extension selects the renderer.
/// - `include_paths`: Include paths to place in the generated template.
/// - `section_path`: Section path represented by this target.
/// - `split_paths`: Section paths split out of the root template.
/// - `env_only_paths`: Leaf field paths omitted from generated config files.
///
/// # Returns
///
/// Returns rendered template content for the target.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn template_for_target<S>(
    path: &Path,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    if ConfigFormat::from_path(path) != ConfigFormat::Yaml
        || (split_paths.is_empty() && env_only_paths.is_empty())
    {
        return template_for_path_with_includes::<S>(path, include_paths);
    }

    Ok(render_yaml_template(
        &S::META,
        include_paths,
        section_path,
        split_paths,
        env_only_paths,
    ))
}

/// Renders a format-specific template and injects an explicit include block.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to render fields.
///
/// # Arguments
///
/// - `path`: Template path whose extension selects the renderer.
/// - `include_paths`: Include paths to inject.
///
/// # Returns
///
/// Returns rendered template content with include paths inserted when present.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn template_for_path_with_includes<S>(
    path: &Path,
    include_paths: &[PathBuf],
) -> ConfigResult<String>
where
    S: ConfigSchema,
{
    let template = template_for_path::<S>(path)?;
    if include_paths.is_empty() {
        return Ok(template);
    }

    let template = match ConfigFormat::from_path(path) {
        ConfigFormat::Yaml => {
            let template = strip_prefix_once(&template, "# Default value: []\n#include: []\n\n");
            format!("{}\n{template}", render_yaml_include(include_paths))
        }
        ConfigFormat::Toml => {
            let template = strip_prefix_once(&template, "# Default value: []\n#include = []\n\n");
            format!("{}\n{template}", render_toml_include(include_paths))
        }
        ConfigFormat::Json => {
            let body = template.strip_prefix("{\n").unwrap_or(&template);
            let body = strip_prefix_once(body, "  // Default value: []\n  //include: [],\n\n");
            format!("{{\n{}\n{body}", render_json5_include(include_paths))
        }
    };

    Ok(template)
}

/// Renders a YAML top-level include list.
///
/// # Arguments
///
/// - `paths`: Include paths to render.
///
/// # Returns
///
/// Returns a YAML `include` block.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn render_yaml_include(paths: &[PathBuf]) -> String {
    let mut out = String::from("include:\n");
    for path in paths {
        out.push_str("  - ");
        out.push_str(&quote_path(path));
        out.push('\n');
    }
    out
}

/// Renders a TOML top-level include list.
///
/// # Arguments
///
/// - `paths`: Include paths to render.
///
/// # Returns
///
/// Returns a TOML `include` assignment.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_toml_include(paths: &[PathBuf]) -> String {
    let entries = paths
        .iter()
        .map(|path| quote_path(path))
        .collect::<Vec<_>>()
        .join(", ");
    format!("include = [{entries}]\n")
}

/// Renders a JSON5 top-level include list.
///
/// # Arguments
///
/// - `paths`: Include paths to render.
///
/// # Returns
///
/// Returns a JSON5 `include` property block.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_json5_include(paths: &[PathBuf]) -> String {
    let mut out = String::from("  include: [\n");
    for path in paths {
        out.push_str("    ");
        out.push_str(&quote_path(path));
        out.push_str(",\n");
    }
    out.push_str("  ],\n");
    out
}

/// Quotes a path using JSON string escaping, which is valid for all outputs.
///
/// # Arguments
///
/// - `path`: Path to render as a quoted string.
///
/// # Returns
///
/// Returns a JSON-escaped string representation of `path`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn quote_path(path: &Path) -> String {
    serde_json::to_string(&path.to_string_lossy()).expect("path string serialization cannot fail")
}

/// Removes one generated default include block when present.
///
/// # Arguments
///
/// - `value`: Text that may start with `prefix`.
/// - `prefix`: Prefix to remove at most once.
///
/// # Returns
///
/// Returns `value` without `prefix` when it was present.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn strip_prefix_once<'a>(value: &'a str, prefix: &str) -> &'a str {
    value.strip_prefix(prefix).unwrap_or(value)
}
