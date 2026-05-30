//! Merges CLI override values on top of the config tree before `confique`
//! validation.

use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use confique::Config;
use rust_config_tree::{
    ConfigSchema,
    cli::ConfigOverrides,
    config::{build_config_figment, load_config_from_figment},
};
use schemars::JsonSchema;

#[derive(Debug, Parser, ConfigOverrides)]
#[command(name = "cli-overrides")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    #[config_override(path = "server.port")]
    server_port: Option<u16>,

    #[arg(long)]
    #[config_override(path = "log.level")]
    log_level: Option<String>,
}

#[derive(Debug, Config, JsonSchema, ConfigSchema)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,

    #[config(nested)]
    log: LogConfig,
}

#[derive(Debug, Config, JsonSchema)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    bind: String,

    #[config(default = 8080)]
    port: u16,
}

#[derive(Debug, Config, JsonSchema)]
struct LogConfig {
    #[config(default = "info")]
    level: String,
}

/// Loads config files, merges CLI overrides, and prints the final config.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let config_path = match &cli.config {
        Some(path) => path.clone(),
        None => write_demo_config()?,
    };

    let figment = build_config_figment::<AppConfig>(&config_path)?
        // CLI overrides are merged last, so provided flags override file and
        // environment values while omitted flags disappear.
        .merge(cli.config_overrides()?);
    let config = load_config_from_figment::<AppConfig>(&figment)?;

    println!("config path: {}", config_path.display());
    println!("include count: {}", config.include.len());
    println!("server bind: {}", config.server.bind);
    println!("server port: {}", config.server.port);
    println!("log level: {}", config.log.level);

    Ok(())
}

/// Creates a minimal config file used when `--config` is omitted.
fn write_demo_config() -> io::Result<PathBuf> {
    let dir = temp_example_dir("cli-overrides")?;
    let root_config = dir.join("config.yaml");

    fs::write(
        &root_config,
        r#"
server:
  bind: 0.0.0.0
  port: 3000
log:
  level: info
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
