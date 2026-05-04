use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use clap_complete::aot::Shell;
use confique::Config;
use schemars::JsonSchema;

use super::*;
use crate::{ConfigSchema, config_output::default_config_template_output};

static CURRENT_DIR_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Parser)]
#[command(name = "demo")]
struct DemoCli {
    #[command(subcommand)]
    command: DemoCommand,
}

#[derive(Debug, Subcommand)]
enum DemoCommand {
    Run,
    #[command(flatten)]
    Config(ConfigCommand),
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct TestConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
}

/// Exposes the fixture include list to template command tests.
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

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RecorderConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
}

impl ConfigSchema for RecorderConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct EngineConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
}

impl ConfigSchema for EngineConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct RequiredConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,
    required_value: String,
}

/// Exposes the fixture include list to validation command tests.
impl ConfigSchema for RequiredConfig {
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

/// Verifies config commands can be flattened into a consumer CLI.
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
fn config_command_can_be_flattened_into_a_consumer_cli() {
    let cli = DemoCli::parse_from(["demo", "config-template", "--output", "config.yaml"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigTemplate { output, schema }) => {
            assert_eq!(output, Some(PathBuf::from("config.yaml")));
            assert_eq!(schema, None);
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies config template defaults are derived from the root config type name.
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
fn config_template_defaults_use_root_config_snake_case_name() {
    let cli = DemoCli::parse_from(["demo", "config-template"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigTemplate { output, schema }) => {
            assert_eq!(output, None);
            assert_eq!(schema, None);
        }
        command => panic!("unexpected command: {command:?}"),
    }

    let output = resolve_config_template_output::<TestConfig>(None).unwrap();

    assert_eq!(output.file_name().unwrap(), "test_config.example.yaml");
    assert_eq!(
        default_config_schema_output::<TestConfig>(),
        PathBuf::from("config/test_config/test_config.schema.json")
    );
}

/// Verifies default target names follow any root ConfigSchema structure name.
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
fn default_targets_use_any_root_config_snake_case_name() {
    assert_eq!(
        default_config_template_output::<RecorderConfig>(),
        PathBuf::from("config/recorder_config/recorder_config.example.yaml")
    );
    assert_eq!(
        default_config_schema_output::<RecorderConfig>(),
        PathBuf::from("config/recorder_config/recorder_config.schema.json")
    );
    assert_eq!(
        default_config_template_output::<EngineConfig>(),
        PathBuf::from("config/engine_config/engine_config.example.yaml")
    );
    assert_eq!(
        default_config_schema_output::<EngineConfig>(),
        PathBuf::from("config/engine_config/engine_config.schema.json")
    );
}

/// Verifies the template command accepts a custom schema output path.
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
fn config_template_command_accepts_schema_path() {
    let cli = DemoCli::parse_from([
        "demo",
        "config-template",
        "--output",
        "config.example.toml",
        "--schema",
        "schemas/myapp.schema.json",
    ]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigTemplate { output, schema }) => {
            assert_eq!(output, Some(PathBuf::from("config.example.toml")));
            assert_eq!(schema, Some(PathBuf::from("schemas/myapp.schema.json")));
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the schema command remains available through CLI flattening.
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
fn config_schema_command_is_flattened_into_consumer_cli() {
    let cli = DemoCli::parse_from([
        "demo",
        "config-schema",
        "--output",
        "schemas/myapp.schema.json",
    ]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::JsonSchema { output }) => {
            assert_eq!(output, Some(PathBuf::from("schemas/myapp.schema.json")));
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the schema command default is handled from the consumer config type.
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
fn config_schema_command_defers_default_output_to_handler() {
    let cli = DemoCli::parse_from(["demo", "config-schema"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::JsonSchema { output }) => {
            assert_eq!(output, None);
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the validation command remains available through CLI flattening.
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
fn config_validate_command_is_flattened_into_consumer_cli() {
    let cli = DemoCli::parse_from(["demo", "config-validate"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigValidate) => {}
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the uninstall completion command remains available through CLI flattening.
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
fn uninstall_completions_command_is_flattened_into_consumer_cli() {
    let cli = DemoCli::parse_from(["demo", "uninstall-completions", "zsh"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::UninstallCompletions { shell }) => {
            assert_eq!(shell, Shell::Zsh);
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the template command writes templates and schemas for a consumer schema.
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
fn handle_config_command_writes_templates_for_consumer_schema() {
    let _guard = CURRENT_DIR_LOCK.lock().unwrap();
    let current_dir = std::env::current_dir().unwrap();
    let root = temp_dir_path("handle-config-template");
    fs::create_dir_all(root.join("config")).unwrap();
    let config_path = root.join("config.yaml");
    let output_path = root.join("examples").join("config.example.yaml");
    fs::write(
        &config_path,
        concat!("include:\n", "  - config/server.yaml\n",),
    )
    .unwrap();
    fs::write(root.join("config").join("server.yaml"), "").unwrap();

    std::env::set_current_dir(&root).unwrap();
    let result = handle_config_command::<DemoCli, TestConfig>(
        ConfigCommand::ConfigTemplate {
            output: Some(output_path.clone()),
            schema: Some(PathBuf::from("schemas/config.schema.json")),
        },
        &config_path,
    );
    std::env::set_current_dir(current_dir).unwrap();
    result.unwrap();

    let expected_output = root
        .join("config")
        .join("test_config")
        .join("config.example.yaml");

    assert!(root.join("schemas").join("config.schema.json").exists());
    assert!(!output_path.exists());
    assert!(expected_output.exists());
    assert!(
        fs::read_to_string(&expected_output)
            .unwrap()
            .starts_with("# yaml-language-server: $schema=../../schemas/config.schema.json\n\n")
    );
    assert!(
        root.join("config")
            .join("test_config")
            .join("config")
            .join("server.yaml")
            .exists()
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies omitted config-template paths use the root ConfigSchema type name.
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
fn handle_config_template_defaults_to_root_config_named_targets() {
    let _guard = CURRENT_DIR_LOCK.lock().unwrap();
    let current_dir = std::env::current_dir().unwrap();
    let root = temp_dir_path("handle-config-template-defaults");
    fs::create_dir_all(&root).unwrap();
    std::env::set_current_dir(&root).unwrap();

    let result = handle_config_command::<DemoCli, RecorderConfig>(
        ConfigCommand::ConfigTemplate {
            output: None,
            schema: None,
        },
        Path::new("recorder.yaml"),
    );

    std::env::set_current_dir(current_dir).unwrap();
    result.unwrap();

    assert!(
        root.join("config")
            .join("recorder_config")
            .join("recorder_config.example.yaml")
            .exists()
    );
    assert!(
        root.join("config")
            .join("recorder_config")
            .join("recorder_config.schema.json")
            .exists()
    );

    let template = fs::read_to_string(
        root.join("config")
            .join("recorder_config")
            .join("recorder_config.example.yaml"),
    )
    .unwrap();
    assert!(
        template.starts_with("# yaml-language-server: $schema=./recorder_config.schema.json\n\n")
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies the schema command writes a Draft 7 JSON Schema.
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
fn handle_config_command_writes_json_schema_for_consumer_schema() {
    let root = temp_dir_path("handle-config-schema");
    fs::create_dir_all(root.join("schemas")).unwrap();
    let schema_path = root.join("schemas").join("myapp.schema.json");

    handle_config_command::<DemoCli, TestConfig>(
        ConfigCommand::JsonSchema {
            output: Some(schema_path.clone()),
        },
        PathBuf::from("config.yaml").as_path(),
    )
    .unwrap();

    let schema = fs::read_to_string(&schema_path).unwrap();
    assert!(schema.contains("http://json-schema.org/draft-07/schema#"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies omitted config-schema output uses the root ConfigSchema type name.
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
fn handle_config_schema_defaults_to_root_config_named_subdirectory() {
    let _guard = CURRENT_DIR_LOCK.lock().unwrap();
    let current_dir = std::env::current_dir().unwrap();
    let root = temp_dir_path("handle-config-schema-defaults");
    fs::create_dir_all(&root).unwrap();
    std::env::set_current_dir(&root).unwrap();

    let result = handle_config_command::<DemoCli, EngineConfig>(
        ConfigCommand::JsonSchema { output: None },
        PathBuf::from("config.yaml").as_path(),
    );

    std::env::set_current_dir(current_dir).unwrap();
    result.unwrap();

    let schema_path = root
        .join("config")
        .join("engine_config")
        .join("engine_config.schema.json");
    let schema = fs::read_to_string(schema_path).unwrap();
    assert!(schema.contains("http://json-schema.org/draft-07/schema#"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies the validation command accepts a complete runtime config.
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
fn handle_config_command_validates_full_runtime_config() {
    let root = temp_dir_path("handle-config-validate");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.yaml");
    fs::write(&config_path, "required_value: present\n").unwrap();

    handle_config_command::<DemoCli, RequiredConfig>(
        ConfigCommand::ConfigValidate,
        config_path.as_path(),
    )
    .unwrap();

    let _ = fs::remove_dir_all(root);
}

/// Verifies the validation command reports invalid runtime config.
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
fn handle_config_command_rejects_invalid_runtime_config() {
    let root = temp_dir_path("handle-config-validate-invalid");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.yaml");
    fs::write(&config_path, "").unwrap();

    let result = handle_config_command::<DemoCli, RequiredConfig>(
        ConfigCommand::ConfigValidate,
        config_path.as_path(),
    );

    assert!(result.is_err());

    let _ = fs::remove_dir_all(root);
}

/// Verifies shell completion setup inserts a new managed block.
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
fn upsert_managed_block_inserts_new_block() {
    let path = temp_file_path("insert");

    upsert_managed_block("app", Shell::Zsh, &path, "body\n").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(
        content,
        "# >>> app zsh completions >>>\nbody\n\n# <<< app zsh completions <<<\n"
    );
    let _ = fs::remove_file(path);
}

/// Verifies shell completion setup replaces an existing managed block.
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
fn upsert_managed_block_replaces_existing_block() {
    let path = temp_file_path("replace");
    fs::write(
        &path,
        concat!(
            "before\n\n",
            "# >>> app bash completions >>>\n",
            "old\n",
            "# <<< app bash completions <<<\n\n",
            "after\n",
        ),
    )
    .unwrap();

    upsert_managed_block("app", Shell::Bash, &path, "new\n").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(
        content,
        concat!(
            "before\n\n",
            "# >>> app bash completions >>>\n",
            "new\n\n",
            "# <<< app bash completions <<<\n",
            "\n",
            "after\n",
        )
    );
    let _ = fs::remove_file(path);
}

/// Verifies malformed managed blocks are rejected instead of duplicated.
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
fn upsert_managed_block_rejects_missing_end_marker() {
    let path = temp_file_path("missing-end");
    fs::write(&path, "# >>> app fish completions >>>\n").unwrap();

    let err = upsert_managed_block("app", Shell::Fish, &path, "body").unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    let _ = fs::remove_file(path);
}

/// Verifies zsh startup setup uses one directory-level managed block.
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
fn zsh_rc_block_uses_shared_completion_marker() {
    let target = ShellInstallTarget::new(Shell::Zsh, PathBuf::from("/tmp/home").as_path()).unwrap();
    let body = target
        .rc_block_body(
            PathBuf::from("/tmp/home/.zsh/completions/_demo").as_path(),
            PathBuf::from("/tmp/home/.zsh/completions").as_path(),
        )
        .unwrap();

    assert_eq!(
        target.managed_block_name("demo"),
        "rust-config-tree".to_owned()
    );
    assert_eq!(
        body,
        concat!(
            "typeset -U fpath\n",
            "fpath=(\"/tmp/home/.zsh/completions\" $fpath)\n",
            "\n",
            "autoload -Uz compinit\n",
            "compinit\n",
        )
    );
}

/// Verifies managed startup files are backed up before rewriting existing content.
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
fn upsert_managed_block_backs_up_existing_startup_file() {
    let path = temp_file_path("backup-upsert");
    let _ = fs::remove_file(&path);
    fs::write(&path, "before\n").unwrap();

    upsert_managed_block("demo", Shell::Bash, &path, "body\n").unwrap();

    let backups = backup_paths_for(&path);
    assert_eq!(backups.len(), 1);
    assert!(
        backups[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("rust-config-tree-cli-backup-upsert-")
    );
    assert!(backups[0].to_string_lossy().contains(".backup.by.demo."));
    assert_eq!(fs::read_to_string(&backups[0]).unwrap(), "before\n");

    let _ = fs::remove_file(&path);
    for backup in backups {
        let _ = fs::remove_file(backup);
    }
}

/// Verifies removing a managed block preserves the original startup file first.
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
fn remove_managed_block_removes_block_and_backs_up_existing_startup_file() {
    let path = temp_file_path("backup-remove");
    let _ = fs::remove_file(&path);
    fs::write(
        &path,
        concat!(
            "before\n\n",
            "# >>> demo bash completions >>>\n",
            "body\n",
            "# <<< demo bash completions <<<\n\n",
            "after\n",
        ),
    )
    .unwrap();

    remove_managed_block("demo", Shell::Bash, &path).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "before\n\nafter\n");
    let backups = backup_paths_for(&path);
    assert_eq!(backups.len(), 1);
    assert!(backups[0].to_string_lossy().contains(".backup.by.demo."));
    assert!(
        fs::read_to_string(&backups[0])
            .unwrap()
            .contains("# >>> demo bash completions >>>")
    );

    let _ = fs::remove_file(&path);
    for backup in backups {
        let _ = fs::remove_file(backup);
    }
}

/// Builds a unique temporary directory path for CLI tests.
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
        "rust-config-tree-cli-{name}-{}-{now}",
        std::process::id()
    ))
}

/// Builds a stable temporary file path for shell startup file tests.
///
/// # Arguments
///
/// - `name`: Stable test-specific name segment.
///
/// # Returns
///
/// Returns a temporary file path that includes the process id.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn temp_file_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rust-config-tree-cli-{name}-{}",
        std::process::id()
    ))
}

/// Lists backup files created for a temporary startup file path.
///
/// # Arguments
///
/// - `path`: Startup file path whose backup siblings should be returned.
///
/// # Returns
///
/// Returns sorted backup paths next to `path`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn backup_paths_for(path: &Path) -> Vec<PathBuf> {
    let prefix = format!("{}.backup.by.", path.file_name().unwrap().to_string_lossy());
    let mut paths = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|entry_path| {
            entry_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}
