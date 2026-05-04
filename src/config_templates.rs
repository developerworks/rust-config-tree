//! Config template rendering and split-template target planning.
//!
//! The public entry points collect the template tree, preserve existing include
//! structure when regenerating templates, append schema-declared default child
//! includes for newly split sections, and optionally bind TOML/YAML output to
//! generated JSON Schemas.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;

use crate::{
    absolutize_lexical, collect_template_targets,
    config::{ConfigResult, ConfigSchema},
    config_output::write_template,
    config_schema::{
        env_only_field_paths, nested_section_paths, root_config_schema, split_section_paths,
    },
    select_template_source,
};

mod binding;
mod includes;
mod render;
mod section;
mod target;
mod yaml;

use binding::{schema_path_for_template_target, template_with_schema_directive};
use includes::{
    append_missing_include_paths, default_child_include_paths, retain_split_include_paths,
    template_source_include_paths,
};
use render::template_for_target;
use section::section_path_for_target;

pub use render::template_for_path;
pub use target::ConfigTemplateTarget;

/// Collects all template targets that should be generated for a config tree.
///
/// The root template source is selected with [`select_template_source`]. Include
/// paths found in the source tree are mirrored under `output_path` for relative
/// includes. Nested `confique` sections marked with `x-tree-split = true` are
/// used to derive child template files with paths from
/// [`ConfigSchema::template_path_for_section`].
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
///
/// # Returns
///
/// Returns all generated template targets in traversal order.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, template_targets_for_paths};
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
/// let targets =
///     template_targets_for_paths::<AppConfig>("config.yaml", "config.example.yaml")?;
///
/// assert_eq!(targets.len(), 1);
/// assert!(targets[0].content.contains("mode"));
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn template_targets_for_paths<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigTemplateTarget>>
where
    S: ConfigSchema + JsonSchema,
{
    let output_path = output_path.as_ref();
    let source_path = select_template_source(config_path, output_path);
    let root_source_path = absolutize_lexical(source_path)?;
    let output_base_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    let full_schema = root_config_schema::<S>()?;
    let all_section_paths = nested_section_paths(&S::META);
    let split_paths = split_section_paths::<S>(&full_schema);
    let env_only_paths = env_only_field_paths::<S>(&full_schema);

    // First collect from the source tree. Existing include entries are kept
    // when they target split sections, and missing schema-derived child
    // includes are appended so regeneration picks up newly marked sections.
    let template_targets = collect_template_targets(
        &root_source_path,
        output_path,
        |node_source_path| -> ConfigResult<Vec<PathBuf>> {
            let include_paths = template_source_include_paths::<S>(node_source_path)?;
            let mut include_paths = retain_split_include_paths::<S>(
                &root_source_path,
                node_source_path,
                include_paths,
                &all_section_paths,
                &split_paths,
            );
            append_missing_include_paths(
                &mut include_paths,
                default_child_include_paths::<S>(&root_source_path, node_source_path, &split_paths),
            );

            Ok(include_paths)
        },
    )?;

    // Then narrow the split paths to sections that actually produced template
    // targets. Rendering uses this bounded set so stale source includes do not
    // force unrelated nested sections out of the root template.
    let split_paths = template_targets
        .iter()
        .filter_map(|target| {
            section_path_for_target::<S>(output_base_dir, target.target_path(), &split_paths)
                .filter(|section_path| !section_path.is_empty())
        })
        .collect::<Vec<_>>();

    template_targets
        .into_iter()
        .map(|target| {
            let (_, target_path, include_paths) = target.into_parts();
            let section_path =
                section_path_for_target::<S>(output_base_dir, &target_path, &split_paths)
                    .unwrap_or_default();
            Ok(ConfigTemplateTarget {
                content: template_for_target::<S>(
                    &target_path,
                    &include_paths,
                    &section_path,
                    &split_paths,
                    &env_only_paths,
                )?,
                path: target_path,
            })
        })
        .collect()
}

/// Collects template targets and binds TOML/YAML templates to JSON Schemas.
///
/// TOML targets receive a `#:schema` directive. YAML targets receive a YAML
/// Language Server modeline. JSON and JSON5 targets are left unchanged so the
/// runtime configuration is not polluted with a `$schema` field. Root targets
/// bind `schema_path`; nested section targets bind their generated section
/// schema path.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
/// - `schema_path`: Root JSON Schema path to reference from root TOML/YAML
///   templates.
///
/// # Returns
///
/// Returns all generated template targets in traversal order.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, template_targets_for_paths_with_schema};
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
/// let targets = template_targets_for_paths_with_schema::<AppConfig>(
///     "config.yaml",
///     "config.example.yaml",
///     "schemas/config.schema.json",
/// )?;
///
/// assert!(targets[0].content.starts_with("# yaml-language-server: $schema="));
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn template_targets_for_paths_with_schema<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigTemplateTarget>>
where
    S: ConfigSchema + JsonSchema,
{
    let output_path = output_path.as_ref();
    let output_base_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    let schema_path = schema_path.as_ref();
    let full_schema = root_config_schema::<S>()?;
    let split_paths = split_section_paths::<S>(&full_schema);

    template_targets_for_paths::<S>(config_path, output_path)?
        .into_iter()
        .map(|mut target| {
            let schema_path = schema_path_for_template_target::<S>(
                output_base_dir,
                &target.path,
                schema_path,
                &split_paths,
            );
            target.content =
                template_with_schema_directive(&target.path, &schema_path, &target.content)?;
            Ok(target)
        })
        .collect()
}

/// Writes all generated config templates for a config tree.
///
/// Parent directories are created before each target is written.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
///
/// # Returns
///
/// Returns `Ok(())` after all template files have been written.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, write_config_templates};
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
/// let output = std::env::temp_dir().join("rust-config-tree-template-doctest.yaml");
/// write_config_templates::<AppConfig>("config.yaml", &output)?;
///
/// assert!(output.exists());
/// # let _ = std::fs::remove_file(output);
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn write_config_templates<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<()>
where
    S: ConfigSchema + JsonSchema,
{
    for target in template_targets_for_paths::<S>(config_path, output_path)? {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}

/// Writes all generated config templates with editor schema bindings.
///
/// TOML targets receive `#:schema <path>`, YAML targets receive
/// `# yaml-language-server: $schema=<path>`, and JSON targets are left
/// unchanged. The schema path is rendered relative to each template file. Root
/// targets bind `schema_path`; nested section targets bind their generated
/// section schema path.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to discover includes and render templates.
///
/// # Arguments
///
/// - `config_path`: Root config path preferred as the template source when it
///   exists.
/// - `output_path`: Root output path for generated templates.
/// - `schema_path`: Root JSON Schema path to reference from root TOML/YAML
///   templates.
///
/// # Returns
///
/// Returns `Ok(())` after all template files have been written.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, write_config_templates_with_schema};
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
/// let output = std::env::temp_dir().join("rust-config-tree-template-schema-doctest.yaml");
/// write_config_templates_with_schema::<AppConfig>(
///     "config.yaml",
///     &output,
///     "schemas/config.schema.json",
/// )?;
///
/// let content = std::fs::read_to_string(&output)?;
/// assert!(content.starts_with("# yaml-language-server: $schema="));
/// # let _ = std::fs::remove_file(output);
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
pub fn write_config_templates_with_schema<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> ConfigResult<()>
where
    S: ConfigSchema + JsonSchema,
{
    for target in
        template_targets_for_paths_with_schema::<S>(config_path, output_path, schema_path)?
    {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}
