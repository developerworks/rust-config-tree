//! Loads a root YAML config plus one included child file into a `confique`
//! schema.

use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;
use rust_config_tree::{ConfigSchema, load_config};

#[derive(Debug, Config)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(default = "paper")]
    mode: String,

    #[config(nested)]
    server: ServerConfig,
}

#[derive(Debug, Config)]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    #[config(env = "APP_SERVER_BIND")]
    bind: String,

    #[config(default = 8080)]
    #[config(env = "APP_SERVER_PORT")]
    port: u16,
}

/// Exposes the example's top-level include list to the tree loader.
impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

/// Writes demo files, loads them, and prints the merged config.
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root_config = write_demo_config()?;
    let config = load_config::<AppConfig>(&root_config)?;

    println!("config path: {}", root_config.display());
    println!("include count: {}", config.include.len());
    println!("mode: {}", config.mode);
    println!("server bind: {}", config.server.bind);
    println!("server port: {}", config.server.port);

    Ok(())
}

/// Creates a root config file and one included child file for the example.
fn write_demo_config() -> io::Result<PathBuf> {
    let dir = temp_example_dir("basic-loading")?;
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir)?;

    let root_config = dir.join("config.yaml");
    fs::write(
        &root_config,
        r#"
include:
  - config/server.yaml

mode: demo
"#
        .trim_start(),
    )?;

    fs::write(
        config_dir.join("server.yaml"),
        r#"
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
