use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;
use figment::Profile;
use schemars::JsonSchema;

use super::*;

static DOTENV_TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct TestConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "paper")]
    mode: String,
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: TestServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct TestServerConfig {
    #[config(default = 8080)]
    port: u16,
}

/// Exposes fixture includes for runtime, template, and schema tests.
impl ConfigSchema for TestConfig {
    /// Returns include paths declared by the fixture layer.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded fixture layer.
    ///
    /// # Returns
    ///
    /// Returns include paths or an empty list when omitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
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

/// Exposes fixture includes for dotenv loading tests.
impl ConfigSchema for DotenvConfig {
    /// Returns include paths declared by the dotenv fixture layer.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded fixture layer.
    ///
    /// # Returns
    ///
    /// Returns include paths or an empty list when omitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
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

/// Exposes fixture includes for environment mapping tests.
impl ConfigSchema for EnvMappedConfig {
    /// Returns include paths declared by the env-mapping fixture layer.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded fixture layer.
    ///
    /// # Returns
    ///
    /// Returns include paths or an empty list when omitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RenderedTemplateConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(default = "root")]
    root_value: String,
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    branch: RenderedBranchConfig,
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    outer: RenderedOuterConfig,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RenderedBranchConfig {
    #[config(default = 42)]
    leaf: u16,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RenderedOuterConfig {
    #[config(default = true)]
    enabled: bool,
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    inner: RenderedInnerConfig,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RenderedInnerConfig {
    #[config(default = "value")]
    value: String,
}

/// Exposes fixture includes and a custom section path for split templates.
impl ConfigSchema for RenderedTemplateConfig {
    /// Returns include paths declared by the split-template fixture layer.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded fixture layer.
    ///
    /// # Returns
    ///
    /// Returns include paths or an empty list when omitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }

    /// Returns a custom template path for the `branch` split section.
    ///
    /// # Arguments
    ///
    /// - `section_path`: Section path requested by template generation.
    ///
    /// # Returns
    ///
    /// Returns a custom path for `branch`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn template_path_for_section(section_path: &[&str]) -> Option<PathBuf> {
        match section_path {
            ["branch"] => Some(PathBuf::from("config/custom-branch.yaml")),
            _ => None,
        }
    }
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct InlineTemplateConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    #[config(nested)]
    inline: InlineSectionConfig,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct InlineSectionConfig {
    #[config(default = "inline")]
    value: String,
}

/// Exposes fixture includes for unsplit nested section tests.
impl ConfigSchema for InlineTemplateConfig {
    /// Returns include paths declared by the inline fixture layer.
    ///
    /// # Arguments
    ///
    /// - `layer`: Partially loaded fixture layer.
    ///
    /// # Returns
    ///
    /// Returns include paths or an empty list when omitted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

/// Verifies a root config and included child load into the final schema.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies `.env` files are discovered from config ancestors.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies process environment values override `.env` values.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies exact `confique` env names are mapped without underscore splitting.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies template targets recurse through includes and render content.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies template writers create missing parent directories.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies root JSON Schema output uses Draft 7 and relaxed required fields.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn write_config_schema_writes_draft7_json_schema() {
    let root = temp_dir_path("write-schema");
    fs::create_dir_all(&root).unwrap();
    let schema_path = root.join("schemas").join("myapp.schema.json");

    write_config_schema::<TestConfig>(&schema_path).unwrap();

    let schema = fs::read_to_string(&schema_path).unwrap();
    assert!(schema.contains("http://json-schema.org/draft-07/schema#"));
    assert!(schema.contains("\"server\""));
    assert!(!schema.contains("\"required\""));
    assert!(schema.ends_with('\n'));

    let _ = fs::remove_dir_all(root);
}

/// Verifies schema generation writes root and split section schemas.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn write_config_schemas_writes_root_and_section_schemas() {
    let root = temp_dir_path("write-section-schemas");
    fs::create_dir_all(root.join("schemas")).unwrap();
    let schema_path = root.join("schemas").join("myapp.schema.json");

    write_config_schemas::<TestConfig>(&schema_path).unwrap();

    let root_schema = fs::read_to_string(&schema_path).unwrap();
    assert!(root_schema.contains("\"mode\""));
    assert!(!root_schema.contains("\"server\""));
    assert!(!root_schema.contains("\"port\""));
    assert!(!root_schema.contains("x-tree-split"));
    assert!(!root_schema.contains("\"definitions\""));
    assert!(!root_schema.contains("\"required\""));

    let server_schema_path = root.join("schemas").join("server.schema.json");
    let server_schema = fs::read_to_string(server_schema_path).unwrap();
    assert!(server_schema.contains("http://json-schema.org/draft-07/schema#"));
    assert!(server_schema.contains("\"port\""));
    assert!(!server_schema.contains("\"mode\""));
    assert!(!server_schema.contains("x-tree-split"));
    assert!(!server_schema.contains("\"required\""));

    let _ = fs::remove_dir_all(root);
}

/// Verifies nested split schemas keep only their own completion fields.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn write_config_schemas_keeps_section_completion_in_section_schemas() {
    let root = temp_dir_path("write-nested-section-schemas");
    fs::create_dir_all(root.join("schemas")).unwrap();
    let schema_path = root.join("schemas").join("myapp.schema.json");

    write_config_schemas::<RenderedTemplateConfig>(&schema_path).unwrap();

    let root_schema = fs::read_to_string(&schema_path).unwrap();
    assert!(root_schema.contains("\"root_value\""));
    assert!(!root_schema.contains("\"branch\""));
    assert!(!root_schema.contains("\"outer\""));

    let outer_schema = fs::read_to_string(root.join("schemas").join("outer.schema.json")).unwrap();
    assert!(outer_schema.contains("\"enabled\""));
    assert!(!outer_schema.contains("\"inner\""));
    assert!(!outer_schema.contains("\"value\""));

    let inner_schema =
        fs::read_to_string(root.join("schemas").join("outer").join("inner.schema.json")).unwrap();
    assert!(inner_schema.contains("\"value\""));
    assert!(!inner_schema.contains("\"enabled\""));

    let _ = fs::remove_dir_all(root);
}

/// Verifies TOML and YAML templates receive editor schema directives.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn template_targets_with_schema_add_toml_and_yaml_directives() {
    let root = temp_dir_path("template-schema-directives");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.toml");
    let schema_path = root.join("schemas").join("myapp.schema.json");

