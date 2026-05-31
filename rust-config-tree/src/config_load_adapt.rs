//! YAML adaptation for transparent array configuration sections.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use figment::{
    Figment,
    providers::{Format, Serialized, Yaml},
};
use schemars::JsonSchema;
use serde_json::Value;
use serde_yaml::{Mapping, Value as YamlValue};

use crate::{
    config::{ConfigResult, ConfigSchema},
    config_format::ConfigFormat,
    config_schema::{
        generate::root_config_schema,
        paths::{inner_field_for_section, split_section_paths, transparent_array_section_paths},
    },
    config_templates::section::section_path_for_target,
    path::absolutize_lexical,
};

/// Tracks transparent section keys observed while merging config files.
#[derive(Debug, Default, Clone)]
pub struct TransparentSectionTracker {
    /// Top-level section field names present in merged config sources.
    pub seen_sections: HashSet<String>,
}

impl TransparentSectionTracker {
    /// Records one transparent section key observed in a merged config source.
    pub fn record_section(&mut self, section: &str) {
        if !section.is_empty() {
            self.seen_sections.insert(section.to_string());
        }
    }
}

/// Runtime metadata used to adapt transparent array sections during loading.
#[derive(Debug, Clone)]
pub struct TransparentSectionContext {
    /// Root config directory used to resolve split section template paths.
    pub root_base_dir: PathBuf,
    /// Split section paths marked with `x-tree-split`.
    pub split_paths: Vec<Vec<&'static str>>,
    /// Split section paths marked with `x-tree-transparent-array`.
    pub transparent_paths: Vec<Vec<&'static str>>,
    /// Full root JSON Schema used to resolve inner field names.
    pub full_schema: Value,
}

impl TransparentSectionContext {
    /// Builds transparent section metadata for one config schema type.
    pub fn for_schema<S>(root_config_path: &Path) -> ConfigResult<Self>
    where
        S: ConfigSchema + JsonSchema,
    {
        let root_config_path = absolutize_lexical(root_config_path)?;
        let root_base_dir = root_config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let full_schema = root_config_schema::<S>()?;
        let split_paths = split_section_paths::<S>(&full_schema);
        let transparent_paths = transparent_array_section_paths::<S>(&full_schema);

        Ok(Self {
            root_base_dir,
            split_paths,
            transparent_paths,
            full_schema,
        })
    }

    /// Returns the split section path represented by one config file, if any.
    pub fn section_path_for_file<S>(&self, path: &Path) -> Option<Vec<&'static str>>
    where
        S: ConfigSchema,
    {
        section_path_for_target::<S>(&self.root_base_dir, path, &self.split_paths)
    }

    /// Returns whether one section path serializes as a transparent array.
    pub fn is_transparent_section(&self, section_path: &[&str]) -> bool {
        self.transparent_paths
            .iter()
            .any(|path| path.as_slice() == section_path)
    }

    /// Returns the confique inner field for one transparent section path.
    pub fn inner_field_for_section(&self, section_path: &[&str]) -> String {
        inner_field_for_section(&self.full_schema, section_path)
    }
}

/// Returns whether one config file represents a split section body file.
pub fn is_split_section_file<S>(context: &TransparentSectionContext, path: &Path) -> bool
where
    S: ConfigSchema,
{
    context.section_path_for_file::<S>(path).is_some()
}

/// Merges one config file into Figment after adapting transparent array sections.
pub fn merge_adapted_file<S>(
    figment: Figment,
    path: &Path,
    context: &TransparentSectionContext,
    tracker: &mut TransparentSectionTracker,
) -> ConfigResult<Figment>
where
    S: ConfigSchema + JsonSchema,
{
    if let Some(section_path) = context.section_path_for_file::<S>(path) {
        let section_key = section_path
            .last()
            .copied()
            .expect("split section path must not be empty");

        if context.is_transparent_section(&section_path) && !yaml_has_root_key(path, section_key) {
            tracker.record_section(section_key);
            let body = read_yaml_value(path)?;
            let inner_field = context.inner_field_for_section(&section_path);
            let section_body = wrap_inner_field(body, inner_field.as_str());
            let merged = nest_section_mapping(&section_path, section_body);
            return Ok(figment.merge(Serialized::defaults(YamlValue::Mapping(merged))));
        }
    }

    merge_mapping_file::<S>(figment, path, context, tracker)
}

/// Merges explicit empty transparent sections that never appeared in config files.
///
/// This prevents `confique` template sample defaults from becoming runtime values
/// when a transparent section is omitted entirely from the loaded config tree.
pub fn merge_missing_transparent_sections(
    figment: Figment,
    context: &TransparentSectionContext,
    tracker: &TransparentSectionTracker,
) -> Figment {
    let mut figment = figment;

    for section_path in &context.transparent_paths {
        let Some(section_key) = section_path.last().copied() else {
            continue;
        };

        if tracker.seen_sections.contains(section_key) {
            continue;
        }

        let inner_field = context.inner_field_for_section(section_path);
        let empty_items = wrap_inner_field(YamlValue::Sequence(Vec::new()), inner_field.as_str());
        let merged = nest_section_mapping(section_path, empty_items);
        figment = figment.merge(Serialized::defaults(YamlValue::Mapping(merged)));
    }

    figment
}

