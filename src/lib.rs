#![warn(missing_docs)]

//! Configuration-tree loading and CLI helpers for layered config files.
//!
//! The high-level API loads `confique` schemas directly, while the lower-level
//! tree traversal helpers remain available for custom loaders.
//!
//! Use [`ConfigSchema`] with a `confique::Config` type when your schema owns an
//! include field. Use [`load_config`] to load the root config, all recursive
//! includes, `.env` values, and schema-declared environment values into the
//! final schema. Use [`build_config_figment`] or [`load_config_with_figment`]
//! when you need runtime source tracking. Use [`write_config_templates`] or
//! [`ConfigCommand`] to generate example template files that mirror the same
//! include tree. Use [`write_config_schemas`] to generate root and section JSON
//! Schemas for editor completion and validation.

mod cli;
mod config;
mod error;
mod path;
mod template;
mod tree;

pub use cli::{
    ConfigCommand, handle_config_command, install_shell_completion, print_shell_completion,
    upsert_managed_block,
};
pub use config::{
    ConfigFormat, ConfigResult, ConfigSchema, ConfigSchemaTarget, ConfigTemplateTarget,
    ConfiqueEnvProvider, build_config_figment, config_schema_targets_for_path, load_config,
    load_config_from_figment, load_config_with_figment, template_for_path,
    template_targets_for_paths, template_targets_for_paths_with_schema, trace_config_sources,
    write_config_schema, write_config_schemas, write_config_templates,
    write_config_templates_with_schema,
};
pub use error::{BoxError, ConfigError, ConfigTreeError, Result};
pub use path::{absolutize_lexical, normalize_lexical, resolve_include_path};
pub use template::{TemplateTarget, collect_template_targets, select_template_source};
pub use tree::{
    ConfigNode, ConfigSource, ConfigTree, ConfigTreeOptions, IncludeOrder, load_config_tree,
};