    let targets = template_targets_for_paths_with_schema::<TestConfig>(
        root.join("config.toml"),
        &output_path,
        &schema_path,
    )
    .unwrap();

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].path, output_path);
    assert!(
        targets[0]
            .content
            .starts_with("#:schema ./schemas/myapp.schema.json\n\n")
    );
    assert!(!targets[0].content.contains("$schema"));

    assert_eq!(targets[1].path, root.join("config").join("server.yaml"));
    assert!(
        targets[1]
            .content
            .starts_with("# yaml-language-server: $schema=../schemas/server.schema.json\n\n")
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies nested YAML templates bind to matching section schemas.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn split_yaml_templates_bind_nested_section_schemas() {
    let root = temp_dir_path("nested-template-schema-directives");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.yaml");
    let schema_path = root.join("schemas").join("myapp.schema.json");

    let targets = template_targets_for_paths_with_schema::<RenderedTemplateConfig>(
        root.join("config.yaml"),
        &output_path,
        &schema_path,
    )
    .unwrap();

    let outer = targets
        .iter()
        .find(|target| target.path == root.join("config").join("outer.yaml"))
        .unwrap();
    assert!(
        outer
            .content
            .starts_with("# yaml-language-server: $schema=../schemas/outer.schema.json\n\n")
    );

    let inner = targets
        .iter()
        .find(|target| target.path == root.join("config").join("outer").join("inner.yaml"))
        .unwrap();
    assert!(
        inner.content.starts_with(
            "# yaml-language-server: $schema=../../schemas/outer/inner.schema.json\n\n"
        )
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies JSON templates are not modified with schema directives.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn schema_binding_keeps_json_templates_unmodified() {
    let root = temp_dir_path("json-template-schema-binding");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.json");
    let schema_path = root.join("schemas").join("myapp.schema.json");

    let targets = template_targets_for_paths_with_schema::<TestConfig>(
        root.join("config.json"),
        &output_path,
        &schema_path,
    )
    .unwrap();

    assert_eq!(targets[0].path, output_path);
    assert!(!targets[0].content.contains("$schema"));
    assert!(!targets[0].content.contains("yaml-language-server"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies config format inference for supported and unknown extensions.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies generated child includes are used when the source has none.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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

/// Verifies unmarked nested sections remain in the root template and schema.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn unmarked_nested_sections_stay_in_root_template_and_schema() {
    let root = temp_dir_path("inline-template-config");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.yaml");

    let template_targets =
        template_targets_for_paths::<InlineTemplateConfig>(root.join("config.yaml"), &output_path)
            .unwrap();

    assert_eq!(template_targets.len(), 1);
    assert_eq!(template_targets[0].path, output_path);
    assert!(template_targets[0].content.contains("inline:"));
    assert!(template_targets[0].content.contains("value: inline"));
    assert!(
        !template_targets[0]
            .content
            .contains("\"config/inline.yaml\"")
    );

    let schema_path = root.join("schemas").join("config.schema.json");
    let schema_targets =
        config_schema_targets_for_path::<InlineTemplateConfig>(&schema_path).unwrap();

    assert_eq!(schema_targets.len(), 1);
    assert_eq!(schema_targets[0].path, schema_path);
    assert!(schema_targets[0].content.contains("\"inline\""));
    assert!(schema_targets[0].content.contains("\"value\""));
    assert!(!schema_targets[0].content.contains("x-tree-split"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies missing schema-derived includes are appended to existing includes.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn template_targets_append_missing_schema_default_includes() {
    let root = temp_dir_path("append-default-template-includes");
    fs::create_dir_all(root.join("config")).unwrap();
    let output_path = root.join("config.example.yaml");
    fs::write(
        &output_path,
        concat!("include:\n", "  - config/custom-branch.yaml\n",),
    )
    .unwrap();
    fs::write(root.join("config").join("custom-branch.yaml"), "").unwrap();

    let targets = template_targets_for_paths::<RenderedTemplateConfig>(
        root.join("config.yaml"),
        &output_path,
    )
    .unwrap();

    assert_eq!(targets.len(), 4);
    assert_eq!(targets[0].path, output_path);
    assert!(targets[0].content.contains("\"config/custom-branch.yaml\""));
    assert!(targets[0].content.contains("\"config/outer.yaml\""));
    assert_eq!(
        targets[1].path,
        root.join("config").join("custom-branch.yaml")
    );
    assert_eq!(targets[2].path, root.join("config").join("outer.yaml"));
    assert_eq!(
        targets[3].path,
        root.join("config").join("outer").join("inner.yaml")
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies split markers produce section template targets automatically.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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
    assert!(!targets[1].content.contains("\nbranch:"));
    assert!(targets[1].content.contains("leaf: 42"));
    assert!(!targets[1].content.contains("root_value"));
    assert!(!targets[1].content.contains("outer:"));

    assert_eq!(targets[2].path, root.join("config").join("outer.yaml"));
    assert!(targets[2].content.contains("\"outer/inner.yaml\""));
    assert!(targets[2].content.contains("outer:"));
    assert!(!targets[2].content.contains("\nouter:"));
    assert!(targets[2].content.contains("enabled: true"));
    assert!(!targets[2].content.contains("inner:"));
    assert!(!targets[2].content.contains("branch:"));

    assert_eq!(
        targets[3].path,
        root.join("config").join("outer").join("inner.yaml")
    );
    assert!(targets[3].content.contains("outer:"));
    assert!(targets[3].content.contains("inner:"));
    assert!(!targets[3].content.contains("\nouter:"));
    assert!(!targets[3].content.contains("\n  inner:"));
    assert!(targets[3].content.contains("value: value"));
    assert!(!targets[3].content.contains("enabled"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies generated split templates can be loaded and regenerated.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn generated_split_templates_can_be_loaded_and_regenerated() {
    let root = temp_dir_path("load-generated-template-config");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("config.example.yaml");

    write_config_templates::<RenderedTemplateConfig>(root.join("config.yaml"), &output_path)
        .unwrap();

    let config = load_config::<RenderedTemplateConfig>(&output_path).unwrap();
    assert_eq!(config.root_value, "root");
    assert_eq!(config.branch.leaf, 42);
    assert!(config.outer.enabled);
    assert_eq!(config.outer.inner.value, "value");

    let targets = template_targets_for_paths::<RenderedTemplateConfig>(
        root.join("config.yaml"),
        &output_path,
    )
    .unwrap();

    assert_eq!(targets.len(), 4);

    let _ = fs::remove_dir_all(root);
}

/// Builds a unique temporary directory path for config tests.
///
/// # Arguments
///
/// - `name`: Stable test-specific name segment.
///
/// # Returns
///
/// Returns a temporary directory path that includes the process id and time.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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
