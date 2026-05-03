//! JSON Schema generation and section-schema splitting.
//!
//! `schemars` produces one full schema for the root config type. This module
//! removes constraints that do not fit partial config files, strips
//! `x-tree-split` marker metadata from the emitted JSON, and optionally emits
//! separate schemas for marked nested sections.

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use confique::meta::{FieldKind, Meta};
use schemars::{JsonSchema, generate::SchemaSettings};
use serde_json::Value;

use crate::{
    config::{ConfigResult, ConfigSchema},
    config_output::write_template,
    config_util::ensure_single_trailing_newline,
};

const TREE_SPLIT_SCHEMA_EXTENSION: &str = "x-tree-split";

/// Generated JSON Schema content for one output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSchemaTarget {
    /// Path that should receive the generated schema.
    pub path: PathBuf,
    /// Complete JSON Schema content to write to `path`.
    pub content: String,
}

/// Builds the root Draft 7 schema and adapts it for partial config files.
///
/// # Type Parameters
///
/// - `S`: Config schema type to render with `schemars`.
///
/// # Arguments
///
/// This function has no arguments.
///
/// # Returns
///
/// Returns the root schema as JSON with `required` constraints removed.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn root_config_schema<S>() -> ConfigResult<Value>
where
    S: JsonSchema,
{
    let generator = SchemaSettings::draft07().into_generator();
    let schema = generator.into_root_schema_for::<S>();
    let mut schema = serde_json::to_value(schema)?;
    remove_required_recursively(&mut schema);

    Ok(schema)
}

/// Serializes a schema value as stable pretty JSON for generated files.
///
/// # Arguments
///
/// - `schema`: Schema value to serialize.
///
/// # Returns
///
/// Returns pretty JSON with exactly one trailing newline.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn schema_json(schema: &Value) -> ConfigResult<String> {
    let mut json = serde_json::to_string_pretty(schema)?;
    ensure_single_trailing_newline(&mut json);
    Ok(json)
}

/// Removes every JSON Schema `required` list from a schema tree.
///
/// # Arguments
///
/// - `value`: Schema subtree to edit in place.
///
/// # Returns
///
/// Returns no value; `value` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn remove_required_recursively(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.remove("required");

            for (key, child) in object.iter_mut() {
                if is_schema_map_key(key) {
                    // Schema maps contain child schemas keyed by property or
                    // definition name; the map object itself is not a schema.
                    remove_required_from_schema_map(child);
                } else {
                    remove_required_recursively(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                remove_required_recursively(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

/// Returns whether a JSON object key names a map of child schemas.
///
/// # Arguments
///
/// - `key`: JSON object key to classify.
///
/// # Returns
///
/// Returns `true` when `key` names a schema map.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn is_schema_map_key(key: &str) -> bool {
    matches!(
        key,
        "$defs" | "definitions" | "properties" | "patternProperties"
    )
}

/// Removes `required` lists from every schema inside a schema map.
///
/// # Arguments
///
/// - `value`: Schema map value or fallback schema value to edit in place.
///
/// # Returns
///
/// Returns no value; `value` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn remove_required_from_schema_map(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for schema in object.values_mut() {
                remove_required_recursively(schema);
            }
        }
        _ => remove_required_recursively(value),
    }
}

/// Extracts a nested section schema and wraps it as a standalone schema.
///
/// # Arguments
///
/// - `root_schema`: Full root schema used for traversal and reference lookup.
/// - `section_path`: Nested section field path to extract.
///
/// # Returns
///
/// Returns a standalone section schema when the path exists.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn section_schema_for_path(root_schema: &Value, section_path: &[&str]) -> Option<Value> {
    let mut current = root_schema;

    for section in section_path {
        current = current.get("properties")?.get(*section)?;
        current = resolve_schema_reference(root_schema, current).unwrap_or(current);
    }

    Some(standalone_section_schema(root_schema, current))
}

