//! Unit tests for transparent array section support.

use crate::ConfigSchema;
use crate::config::{load_config, template_targets_for_paths, write_config_schemas};
use crate::transparent_array_section;
use confique::Config;
use schemars::JsonSchema;
use std::{fs, path::PathBuf};

transparent_array_section! {
    /// Sample child declarations for tests.
    pub struct ChildrenSection {
        #[config(default = [{ "name": "worker" }])]
        pub items: Vec<ChildDeclaration>,
    }
}

/// One child declaration used by transparent section tests.
#[derive(Debug, Clone, PartialEq, Eq, Config, JsonSchema, serde::Serialize, serde::Deserialize)]
pub struct ChildDeclaration {
    /// Child name.
    pub name: String,
}

/// Root config used by transparent section tests.
#[derive(Debug, Clone, PartialEq, Config, JsonSchema, ConfigSchema)]
pub struct TestConfig {
    /// Included child files.
    #[config(default = [])]
    pub include: Vec<PathBuf>,
    /// Root strategy value.
    #[config(default = "demo")]
    pub mode: String,
    /// Transparent child declarations.
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true, "x-tree-transparent-array" = true))]
    pub children: ChildrenSection,
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rust-config-tree-transparent-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn transparent_split_template_rewrites_flow_style_body() {
    let dir = temp_dir("template");
    let root = dir.join("root.example.yaml");
    fs::write(&root, "mode: demo\n").expect("write root");

    let targets = template_targets_for_paths::<TestConfig>(&root, &root).expect("template targets");
    let children = targets
        .iter()
        .find(|target| target.path.ends_with("children.yaml"))
        .expect("children template");

    assert!(!children.content.contains("[{"));
    assert!(children.content.contains("- name: worker"));
    assert!(!children.content.contains("\nchildren:\n"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn transparent_split_schema_is_array() {
    let dir = temp_dir("schema");
    let schema_path = dir.join("test.schema.json");
    write_config_schemas::<TestConfig>(&schema_path).expect("write schemas");

    let children_schema = fs::read_to_string(dir.join("children.schema.json")).expect("read");
    assert!(children_schema.contains("\"type\": \"array\""));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn load_accepts_body_only_children_yaml() {
    let dir = temp_dir("load");
    let root = dir.join("root.yaml");
    let children = dir.join("children.yaml");

    fs::write(&root, "include:\n  - children.yaml\nmode: demo\n").expect("write root");
    fs::write(&children, "- name: api\n").expect("write children");

    let config = load_config::<TestConfig>(&root).expect("load config");
    assert_eq!(config.children.items.len(), 1);
    assert_eq!(config.children.items[0].name, "api");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn load_omitted_transparent_section_does_not_apply_template_default() {
    let dir = temp_dir("omitted");
    let root = dir.join("root.yaml");
    fs::write(&root, "mode: demo\n").expect("write root");

    let config = load_config::<TestConfig>(&root).expect("load config");
    assert!(config.children.items.is_empty());

    let _ = fs::remove_dir_all(dir);
}
