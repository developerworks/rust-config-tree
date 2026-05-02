use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use confique::Config;
use figment::providers::Serialized;
use rust_config_tree::{ConfigSchema, build_config_figment, load_config_from_figment};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "cli-overrides")]
struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    server_port: Option<u16>,

    #[arg(long)]
    log_level: Option<String>,
}

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,

    #[config(nested)]
    log: LogConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    bind: String,

    #[config(default = 8080)]
    port: u16,
}

#[derive(Debug, Config)]
struct LogConfig {
    #[config(default = "info")]
    level: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

#[derive(Debug, Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<CliServerOverrides>,

    #[serde(skip_serializing_if = "Option::is_none")]
    log: Option<CliLogOverrides>,
}

#[derive(Debug, Serialize)]
struct CliServerOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

#[derive(Debug, Serialize)]
struct CliLogOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<String>,
}

impl CliOverrides {
    fn from_cli(cli: &Cli) -> Self {
        Self {
            server: cli
                .server_port
                .map(|port| CliServerOverrides { port: Some(port) }),
            log: cli
                .log_level
                .clone()
                .map(|level| CliLogOverrides { level: Some(level) }),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let config_path = match &cli.config {
        Some(path) => path.clone(),
        None => write_demo_config()?,
    };

    let figment = build_config_figment::<AppConfig>(&config_path)?
        .merge(Serialized::defaults(CliOverrides::from_cli(&cli)));
    let config = load_config_from_figment::<AppConfig>(&figment)?;

    println!("config path: {}", config_path.display());
    println!("include count: {}", config.include.len());
    println!("server bind: {}", config.server.bind);
    println!("server port: {}", config.server.port);
    println!("log level: {}", config.log.level);

    Ok(())
}

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

fn temp_example_dir(name: &str) -> io::Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust-config-tree-{name}-{nanos}"));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
