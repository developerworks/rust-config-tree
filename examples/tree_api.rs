use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rust_config_tree::{ConfigSource, ConfigTreeOptions, IncludeOrder, load_config_tree};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root_config = write_demo_tree()?;
    let tree = load_config_tree(&root_config, load_source)?;

    println!("declared order:");
    for node in tree.nodes() {
        println!("{} -> {:?}", node.path().display(), node.includes());
    }

    let reverse_tree = ConfigTreeOptions::default()
        .include_order(IncludeOrder::Reverse)
        .load(&root_config, load_source)?;

    println!("reverse sibling order:");
    for value in reverse_tree.into_values() {
        println!("{}", value.lines().next().unwrap_or_default());
    }

    Ok(())
}

fn load_source(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = content
        .lines()
        .filter_map(|line| line.strip_prefix("include="))
        .map(PathBuf::from)
        .collect();

    Ok(ConfigSource::new(content, includes))
}

fn write_demo_tree() -> io::Result<PathBuf> {
    let dir = temp_example_dir("tree-api")?;
    let config_dir = dir.join("config");
    fs::create_dir_all(&config_dir)?;

    let root_config = dir.join("root.conf");
    fs::write(
        &root_config,
        "root\ninclude=config/database.conf\ninclude=config/server.conf\n",
    )?;
    fs::write(config_dir.join("database.conf"), "database\npool_size=16\n")?;
    fs::write(config_dir.join("server.conf"), "server\nport=3000\n")?;

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
