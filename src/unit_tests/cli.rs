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

impl ConfigSchema for TestConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

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
            schema: None,
        },
        &config_path,
    )
    .unwrap();

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

#[test]
fn upsert_managed_block_rejects_missing_end_marker() {
    let path = temp_file_path("missing-end");
    fs::write(&path, "# >>> app fish completions >>>\n").unwrap();

    let err = upsert_managed_block("app", Shell::Fish, &path, "body").unwrap_err();

    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    let _ = fs::remove_file(path);
}

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

fn temp_file_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rust-config-tree-cli-{name}-{}",
        std::process::id()
    ))
}