/// Resolves the local schema reference shape emitted by `schemars`.
///
/// # Arguments
///
/// - `root_schema`: Full root schema that owns referenced definitions.
/// - `schema`: Schema value that may contain a local `$ref`.
///
/// # Returns
///
/// Returns the referenced schema when `schema` contains a supported reference.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn resolve_schema_reference<'a>(root_schema: &'a Value, schema: &'a Value) -> Option<&'a Value> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return resolve_json_pointer_ref(root_schema, reference);
    }

    schema
        .get("allOf")
        .and_then(Value::as_array)
        .and_then(|schemas| schemas.first())
        .and_then(|schema| schema.get("$ref"))
        .and_then(Value::as_str)
        .and_then(|reference| resolve_json_pointer_ref(root_schema, reference))
}

/// Resolves a local JSON Pointer `$ref` against the root schema.
///
/// # Arguments
///
/// - `root_schema`: Full schema to query with the JSON Pointer.
/// - `reference`: `$ref` string that must start with `#`.
///
/// # Returns
///
/// Returns the referenced schema value when the pointer resolves.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn resolve_json_pointer_ref<'a>(root_schema: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix('#')?;
    root_schema.pointer(pointer)
}

/// Copies root-level schema metadata needed by an extracted section schema.
///
/// # Arguments
///
/// - `root_schema`: Full root schema that owns `$schema`, `definitions`, and
///   `$defs`.
/// - `section_schema`: Extracted section schema to make standalone.
///
/// # Returns
///
/// Returns a cloned section schema with necessary root metadata attached.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn standalone_section_schema(root_schema: &Value, section_schema: &Value) -> Value {
    let mut section_schema = section_schema.clone();
    let Some(object) = section_schema.as_object_mut() else {
        return section_schema;
    };

    if let Some(schema_uri) = root_schema.get("$schema") {
        object
            .entry("$schema".to_owned())
            .or_insert_with(|| schema_uri.clone());
    }

    if let Some(definitions) = root_schema.get("definitions") {
        object
            .entry("definitions".to_owned())
            .or_insert_with(|| definitions.clone());
    }

    if let Some(defs) = root_schema.get("$defs") {
        object
            .entry("$defs".to_owned())
            .or_insert_with(|| defs.clone());
    }

    section_schema
}

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
pub(crate) fn schema_path_for_section(root_schema_path: &Path, section_path: &[&str]) -> PathBuf {
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

/// Builds the schema content for either the root output or one split section.
///
/// # Arguments
///
/// - `full_schema`: Full root schema generated by `schemars`.
/// - `section_path`: Empty for the root schema, or the split section path.
/// - `split_paths`: All split section paths used to prune child sections.
///
/// # Returns
///
/// Returns the generated schema value for one output file.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn schema_for_output_path(
    full_schema: &Value,
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) -> ConfigResult<Value> {
    let mut schema = if section_path.is_empty() {
        full_schema.clone()
    } else {
        section_schema_for_path(full_schema, section_path).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "failed to extract JSON Schema for config section {}",
                    section_path.join(".")
                ),
            )
        })?
    };

    // Each generated file owns only its direct fields. Split child sections are
    // completed by their own schema files, so remove them from the parent.
    remove_child_section_properties(&mut schema, section_path, split_paths);
    prune_unused_schema_maps(&mut schema);
    remove_tree_split_extensions(&mut schema);

    Ok(schema)
}

/// Removes direct split child sections from the schema owned by this output.
///
/// # Arguments
///
/// - `schema`: Schema value for the current output file.
/// - `section_path`: Section path owned by the current output file.
/// - `split_paths`: All split section paths in the root schema.
///
/// # Returns
///
/// Returns no value; `schema` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn remove_child_section_properties(
    schema: &mut Value,
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) {
    let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) else {
        return;
    };

    for child_section_path in direct_child_split_section_paths(section_path, split_paths) {
        if let Some(child_name) = child_section_path.last() {
            properties.remove(*child_name);
        }
    }
}

