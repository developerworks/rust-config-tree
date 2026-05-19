//! CLI binary for the `rust-config-tree` crate.
//!
//! Embeds the reusable `config-*` subcommands through [`ConfigCommand`]
//! flatten and dispatches through [`handle_config_command`].
//!
//! # Installation
//!
//! ```sh
//! cargo install rust-config-tree
//! ```
//!
//! # Usage
//!
//! ```sh
//! rust-config-tree generate-template --type my_crate::config::AppConfig
//! rust-config-tree generate-schema
//! rust-config-tree validate-config --config config.yaml
//! ```

use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use confique::Config;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command};
use schemars::JsonSchema;

#[derive(Debug, Parser)]
#[command(name = "rust-config-tree", version, about)]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(flatten)]
    Config(ConfigCommand),
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "demo")]
    mode: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Config(command)) => {
            let config_path = match &cli.config {
                Some(path) => path.clone(),
                None => write_demo_config()?,
            };
            handle_config_command::<Cli, AppConfig>(command, &config_path)?;
        }
        None => {
            let config_path = match &cli.config {
                Some(path) => path.clone(),
                None => write_demo_config()?,
            };
            handle_config_command::<Cli, AppConfig>(
                ConfigCommand::GenerateTemplate {
                    output: None,
                    schema: None,
                    r#type: "unknown".to_owned(),
                },
                &config_path,
            )?;
        }
    }

    Ok(())
}

fn write_demo_config() -> io::Result<PathBuf> {
    let dir = temp_example_dir("rust-config-tree")?;
    let root_config = dir.join("config.yaml");

    fs::write(
        &root_config,
        r#"
mode: local
"#
        .trim_start(),
    )?;

    Ok(root_config)
}

fn temp_example_dir(name: &str) -> io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust-config-tree-{name}-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
