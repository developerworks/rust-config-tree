//! Format inference for config files and generated templates.

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
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_config_tree::config::ConfigFormat;
    ///
    /// assert_eq!(ConfigFormat::from_path("config.toml"), ConfigFormat::Toml);
    /// assert_eq!(ConfigFormat::from_path("config.json5"), ConfigFormat::Json);
    /// assert_eq!(ConfigFormat::from_path("config.unknown"), ConfigFormat::Yaml);
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        match path.as_ref().extension().and_then(OsStr::to_str) {
            Some("toml") => Self::Toml,
            Some("json" | "json5") => Self::Json,
            Some("yaml" | "yml") | Some(_) | None => Self::Yaml,
        }
    }
}
