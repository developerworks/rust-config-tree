use std::path::Path;

use schemars::JsonSchema;

use crate::{
    config::{ConfigResult, ConfigSchema},
    config_output::write_template,
};

use super::{
    adapt::{
        remove_empty_object_properties, remove_env_only_properties,
        remove_schema_extensions, schema_for_output_path, prune_unused_schema_maps,
    },
    generate::{root_config_schema, schema_json},
    paths::{schema_path_for_section, split_section_paths},
    target::ConfigSchemaTarget,
};

/// Writes a Draft 7 JSON Schema for the root config type.
///
/// The same generated schema can be referenced from TOML, YAML, JSON, and JSON5
/// configuration files. TOML and YAML templates bind it with editor directives.
/// JSON and JSON5 templates bind it with a top-level `$schema` property.
/// Generated schemas omit JSON Schema `required` constraints so editors provide
/// completion without requiring every config field to exist in each partial
/// config file.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`JsonSchema`].
///
/// # Arguments
///
/// - `output_path`: Destination path for the generated JSON Schema.
///
/// # Returns
///
/// Returns `Ok(())` after the schema file has been written.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, write_config_schema};
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
/// let path = std::env::temp_dir().join("rust-config-tree-write-schema-doctest.json");
/// write_config_schema::<AppConfig>(&path)?;
///
/// assert!(path.exists());
/// # let _ = std::fs::remove_file(path);
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn write_config_schema<S>(output_path: impl AsRef<Path>) -> ConfigResult<()>
where
    S: JsonSchema,
{
    let mut schema = root_config_schema::<S>()?;
    remove_env_only_properties(&mut schema);
    remove_empty_object_properties(&mut schema);
    prune_unused_schema_maps(&mut schema);
    remove_schema_extensions(&mut schema);
    let json = schema_json(&schema)?;

    write_template(output_path.as_ref(), &json)
}

/// Collects the root schema and section schemas for a config type.
///
/// The root schema is written to `output_path`. Nested `confique` sections are
/// written next to it as `<section>.schema.json` when the nested field schema
/// has `x-tree-split = true`; deeper split sections are nested in matching
/// directories, for example `schemas/outer/inner.schema.json`. Each generated
/// schema contains only the fields for its own template file; split child
/// section fields are omitted and completed by their own section schemas.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`JsonSchema`] and exposes section
///   metadata through [`ConfigSchema`].
///
/// # Arguments
///
/// - `output_path`: Destination path for the root JSON Schema.
///
/// # Returns
///
/// Returns all generated schema targets in traversal order.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, config_schema_targets_for_path};
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
/// let targets = config_schema_targets_for_path::<AppConfig>("schemas/config.schema.json")?;
///
/// assert_eq!(targets.len(), 1);
/// assert!(targets[0].content.contains("\"mode\""));
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn config_schema_targets_for_path<S>(
    output_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigSchemaTarget>>
where
    S: ConfigSchema + JsonSchema,
{
    let output_path = output_path.as_ref();
    let full_schema = root_config_schema::<S>()?;
    let split_paths = split_section_paths::<S>(&full_schema);
    let root_schema = schema_for_output_path(&full_schema, &[], &split_paths)?;
    let mut targets = vec![ConfigSchemaTarget {
        path: output_path.to_path_buf(),
        content: schema_json(&root_schema)?,
    }];

    for section_path in &split_paths {
        let schema_path = schema_path_for_section(output_path, section_path);
        let section_schema = schema_for_output_path(&full_schema, section_path, &split_paths)?;

        targets.push(ConfigSchemaTarget {
            path: schema_path,
            content: schema_json(&section_schema)?,
        });
    }

    Ok(targets)
}

/// Writes the root schema and section schemas for a config type.
///
/// Parent directories are created before each schema is written. Generated
/// schemas omit JSON Schema `required` constraints so they can be used for IDE
/// completion against partial config files. The root schema does not complete
/// split nested section fields.
///
/// # Type Parameters
///
/// - `S`: Config schema type that derives [`JsonSchema`] and exposes section
///   metadata through [`ConfigSchema`].
///
/// # Arguments
///
/// - `output_path`: Destination path for the root JSON Schema.
///
/// # Returns
///
/// Returns `Ok(())` after all schema files have been written.
///
/// # Examples
///
/// ```
/// use confique::Config;
/// use rust_config_tree::{ConfigSchema, write_config_schemas};
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
/// let path = std::env::temp_dir()
///     .join("rust-config-tree-write-schemas-doctest")
///     .join("config.schema.json");
/// write_config_schemas::<AppConfig>(&path)?;
///
/// assert!(path.exists());
/// # let _ = std::fs::remove_file(&path);
/// # if let Some(parent) = path.parent() { let _ = std::fs::remove_dir_all(parent); }
/// # Ok::<(), rust_config_tree::ConfigError>(())
/// ```
pub fn write_config_schemas<S>(output_path: impl AsRef<Path>) -> ConfigResult<()>
where
    S: ConfigSchema + JsonSchema,
{
    for target in config_schema_targets_for_path::<S>(output_path)? {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}

