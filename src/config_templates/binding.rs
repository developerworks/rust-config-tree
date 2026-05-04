//! Schema binding helpers for generated config templates.

use std::{
    path::Component,
    path::{Path, PathBuf},
};

use crate::absolutize_lexical;

use super::section::section_path_for_target;
use crate::{
    config::{ConfigResult, ConfigSchema},
    config_format::ConfigFormat,
    config_schema::schema_path_for_section,
};

/// Chooses the schema path that should be bound to one template target.
///
/// # Type Parameters
///
/// - `S`: Config schema type used to map template targets to sections.
///
/// # Arguments
///
/// - `root_base_dir`: Directory containing the root template output.
/// - `target_path`: Template target path being bound to a schema.
/// - `root_schema_path`: Path to the root generated schema.
/// - `split_paths`: Nested section paths that have their own schemas.
///
/// # Returns
///
/// Returns the root schema path or the matching section schema path.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn schema_path_for_template_target<S>(
    root_base_dir: &Path,
    target_path: &Path,
    root_schema_path: &Path,
    split_paths: &[Vec<&'static str>],
) -> PathBuf
where
    S: ConfigSchema,
{
    section_path_for_target::<S>(root_base_dir, target_path, split_paths)
        .filter(|section_path| !section_path.is_empty())
        .map(|section_path| schema_path_for_section(root_schema_path, &section_path))
        .unwrap_or_else(|| root_schema_path.to_path_buf())
}

/// Adds an editor schema binding when the template format supports one.
///
/// # Arguments
///
/// - `template_path`: Template path whose extension selects directive syntax.
/// - `schema_path`: Schema path to reference from the template.
/// - `content`: Existing template content.
///
/// # Returns
///
/// Returns template content with a directive for TOML/YAML or a top-level
/// `$schema` property for JSON/JSON5.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(super) fn template_with_schema_directive(
    template_path: &Path,
    schema_path: &Path,
    content: &str,
) -> ConfigResult<String> {
    let schema_ref = schema_reference_for_path(template_path, schema_path)?;
    let content = match ConfigFormat::from_path(template_path) {
        ConfigFormat::Yaml => format!("# yaml-language-server: $schema={schema_ref}\n\n{content}"),
        ConfigFormat::Toml => format!("#:schema {schema_ref}\n\n{content}"),
        ConfigFormat::Json => template_with_json_schema_property(&schema_ref, content),
    };

    Ok(content)
}

/// Inserts a top-level JSON Schema property into a JSON5 object template.
///
/// # Arguments
///
/// - `schema_ref`: Schema path reference to write.
/// - `content`: Existing JSON5 object template content.
///
/// # Returns
///
/// Returns `content` with a leading `$schema` property when the content is an
/// object template.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn template_with_json_schema_property(schema_ref: &str, content: &str) -> String {
    let schema_ref = serde_json::to_string(schema_ref).expect("schema reference is a string");

    if let Some(body) = content.strip_prefix("{\n") {
        let separator = if body.trim_start().starts_with('}') {
            "\n"
        } else {
            ",\n"
        };
        return format!("{{\n  \"$schema\": {schema_ref}{separator}{body}");
    }

    if content.trim() == "{}" {
        return format!("{{\n  \"$schema\": {schema_ref}\n}}\n");
    }

    content.to_owned()
}

/// Builds a template-local schema reference from two filesystem paths.
///
/// # Arguments
///
/// - `template_path`: Template path that will contain the schema directive.
/// - `schema_path`: Schema path referenced by the directive.
///
/// # Returns
///
/// Returns a path reference relative to the template directory when possible.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn schema_reference_for_path(template_path: &Path, schema_path: &Path) -> ConfigResult<String> {
    let template_path = absolutize_lexical(template_path)?;
    let schema_path = absolutize_lexical(schema_path)?;
    let template_dir = template_path.parent().unwrap_or_else(|| Path::new("."));
    let relative_path = relative_path_from(&schema_path, template_dir);
    Ok(render_schema_reference(&relative_path))
}

/// Computes a lexical relative path from `base` to `path`.
///
/// # Arguments
///
/// - `path`: Destination path to reference.
/// - `base`: Base directory from which the reference should be relative.
///
/// # Returns
///
/// Returns a lexical relative path, or `path` unchanged when no prefix matches.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn relative_path_from(path: &Path, base: &Path) -> PathBuf {
    let path_components = path.components().collect::<Vec<_>>();
    let base_components = base.components().collect::<Vec<_>>();

    let mut common_len = 0;
    while common_len < path_components.len()
        && common_len < base_components.len()
        && path_components[common_len] == base_components[common_len]
    {
        common_len += 1;
    }

    if common_len == 0 {
        return path.to_path_buf();
    }

    let mut relative = PathBuf::new();
    for component in &base_components[common_len..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }

    for component in &path_components[common_len..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

/// Renders a schema reference in the form expected by editor directives.
///
/// # Arguments
///
/// - `path`: Schema path reference to render.
///
/// # Returns
///
/// Returns a slash-separated reference with `./` added for local relative paths.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn render_schema_reference(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if path.is_absolute() || value.starts_with("../") || value.starts_with("./") {
        value
    } else {
        format!("./{value}")
    }
}
