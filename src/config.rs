//! High-level `confique` integration and config-template rendering.
//!
//! This module loads `.env` values, builds a Figment runtime source graph,
//! extracts it into a `confique` schema for defaults and validation, renders
//! example templates that mirror the same include tree, and writes JSON Schema
//! files that editors can use for completion and validation. YAML templates can
//! also be split across nested schema sections.

use std::path::PathBuf;

use confique::Config;

pub use crate::config_env::ConfiqueEnvProvider;
pub use crate::config_format::ConfigFormat;
pub use crate::config_load::{
    build_config_figment, load_config, load_config_from_figment, load_config_with_figment,
};
pub(crate) use crate::config_output::resolve_config_template_output;
pub use crate::config_schema::{
    ConfigSchemaTarget, config_schema_targets_for_path, write_config_schema, write_config_schemas,
};
pub use crate::config_templates::{
    ConfigTemplateTarget, template_for_path, template_targets_for_paths,
    template_targets_for_paths_with_schema, write_config_templates,
    write_config_templates_with_schema,
};
pub use crate::config_trace::trace_config_sources;

/// Result type used by the high-level configuration API.
///
/// The error type is [`ConfigError`](crate::ConfigError).
pub type ConfigResult<T> = std::result::Result<T, crate::ConfigError>;

/// A `confique` schema that can expose recursive include paths and template
/// section layout.
///
/// Implement this trait for the same type that derives `confique::Config`.
/// `include_paths` receives a partially loaded layer so the crate can discover
/// child config files before the final schema is merged.
pub trait ConfigSchema: Config + Sized {
    /// Returns include paths declared by a loaded config layer.
    ///
    /// Relative paths are resolved from the file that declared them. Empty paths
    /// are rejected before traversal continues.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded `confique` layer for one config file.
    ///
    /// # Returns
    ///
    /// Returns include paths declared by `layer`.
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf>;

    /// Overrides the generated template file path for a split nested section.
    ///
    /// A nested section is split only when its field schema has
    /// `x-tree-split = true`, for example
    /// `#[schemars(extend("x-tree-split" = true))]`. By default, top-level
    /// split sections are generated as `config/<field>.yaml` and nested split
    /// sections as children of their parent section file stem, e.g.
    /// `config/trading/risk.yaml`.
    ///
    /// # Arguments
    ///
    /// - `section_path`: Path of nested schema field names from the root schema
    ///   to the section being rendered.
    ///
    /// # Returns
    ///
    /// Returns `Some(path)` to override the generated file path, or `None` to
    /// use the default section path.
    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        let _ = section_path;
        None
    }
}

#[cfg(test)]
#[path = "unit_tests/config.rs"]
mod unit_tests;
