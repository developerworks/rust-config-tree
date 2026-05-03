//! Embeds the reusable `config-*` subcommands in an application CLI.

use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};
use confique::Config;
use rust_config_tree::{ConfigCommand, ConfigSchema, handle_config_command, load_config};
use schemars::JsonSchema;

#[derive(Debug, Parser)]
#[command(name = "config-commands")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run,

    /// Flatten the crate-provided config commands into this example CLI.
    #[command(flatten)]
    Config(ConfigCommand),
}

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "demo")]
    mode: String,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    bind: String,

    #[config(default = 8080)]
    port: u16,
}

/// Exposes the example's include list to the config command handlers.
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

/// Parses the example CLI and dispatches either app run or config commands.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let config_path = match &cli.config {
        Some(path) => path.clone(),
        None => write_demo_config()?,
    };

    match cli.command.unwrap_or(Command::Run) {
        Command::Run => {
            let config = load_config::<AppConfig>(&config_path)?;
            println!("config path: {}", config_path.display());
            println!("include count: {}", config.include.len());
            println!("mode: {}", config.mode);
            println!("server bind: {}", config.server.bind);
            println!("server port: {}", config.server.port);
        }
        Command::Config(command) => {
            handle_config_command::<Cli, AppConfig>(command, &config_path)?;
        }
    }

    Ok(())
}

/// Creates a minimal config file used when `--config` is omitted.
fn write_demo_config() -> io::Result<PathBuf> {
    let dir = temp_example_dir("config-commands")?;
    let root_config = dir.join("config.yaml");

    fs::write(
        &root_config,
        r#"
mode: local
server:
  bind: 0.0.0.0
  port: 3000
"#
        .trim_start(),
    )?;

    Ok(root_config)
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
