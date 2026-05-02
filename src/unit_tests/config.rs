use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;
use figment::Profile;

use super::*;

static DOTENV_TEST_LOCK: Mutex<()> = Mutex::new(());

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
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct DotenvConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(env = "RUST_CONFIG_TREE_DOTENV_MODE", default = "paper")]
    mode: String,
}

impl ConfigSchema for DotenvConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct EnvMappedConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(nested)]
    database: EnvMappedDatabaseConfig,
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct EnvMappedDatabaseConfig {
    #[config(env = "APP_DATABASE_POOL_SIZE", default = 16)]
    pool_size: u32,
}

impl ConfigSchema for EnvMappedConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct RenderedTemplateConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "root")]
    root_value: String,
    #[config(nested)]
    branch: RenderedBranchConfig,
    #[config(nested)]
    outer: RenderedOuterConfig,
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct RenderedBranchConfig {
    #[config(default = 42)]
    leaf: u16,
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct RenderedOuterConfig {
    #[config(default = true)]
    enabled: bool,
    #[config(nested)]
    inner: RenderedInnerConfig,
}

#[derive(Debug, Config)]
#[allow(dead_code)]
struct RenderedInnerConfig {
    #[config(default = "value")]
    value: String,
}

impl ConfigSchema for RenderedTemplateConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["branch"] => Some(PathBuf::from("config/custom-branch.yaml")),
            _ => None,
        }
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
fn load_config_loads_dotenv_from_config_ancestors() {
    let _guard = DOTENV_TEST_LOCK.lock().unwrap();
    unsafe {
        std::env::remove_var("RUST_CONFIG_TREE_DOTENV_MODE");
    }

    let root = temp_dir_path("load-dotenv");
    fs::create_dir_all(root.join("config")).unwrap();
    fs::write(root.join(".env"), "RUST_CONFIG_TREE_DOTENV_MODE=shadow\n").unwrap();
    fs::write(root.join("config").join("app.yaml"), "").unwrap();

    let config = load_config::<DotenvConfig>(root.join("config").join("app.yaml")).unwrap();

    assert_eq!(config.mode, "shadow");

    unsafe {
        std::env::remove_var("RUST_CONFIG_TREE_DOTENV_MODE");
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_config_preserves_environment_over_dotenv() {
    let _guard = DOTENV_TEST_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("RUST_CONFIG_TREE_DOTENV_MODE", "process");
    }

    let root = temp_dir_path("preserve-env-over-dotenv");
    fs::create_dir_all(root.join("config")).unwrap();
    fs::write(root.join(".env"), "RUST_CONFIG_TREE_DOTENV_MODE=dotenv\n").unwrap();
    fs::write(root.join("config").join("app.yaml"), "").unwrap();

    let config = load_config::<DotenvConfig>(root.join("config").join("app.yaml")).unwrap();

    assert_eq!(config.mode, "process");

    unsafe {
        std::env::remove_var("RUST_CONFIG_TREE_DOTENV_MODE");
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn load_config_maps_confique_env_names_without_splitting_underscores() {
    let _guard = DOTENV_TEST_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("APP_DATABASE_POOL_SIZE", "64");
    }

    let root = temp_dir_path("confique-env-provider");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("config.yaml"),
        concat!("database:\n", "  pool_size: 32\n",),
    )
    .unwrap();

    let (config, figment) =
        load_config_with_figment::<EnvMappedConfig>(root.join("config.yaml")).unwrap();

    assert_eq!(config.database.pool_size, 64);

    let metadata = figment.find_metadata("database.pool_size").unwrap();
    assert_eq!(
        metadata.interpolate(&Profile::Default, &["database", "pool_size"]),
        "APP_DATABASE_POOL_SIZE",
    );

    unsafe {
        std::env::remove_var("APP_DATABASE_POOL_SIZE");
    }
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

#[test]
fn template_targets_auto_split_nested_schema_sections() {
    let root = temp_dir_path("rendered-template-config");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.yaml");

    let targets = template_targets_for_paths::<RenderedTemplateConfig>(
        root.join("config.yaml"),
        &output_path,
    )
    .unwrap();

    assert_eq!(targets.len(), 4);
    assert_eq!(targets[0].path, output_path);
    assert!(targets[0].content.contains("\"config/custom-branch.yaml\""));
    assert!(targets[0].content.contains("\"config/outer.yaml\""));
    assert!(targets[0].content.contains("root_value"));
    assert!(!targets[0].content.contains("branch:"));
    assert!(!targets[0].content.contains("outer:"));

    assert_eq!(
        targets[1].path,
        root.join("config").join("custom-branch.yaml")
    );
    assert!(targets[1].content.contains("branch:"));
    assert!(targets[1].content.contains("leaf: 42"));
    assert!(!targets[1].content.contains("root_value"));
    assert!(!targets[1].content.contains("outer:"));

    assert_eq!(targets[2].path, root.join("config").join("outer.yaml"));
    assert!(targets[2].content.contains("\"outer/inner.yaml\""));
    assert!(targets[2].content.contains("outer:"));
    assert!(targets[2].content.contains("enabled: true"));
    assert!(!targets[2].content.contains("inner:"));
    assert!(!targets[2].content.contains("branch:"));

    assert_eq!(
        targets[3].path,
        root.join("config").join("outer").join("inner.yaml")
    );
    assert!(targets[3].content.contains("outer:"));
    assert!(targets[3].content.contains("inner:"));
    assert!(targets[3].content.contains("value: value"));
    assert!(!targets[3].content.contains("enabled"));

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
