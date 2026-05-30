//! Local `$ref` resolution and schema-map reference collection.

use std::collections::BTreeSet;

use serde_json::Value;

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
pub fn resolve_schema_reference<'a>(root_schema: &'a Value, schema: &'a Value) -> Option<&'a Value> {
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
pub fn collect_transitive_schema_refs(
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
pub fn collect_schema_refs(
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
pub fn retain_schema_map(schema: &mut Value, key: &str, used_names: &BTreeSet<String>) {
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
