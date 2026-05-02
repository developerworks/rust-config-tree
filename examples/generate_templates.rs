use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use confique::Config;
use rust_config_tree::{ConfigSchema, write_config_schemas, write_config_templates_with_schema};
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct AppConfig {
    #[config(default = [])]
    include: Vec<PathBuf>,

    #[config(nested)]
    server: ServerConfig,

    #[config(nested)]
    database: DatabaseConfig,

    #[config(nested)]
    log: LogConfig,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct ServerConfig {
    /// HTTP bind address.
    #[config(default = "127.0.0.1")]
    #[config(env = "APP_SERVER_BIND")]
    bind: String,

    /// HTTP listen port.
    #[config(default = 8080)]
    #[config(env = "APP_SERVER_PORT")]
    port: u16,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct DatabaseConfig {
    /// Database URL.
    #[config(env = "APP_DATABASE_URL")]
    url: String,

    /// Database connection pool size.
    #[config(default = 16)]
    #[config(env = "APP_DATABASE_POOL_SIZE")]
    pool_size: u32,
}

#[derive(Debug, Config, JsonSchema)]
#[allow(dead_code)]
struct LogConfig {
    /// Log level.
    #[config(default = "info")]
    #[config(env = "APP_LOG_LEVEL")]
    level: String,
}

impl ConfigSchema for AppConfig {
    fn include_paths(layer: &<Self as Config>::Layer) -> Vec<PathBuf> {
        layer.include.clone().unwrap_or_default()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir = temp_example_dir("generate-templates")?;
    let config_path = dir.join("config.yaml");
    let schema_path = dir.join("schemas").join("myapp.schema.json");

    write_config_schemas::<AppConfig>(&schema_path)?;
    for file_name in [
        "config.example.toml",
        "config.example.yaml",
        "config.example.json",
    ] {
        write_config_templates_with_schema::<AppConfig>(
            &config_path,
            dir.join(file_name),
            &schema_path,
        )?;
    }

    println!("schema: {}", schema_path.display());
    for path in generated_files(&dir)? {
        println!("{}", path.display());
    }

    Ok(())
}

fn generated_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }

    Ok(())
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
