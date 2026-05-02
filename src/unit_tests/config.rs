use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;

use super::*;

#[derive(Debug, Config)]
#[allow(dead_code)]
struct TestConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
    #[config(nested)]
    server: TestServerConfig,
}

#[derive(Debug, Config)]
struct TestServerConfig {
    #[config(default = 8080)]
    port: u16,
}

impl ConfigSchema for TestConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_include_paths() -> Vec<PathBuf> {
        vec![PathBuf::from("config/server.yaml")]
    }
}

#[test]
fn load_config_returns_accessible_config_object() {
    let root = temp_dir_path("load-config");
    fs::create_dir_all(root.join("config")).unwrap();
    fs::write(
        root.join("config.yaml"),
        concat!(
            "include:\n",
            "  - config/server.yaml\n",
            "\n",
            "mode: shadow\n",
        ),
    )
    .unwrap();
    fs::write(
        root.join("config").join("server.yaml"),
        concat!("server:\n", "  port: 7777\n",),
    )
    .unwrap();

    let config = load_config::<TestConfig>(root.join("config.yaml")).unwrap();

    assert_eq!(config.mode, "shadow");
    assert_eq!(config.server.port, 7777);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn template_targets_for_paths_recurses_and_renders_templates() {
    let root = temp_dir_path("template-config");
    fs::create_dir_all(root.join("config")).unwrap();
    let config_path = root.join("config.yaml");
    let output_path = root.join("examples").join("config.example.yaml");
    fs::write(
        &config_path,
        concat!("include:\n", "  - config/server.yaml\n",),
    )
    .unwrap();
    fs::write(root.join("config").join("server.yaml"), "").unwrap();

    let targets = template_targets_for_paths::<TestConfig>(&config_path, &output_path).unwrap();

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].path, output_path);
    assert!(targets[0].content.contains("include:\n"));
    assert!(targets[0].content.contains("\"config/server.yaml\""));
    assert_eq!(
        targets[1].path,
        root.join("examples").join("config").join("server.yaml")
    );
    assert!(targets[1].content.contains("server:"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn write_config_templates_creates_parent_directories() {
    let root = temp_dir_path("write-templates");
    fs::create_dir_all(root.join("config")).unwrap();
    let config_path = root.join("config.yaml");
    let output_path = root.join("examples").join("config.example.yaml");
    fs::write(
        &config_path,
        concat!("include:\n", "  - config/server.yaml\n",),
    )
    .unwrap();
    fs::write(root.join("config").join("server.yaml"), "").unwrap();

    write_config_templates::<TestConfig>(&config_path, &output_path).unwrap();

    assert!(output_path.exists());
    assert!(
        root.join("examples")
            .join("config")
            .join("server.yaml")
            .exists()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn config_format_is_inferred_from_extension() {
    assert_eq!(ConfigFormat::from_path("config.yaml"), ConfigFormat::Yaml);
    assert_eq!(ConfigFormat::from_path("config.yml"), ConfigFormat::Yaml);
    assert_eq!(ConfigFormat::from_path("config.toml"), ConfigFormat::Toml);
    assert_eq!(ConfigFormat::from_path("config.json"), ConfigFormat::Json);
    assert_eq!(ConfigFormat::from_path("config.json5"), ConfigFormat::Json);
    assert_eq!(
        ConfigFormat::from_path("config.unknown"),
        ConfigFormat::Yaml
    );
}

#[test]
fn template_targets_use_schema_default_includes_when_source_has_none() {
    let root = temp_dir_path("default-template-config");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.yaml");
    fs::write(&output_path, "#include: []\n").unwrap();

    let targets =
        template_targets_for_paths::<TestConfig>(root.join("config.yaml"), &output_path).unwrap();

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].path, output_path);
    assert!(targets[0].content.contains("\"config/server.yaml\""));
    assert_eq!(targets[1].path, root.join("config").join("server.yaml"));

    let _ = fs::remove_dir_all(root);
}

fn temp_dir_path(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rust-config-tree-config-{name}-{}-{now}",
        std::process::id()
    ))
}
