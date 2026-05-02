//! Recursive include tree utilities for layered configuration files.
//!
//! This crate is intentionally format-agnostic. The caller provides the loader
//! that reads one config file and returns both its parsed value and the include
//! paths declared by that file.

mod error;
mod path;
mod template;
mod tree;

pub use error::{BoxError, ConfigTreeError, Result};
pub use path::{absolutize_lexical, normalize_lexical, resolve_include_path};
pub use template::{TemplateTarget, collect_template_targets, select_template_source};
pub use tree::{
    ConfigNode, ConfigSource, ConfigTree, ConfigTreeOptions, IncludeOrder, load_config_tree,
};