fn nest_section_mapping(section_path: &[&str], body: YamlValue) -> Mapping {
    let mut current = body;
    for section in section_path.iter().rev() {
        let mut map = Mapping::new();
        map.insert(YamlValue::String(section.to_string()), current);
        current = YamlValue::Mapping(map);
    }

    match current {
        YamlValue::Mapping(map) => map,
        other => {
            let mut map = Mapping::new();
            if let Some(section) = section_path.last() {
                map.insert(YamlValue::String(section.to_string()), other);
            }
            map
        }
    }
}

fn merge_mapping_file<S>(
    figment: Figment,
    path: &Path,
    context: &TransparentSectionContext,
    tracker: &mut TransparentSectionTracker,
) -> ConfigResult<Figment>
where
    S: ConfigSchema,
{
    match ConfigFormat::from_path(path) {
        ConfigFormat::Yaml => {
            let value = read_yaml_value(path)?;
            if matches!(value, YamlValue::Null) {
                return Ok(figment);
            }
            let split_file = context.section_path_for_file::<S>(path);
            record_transparent_sections_in_value(&value, context, tracker);
            let adapted = adapt_config_yaml(value, context, split_file.as_deref());
            Ok(figment.merge(Serialized::defaults(adapted)))
        }
        ConfigFormat::Toml => Ok(figment.merge(figment::providers::Toml::file(path))),
        ConfigFormat::Json => Ok(figment.merge(figment::providers::Json::file(path))),
    }
}

fn record_transparent_sections_in_value(
    value: &YamlValue,
    context: &TransparentSectionContext,
    tracker: &mut TransparentSectionTracker,
) {
    let YamlValue::Mapping(map) = value else {
        return;
    };

    for key in map.keys() {
        if is_transparent_section_key(key, context) {
            if let Some(section) = key.as_str() {
                tracker.record_section(section);
            }
        }
    }
}

/// Adapts one YAML document for transparent array sections before Figment merge.
pub fn adapt_config_yaml(
    value: YamlValue,
    context: &TransparentSectionContext,
    split_file: Option<&[&str]>,
) -> YamlValue {
    match value {
        YamlValue::Sequence(_) if split_file.is_some() => {
            adapt_split_section_body(value, context, split_file.expect("split section path"))
        }
        YamlValue::Mapping(map) => {
            let mut adapted = Mapping::new();
            for (key, child) in map {
                let next = if is_transparent_section_key(&key, context) {
                    let section = key.as_str().unwrap_or("");
                    adapt_section_value(child, context, section)
                } else {
                    adapt_config_yaml(child, context, None)
                };
                adapted.insert(key, next);
            }
            YamlValue::Mapping(adapted)
        }
        other => other,
    }
}

fn adapt_split_section_body(
    value: YamlValue,
    context: &TransparentSectionContext,
    section_path: &[&str],
) -> YamlValue {
    let inner_field_name = context.inner_field_for_section(section_path);
    let inner_field = inner_field_name.as_str();
    match value {
        YamlValue::Sequence(sequence) => {
            wrap_inner_field(YamlValue::Sequence(sequence), inner_field)
        }
        YamlValue::Mapping(map)
            if map.contains_key(YamlValue::String(inner_field_name.clone())) =>
        {
            YamlValue::Mapping(map)
        }
        other => other,
    }
}

fn adapt_section_value(
    value: YamlValue,
    context: &TransparentSectionContext,
    section: &str,
) -> YamlValue {
    let fallback = [section];
    let section_path = context
        .transparent_paths
        .iter()
        .find(|path| path.last() == Some(&section))
        .map(Vec::as_slice)
        .unwrap_or(&fallback);

    let inner_field_name = context.inner_field_for_section(section_path);
    let inner_field = inner_field_name.as_str();
    match value {
        YamlValue::Sequence(sequence) => {
            wrap_inner_field(YamlValue::Sequence(sequence), inner_field)
        }
        YamlValue::Mapping(map)
            if map.contains_key(YamlValue::String(inner_field_name.clone())) =>
        {
            YamlValue::Mapping(map)
        }
        other => other,
    }
}

fn wrap_inner_field(value: YamlValue, inner_field: &str) -> YamlValue {
    let mut map = Mapping::new();
    map.insert(YamlValue::String(inner_field.to_string()), value);
    YamlValue::Mapping(map)
}

fn is_transparent_section_key(key: &YamlValue, context: &TransparentSectionContext) -> bool {
    key.as_str().is_some_and(|name| {
        context
            .transparent_paths
            .iter()
            .any(|path| path.last() == Some(&name))
    })
}

fn yaml_has_root_key(path: &Path, key: &str) -> bool {
    if ConfigFormat::from_path(path) != ConfigFormat::Yaml || key.is_empty() {
        return false;
    }

    Figment::from(Yaml::file(path)).find_value(key).is_ok()
}

fn read_yaml_value(path: &Path) -> ConfigResult<YamlValue> {
    let content = std::fs::read_to_string(path)?;
    serde_yaml::from_str(&content).map_err(|error| {
        figment::Error::from(figment::error::Kind::Message(error.to_string())).into()
    })
}