/// Drops unused `definitions` and `$defs` entries after section pruning.
///
/// # Arguments
///
/// - `schema`: Schema value whose schema maps should be pruned.
///
/// # Returns
///
/// Returns no value; `schema` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn prune_unused_schema_maps(schema: &mut Value) {
    let mut definitions = BTreeSet::new();
    let mut defs = BTreeSet::new();

    collect_schema_refs(schema, false, &mut definitions, &mut defs);

    loop {
        let previous_len = definitions.len() + defs.len();
        collect_transitive_schema_refs(schema, &mut definitions, &mut defs);

        if definitions.len() + defs.len() == previous_len {
            break;
        }
    }

    retain_schema_map(schema, "definitions", &definitions);
    retain_schema_map(schema, "$defs", &defs);
}

/// Expands the reference set with references used by already retained schemas.
///
/// # Arguments
///
/// - `schema`: Root schema containing schema maps to inspect.
/// - `definitions`: Referenced `definitions` names to expand in place.
/// - `defs`: Referenced `$defs` names to expand in place.
///
/// # Returns
///
/// Returns no value; `definitions` and `defs` are updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_transitive_schema_refs(
    schema: &Value,
    definitions: &mut BTreeSet<String>,
    defs: &mut BTreeSet<String>,
) {
    let current_definitions = definitions.clone();
    let current_defs = defs.clone();
    let mut referenced_definitions = BTreeSet::new();
    let mut referenced_defs = BTreeSet::new();

    if let Some(schema_map) = schema.get("definitions").and_then(Value::as_object) {
        for name in &current_definitions {
            if let Some(schema) = schema_map.get(name) {
                collect_schema_refs(
                    schema,
                    true,
                    &mut referenced_definitions,
                    &mut referenced_defs,
                );
            }
        }
    }

    if let Some(schema_map) = schema.get("$defs").and_then(Value::as_object) {
        for name in &current_defs {
            if let Some(schema) = schema_map.get(name) {
                collect_schema_refs(
                    schema,
                    true,
                    &mut referenced_definitions,
                    &mut referenced_defs,
                );
            }
        }
    }

    definitions.extend(referenced_definitions);
    defs.extend(referenced_defs);
}

/// Walks a schema value and collects local references to schema maps.
///
/// # Arguments
///
/// - `value`: Schema subtree to inspect.
/// - `include_schema_maps`: Whether nested `definitions` and `$defs` maps
///   should also be traversed.
/// - `definitions`: Output set of referenced `definitions` names.
/// - `defs`: Output set of referenced `$defs` names.
///
/// # Returns
///
/// Returns no value; output sets are updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_schema_refs(
    value: &Value,
    include_schema_maps: bool,
    definitions: &mut BTreeSet<String>,
    defs: &mut BTreeSet<String>,
) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                collect_schema_ref(reference, definitions, defs);
            }

            for (key, child) in object {
                if !include_schema_maps && matches!(key.as_str(), "definitions" | "$defs") {
                    continue;
                }

                collect_schema_refs(child, include_schema_maps, definitions, defs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_schema_refs(item, include_schema_maps, definitions, defs);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

/// Records one local `$ref` if it points at `definitions` or `$defs`.
///
/// # Arguments
///
/// - `reference`: `$ref` string to inspect.
/// - `definitions`: Output set of referenced `definitions` names.
/// - `defs`: Output set of referenced `$defs` names.
///
/// # Returns
///
/// Returns no value; matching references are inserted into the output sets.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn collect_schema_ref(
    reference: &str,
    definitions: &mut BTreeSet<String>,
    defs: &mut BTreeSet<String>,
) {
    if let Some(name) = schema_ref_name(reference, "#/definitions/") {
        definitions.insert(name);
    } else if let Some(name) = schema_ref_name(reference, "#/$defs/") {
        defs.insert(name);
    }
}

/// Extracts and JSON-Pointer-decodes a schema-map entry name from a `$ref`.
///
/// # Arguments
///
/// - `reference`: `$ref` string to parse.
/// - `prefix`: Schema-map pointer prefix to strip.
///
/// # Returns
///
/// Returns the decoded schema-map entry name when the reference matches.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn schema_ref_name(reference: &str, prefix: &str) -> Option<String> {
    let name = reference.strip_prefix(prefix)?.split('/').next()?;
    Some(decode_json_pointer_token(name))
}

