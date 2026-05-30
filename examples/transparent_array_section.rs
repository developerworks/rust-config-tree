//! Generates templates and schemas for a config with transparent array sections.
//!
//! Run from the repository root:
//!
//! ```bash
//! cargo run --example transparent_array_section
//! ```
//!
//! Template generation for transparent split sections uses
//! `template_targets_for_paths` or `write_config_templates`. See
//! `manual/en/ide-completions.md` for the full workflow.

use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;
use rust_config_tree::{
    config::{load_config, write_config_schemas},
    transparent_array_section,
    ConfigSchema,
};
use schemars::JsonSchema;

transparent_array_section! {
    /// Child declarations stored as a transparent array section.
    pub struct ChildrenSection {
        #[config(default = [{ "name": "worker" }])]
        pub items: Vec<ChildDeclaration>,
    }
}

/// One child declaration used by the example config schema.
#[derive(Debug, Clone, PartialEq, Config, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChildDeclaration {
    /// Child name.
    pub name: String,
}

/// Root config schema used by the transparent array section example.
#[derive(Debug, Clone, PartialEq, Config, JsonSchema, ConfigSchema)]
pub struct AppConfig {
    /// Included child configuration files.
    #[config(default = [])]
    pub include: Vec<PathBuf>,

    /// Root scalar value.
    #[config(default = "demo")]
    pub mode: String,

    /// Transparent child declarations split into `children.yaml`.
    #[config(nested)]
    #[schemars(extend(
        "x-tree-split" = true,
        "x-tree-transparent-array" = true
    ))]
    pub children: ChildrenSection,
}

/// Writes demo files, generates schemas/templates, loads split config, and prints results.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir = temp_example_dir("transparent-array-section")?;
    let root = dir.join("config.yaml");
    let schema = dir.join("app.schema.json");
    let children = dir.join("children.yaml");

    fs::write(
        &root,
        "include:\n  - children.yaml\nmode: demo\n",
    )?;
    fs::write(&children, "- name: api\n")?;

    write_config_schemas::<AppConfig>(&schema)?;
    println!("schema: {}", schema.display());

    let config = load_config::<AppConfig>(&root)?;
    println!("loaded children: {}", config.children.len());
    assert_eq!(config.children.items[0].name, "api");

    println!("split file: {}", children.display());

    let _ = fs::remove_dir_all(dir);
    Ok(())
}

/// Creates a unique temporary directory for one example run.
fn temp_example_dir(name: &str) -> io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust-config-tree-{name}-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
