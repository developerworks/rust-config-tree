#![warn(missing_docs)]

//! Configuration-tree loading and CLI helpers for layered config files.
//!
//! The high-level API loads `confique` schemas directly, while the lower-level
//! tree traversal helpers remain available for custom loaders.
//!
//! Use [`config::ConfigSchema`] with a `confique::Config` type when your schema owns an
//! include field. Use [`config::load_config`] to load the root config, all recursive
//! includes, `.env` values, and schema-declared environment values into the
//! final schema. Use [`config::build_config_figment`] or [`config::load_config_with_figment`]
//! when you need runtime source tracking. Use [`config::write_config_templates`] or
//! [`cli::ConfigCommand`] to generate example template files that mirror the same
//! include tree. Use [`config::write_config_schemas`] to generate root and section JSON
//! Schemas for editor completion and validation. Use
//! [`cli::install_shell_completion`] and [`cli::uninstall_shell_completion`] for reusable
//! shell completion lifecycle commands.

pub mod cli;
pub mod config;
pub mod config_schema;
pub mod error;
pub mod path;
pub mod template_tree;
pub mod tree;

mod config_env;
mod config_format;
mod config_load;
mod config_output;
mod config_templates;
mod config_trace;
mod config_util;