/// Decodes the escaping used by one JSON Pointer path token.
///
/// # Arguments
///
/// - `token`: Encoded JSON Pointer path token.
///
/// # Returns
///
/// Returns `token` with `~1` and `~0` escapes decoded.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn decode_json_pointer_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// Retains only referenced entries in a root schema map.
///
/// # Arguments
///
/// - `schema`: Root schema containing the schema map.
/// - `key`: Schema-map key, such as `definitions` or `$defs`.
/// - `used_names`: Entry names that should remain in the map.
///
/// # Returns
///
/// Returns no value; the map is pruned in place.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn retain_schema_map(schema: &mut Value, key: &str, used_names: &BTreeSet<String>) {
    let Some(object) = schema.as_object_mut() else {
        return;
    };

    let Some(schema_map) = object.get_mut(key).and_then(Value::as_object_mut) else {
        return;
    };

    schema_map.retain(|name, _| used_names.contains(name));

    if schema_map.is_empty() {
        object.remove(key);
    }
}

/// Removes internal `x-tree-split` markers before writing public schemas.
///
/// # Arguments
///
/// - `value`: Schema subtree to sanitize.
///
/// # Returns
///
/// Returns no value; `value` is updated directly.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn remove_tree_split_extensions(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.remove(TREE_SPLIT_SCHEMA_EXTENSION);

            for child in object.values_mut() {
                remove_tree_split_extensions(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                remove_tree_split_extensions(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

/// Writes a Draft 7 JSON Schema for the root config type.
///
/// The same generated schema can be referenced from TOML, YAML, and JSON
/// configuration files. TOML and YAML templates can bind it with editor
/// directives. JSON files should usually be bound through editor settings
/// rather than a runtime `$schema` field. Generated schemas omit JSON Schema
/// `required` constraints so editors provide completion without requiring every
/// config field to exist in each partial config file.
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
    remove_tree_split_extensions(&mut schema);
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
pub(crate) fn nested_section_paths(meta: &'static Meta) -> Vec<Vec<&'static str>> {
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
pub(crate) fn split_section_paths<S>(full_schema: &Value) -> Vec<Vec<&'static str>>
where
    S: ConfigSchema,
{
    nested_section_paths(&S::META)
        .into_iter()
        .filter(|section_path| section_has_tree_split_marker(full_schema, section_path))
        .collect()
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
    section_property_schema_for_path(root_schema, section_path)
        .and_then(|schema| schema.get(TREE_SPLIT_SCHEMA_EXTENSION))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Returns the original property schema for a nested section path.
///
/// # Arguments
///
/// - `root_schema`: Full root schema to traverse.
/// - `section_path`: Nested section field path to locate.
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
fn section_property_schema_for_path<'a>(
    root_schema: &'a Value,
    section_path: &[&str],
) -> Option<&'a Value> {
    let mut current = root_schema;

    for (index, section) in section_path.iter().enumerate() {
        let property = current.get("properties")?.get(*section)?;
        if index + 1 == section_path.len() {
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
pub(crate) fn direct_child_split_section_paths(
    section_path: &[&'static str],
    split_paths: &[Vec<&'static str>],
) -> Vec<Vec<&'static str>> {
    split_paths
        .iter()
        .filter(|path| path.len() == section_path.len() + 1 && path.starts_with(section_path))
        .cloned()
        .collect()
}
