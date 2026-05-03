use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use clap_complete::aot::Shell;
use confique::Config;
use schemars::JsonSchema;

use super::*;
use crate::ConfigSchema;

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
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

/// Verifies config commands can be flattened into a consumer CLI.
#[test]
fn config_command_can_be_flattened_into_a_consumer_cli() {
    let cli = DemoCli::parse_from(["demo", "config-template", "--output", "config.yaml"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigTemplate { output, schema }) => {
            assert_eq!(output, Some(PathBuf::from("config.yaml")));
            assert_eq!(schema, Some(PathBuf::from("schemas/config.schema.json")));
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the template command accepts a custom schema output path.
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
            assert_eq!(output, PathBuf::from("schemas/myapp.schema.json"));
        }
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the validation command remains available through CLI flattening.
#[test]
fn config_validate_command_is_flattened_into_consumer_cli() {
    let cli = DemoCli::parse_from(["demo", "config-validate"]);

    match cli.command {
        DemoCommand::Config(ConfigCommand::ConfigValidate) => {}
        command => panic!("unexpected command: {command:?}"),
    }
}

/// Verifies the template command writes templates and schemas for a consumer schema.
#[test]
fn handle_config_command_writes_templates_for_consumer_schema() {
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

    handle_config_command::<DemoCli, TestConfig>(
        ConfigCommand::ConfigTemplate {
            output: Some(output_path.clone()),
            schema: Some(root.join("schemas").join("config.schema.json")),
        },
        &config_path,
    )
    .unwrap();

    assert!(root.join("schemas").join("config.schema.json").exists());
    assert!(output_path.exists());
    assert!(
        fs::read_to_string(&output_path)
            .unwrap()
            .starts_with("# yaml-language-server: $schema=../schemas/config.schema.json\n\n")
    );
    assert!(
        root.join("examples")
            .join("config")
            .join("server.yaml")
            .exists()
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies the schema command writes a Draft 7 JSON Schema.
#[test]
fn handle_config_command_writes_json_schema_for_consumer_schema() {
    let root = temp_dir_path("handle-config-schema");
    fs::create_dir_all(root.join("schemas")).unwrap();
    let schema_path = root.join("schemas").join("myapp.schema.json");

    handle_config_command::<DemoCli, TestConfig>(
        ConfigCommand::JsonSchema {
            output: schema_path.clone(),
        },
        PathBuf::from("config.yaml").as_path(),
    )
    .unwrap();

    let schema = fs::read_to_string(&schema_path).unwrap();
    assert!(schema.contains("http://json-schema.org/draft-07/schema#"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies the validation command accepts a complete runtime config.
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
#[test]
fn upsert_managed_block_rejects_missing_end_marker() {
    let path = temp_file_path("missing-end");
    fs::write(&path, "# >>> app fish completions >>>\n").unwrap();

    let err = upsert_managed_block("app", Shell::Fish, &path, "body").unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    let _ = fs::remove_file(path);
}

/// Builds a unique temporary directory path for CLI tests.
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
fn temp_file_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rust-config-tree-cli-{name}-{}",
        std::process::id()
    ))
}
