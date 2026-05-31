//! Root schema generation and JSON serialization.

use schemars::{JsonSchema, generate::SchemaSettings};
use serde_json::Value;

use crate::{config::ConfigResult, config_util::ensure_single_trailing_newline};

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
pub fn root_config_schema<S>() -> ConfigResult<Value>
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
pub fn schema_json(schema: &Value) -> ConfigResult<String> {
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
