//! Format inference and renderer option presets.
//!
//! Runtime loading and template rendering both use [`ConfigFormat`] so unknown
//! or extensionless files consistently fall back to YAML. The private option
//! helpers keep generated templates comment-rich and visually aligned across
//! YAML, TOML, and JSON5.

use std::{ffi::OsStr, path::Path};

/// File format used when loading config files or rendering templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// YAML format, selected for `.yaml`, `.yml`, unknown extensions, and paths
    /// without an extension.
    Yaml,
    /// TOML format, selected for `.toml`.
    Toml,
    /// JSON5-compatible format, selected for `.json` and `.json5`.
    Json,
}

/// Path-based format inference for config files and generated templates.
impl ConfigFormat {
    /// Infers the config format from a path extension.
    ///
    /// Unknown extensions intentionally fall back to YAML.
    ///
    /// # Arguments
    ///
    /// - `path`: Config or template path whose extension should be inspected.
    ///
    /// # Returns
    ///
    /// Returns the inferred [`ConfigFormat`].
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("toml") => Self::Toml,
            Some("json" | "json5") => Self::Json,
            Some("yaml" | "yml") | Some(_) | None => Self::Yaml,
        }
    }
}

/// Builds the YAML renderer options used by default templates.
pub(crate) fn yaml_options() -> confique::yaml::FormatOptions {
    let mut options = confique::yaml::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

/// Builds the TOML renderer options used by default templates.
pub(crate) fn toml_options() -> confique::toml::FormatOptions {
    let mut options = confique::toml::FormatOptions::default();
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}

/// Builds the JSON5 renderer options used by default templates.
pub(crate) fn json5_options() -> confique::json5::FormatOptions {
    let mut options = confique::json5::FormatOptions::default();
    options.indent = 2;
    options.general.comments = true;
    options.general.env_keys = true;
    options.general.nested_field_gap = 1;
    options
}
