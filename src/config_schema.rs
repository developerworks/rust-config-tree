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
fn schema_json(schema: &Value) -> ConfigResult<String> {
    let mut json = serde_json::to_string_pretty(schema)?;
    ensure_single_trailing_newline(&mut json);
    Ok(json)
}

/// Removes every JSON Schema `required` list from a schema tree.
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
fn is_schema_map_key(key: &str) -> bool {
    matches!(
        key,
        "$defs" | "definitions" | "properties" | "patternProperties"
    )
}

/// Removes `required` lists from every schema inside a schema map.
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
fn section_schema_for_path(root_schema: &Value, section_path: &[&str]) -> Option<Value> {
    let mut current = root_schema;

    for section in section_path {
        current = current.get("properties")?.get(*section)?;
        current = resolve_schema_reference(root_schema, current).unwrap_or(current);
    }

    Some(standalone_section_schema(root_schema, current))
}

/// Resolves the local schema reference shape emitted by `schemars`.
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
fn resolve_json_pointer_ref<'a>(root_schema: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix('#')?;
    root_schema.pointer(pointer)
}

/// Copies root-level schema metadata needed by an extracted section schema.
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
fn schema_ref_name(reference: &str, prefix: &str) -> Option<String> {
    let name = reference.strip_prefix(prefix)?.split('/').next()?;
    Some(decode_json_pointer_token(name))
}

/// Decodes the escaping used by one JSON Pointer path token.
fn decode_json_pointer_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// Retains only referenced entries in a root schema map.
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
pub(crate) fn nested_section_paths(meta: &'static Meta) -> Vec<Vec<&'static str>> {
    let mut paths = Vec::new();
    collect_nested_section_paths(meta, &mut Vec::new(), &mut paths);
    paths
}

/// Finds nested sections whose field schema opts into template/schema splitting.
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
fn section_has_tree_split_marker(root_schema: &Value, section_path: &[&str]) -> bool {
    section_property_schema_for_path(root_schema, section_path)
        .and_then(|schema| schema.get(TREE_SPLIT_SCHEMA_EXTENSION))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Returns the original property schema for a nested section path.
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
