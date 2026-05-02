use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use confique::{Config, File, FileFormat};

use crate::{
    ConfigError, ConfigSource, ConfigTreeOptions, IncludeOrder, collect_template_targets,
    normalize_lexical,
};

pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

pub trait ConfigSchema: Config + Sized {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Toml,
    Json,
}

impl ConfigFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("toml") => Self::Toml,
            Some("json" | "json5") => Self::Json,
            Some("yaml" | "yml") | Some(_) | None => Self::Yaml,
        }
    }

    pub fn as_file_format(self) -> FileFormat {
        match self {
            Self::Yaml => FileFormat::Yaml,
            Self::Toml => FileFormat::Toml,
            Self::Json => FileFormat::Json5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTemplateTarget {
    pub path: PathBuf,
    pub content: String,
}

pub fn load_config<S>(path: impl AsRef<Path>) -> ConfigResult<S>
where
    S: ConfigSchema,
{
    let mut builder = S::builder().env();
    let tree = ConfigTreeOptions::default()
        .include_order(IncludeOrder::Reverse)
        .load(
            path,
            |path| -> ConfigResult<ConfigSource<<S as Config>::Layer>> {
                let layer = load_layer::<S>(path)?;
                let include_paths = S::include_paths(&layer);
                Ok(ConfigSource::new(layer, include_paths))
            },
        )?;

    for layer in tree.into_values() {
        builder = builder.preloaded(layer);
    }

    Ok(builder.load()?)
}

pub fn load_layer<S>(path: &Path) -> ConfigResult<<S as Config>::Layer>
where
    S: ConfigSchema,
{
    Ok(File::with_format(path, ConfigFormat::from_path(path).as_file_format()).load()?)
}

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

pub fn template_targets_for_paths<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<Vec<ConfigTemplateTarget>>
where
    S: ConfigSchema,
{
    collect_template_targets(
        config_path,
        output_path.as_ref(),
        |source_path| -> ConfigResult<Vec<PathBuf>> {
            let layer = load_layer::<S>(source_path)?;
            Ok(S::include_paths(&layer))
        },
    )?
    .into_iter()
    .map(|target| {
        let (_, target_path, include_paths) = target.into_parts();
        Ok(ConfigTemplateTarget {
            content: template_for_path_with_includes::<S>(&target_path, &include_paths)?,
            path: target_path,
        })
    })
    .collect()
}

pub fn write_config_templates<S>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> ConfigResult<()>
where
    S: ConfigSchema,
{
    for target in template_targets_for_paths::<S>(config_path, output_path)? {
        write_template(&target.path, &target.content)?;
    }

    Ok(())
}

pub(crate) fn write_template(path: &Path, content: &str) -> ConfigResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

pub(crate) fn resolve_config_template_output(output: Option<PathBuf>) -> ConfigResult<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let output = output.unwrap_or_else(|| PathBuf::from("config.example.yaml"));
    let output = if output.is_absolute() {
        output
    } else {
        current_dir.join(output)
    };

    Ok(normalize_lexical(output))
}

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

fn render_yaml_include(paths: &[PathBuf]) -> String {
    let mut out = String::from("include:\n");
    for path in paths {
        out.push_str("  - ");
        out.push_str(&quote_path(path));
        out.push('\n');
    }
    out
}

fn render_toml_include(paths: &[PathBuf]) -> String {
    let entries = paths
        .iter()
        .map(|path| quote_path(path))
        .collect::<Vec<_>>()
        .join(", ");
    format!("include = [{entries}]\n")
}

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

fn quote_path(path: &Path) -> String {
    serde_json::to_string(&path.to_string_lossy()).expect("path string serialization cannot fail")
}

fn strip_prefix_once<'a>(value: &'a str, prefix: &str) -> &'a str {
    value.strip_prefix(prefix).unwrap_or(value)
}

fn yaml_options() -> confique::yaml::FormatOptions {
    let mut options = confique::yaml::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

fn toml_options() -> confique::toml::FormatOptions {
    let mut options = confique::toml::FormatOptions::default();
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

fn json5_options() -> confique::json5::FormatOptions {
    let mut options = confique::json5::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

#[cfg(test)]
#[path = "unit_tests/config.rs"]
mod unit_tests;
