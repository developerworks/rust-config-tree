//! Schema binding helpers for generated TOML and YAML templates.

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

/// Prepends an editor schema directive when the template format supports one.
pub(super) fn template_with_schema_directive(
    template_path: &Path,
    schema_path: &Path,
    content: &str,
) -> ConfigResult<String> {
    let schema_ref = schema_reference_for_path(template_path, schema_path)?;
    let directive = match ConfigFormat::from_path(template_path) {
        ConfigFormat::Yaml => Some(format!("# yaml-language-server: $schema={schema_ref}")),
        ConfigFormat::Toml => Some(format!("#:schema {schema_ref}")),
        ConfigFormat::Json => None,
    };

    let Some(directive) = directive else {
        return Ok(content.to_owned());
    };

    Ok(format!("{directive}\n\n{content}"))
}

/// Builds a template-local schema reference from two filesystem paths.
fn schema_reference_for_path(template_path: &Path, schema_path: &Path) -> ConfigResult<String> {
    let template_path = absolutize_lexical(template_path)?;
    let schema_path = absolutize_lexical(schema_path)?;
    let template_dir = template_path.parent().unwrap_or_else(|| Path::new("."));
    let relative_path = relative_path_from(&schema_path, template_dir);
    Ok(render_schema_reference(&relative_path))
}

/// Computes a lexical relative path from `base` to `path`.
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
fn render_schema_reference(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    if path.is_absolute() || value.starts_with("../") || value.starts_with("./") {
        value
    } else {
        format!("./{value}")
    }
}
