//! Format-specific template rendering and include block injection.

use std::path::{Path, PathBuf};

use confique::meta::Meta;
use schemars::JsonSchema;

use super::{
    json5::render_json5_template,
    toml::render_toml_template,
    yaml::render_yaml_template,
};
use crate::{
    config::{ConfigResult, ConfigSchema},
    config_format::ConfigFormat,
    config_schema::{
        generate::root_config_schema,
        paths::env_only_field_paths,
    },
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
/// use rust_config_tree::config::{ConfigSchema, template_for_path};
/// use schemars::JsonSchema;
///
/// #[derive(Config, JsonSchema)]
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
/// # Ok::<(), rust_config_tree::error::ConfigError>(())
/// ```
pub fn template_for_path<S>(path: impl AsRef<Path>) -> ConfigResult<String>
where
    S: ConfigSchema + JsonSchema,
{
    let full_schema = root_config_schema::<S>()?;
    let env_only_paths = env_only_field_paths::<S>(&full_schema);

    Ok(render_template(
        ConfigFormat::from_path(path.as_ref()),
        &S::META,
        &[],
        &[],
        &[],
        &env_only_paths,
    ))
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
    Ok(render_template(
        ConfigFormat::from_path(path),
        &S::META,
        include_paths,
        section_path,
        split_paths,
        env_only_paths,
    ))
}

fn render_template(
    format: ConfigFormat,
    meta: &'static Meta,
    include_paths: &[PathBuf],
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
    env_only_paths: &[Vec<&'static str>],
) -> String {
    match format {
        ConfigFormat::Yaml => render_yaml_template(
            meta,
            include_paths,
            section_path,
            split_paths,
            env_only_paths,
        ),
        ConfigFormat::Toml => render_toml_template(
            meta,
            include_paths,
            section_path,
            split_paths,
            env_only_paths,
        ),
        ConfigFormat::Json => render_json5_template(
            meta,
            include_paths,
            section_path,
            split_paths,
            env_only_paths,
        ),
    }
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
pub(super) fn render_toml_include(paths: &[PathBuf]) -> String {
    let entries = paths
        .iter()
        .map(|path| quote_path(path))
        .collect::<Vec<_>>()
        .join(", ");
    format!("include = [{entries}]\n")
}

/// Renders a JSON5 top-level include list.
pub(super) fn render_json5_include(paths: &[PathBuf]) -> String {
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
