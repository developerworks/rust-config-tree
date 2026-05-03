use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;

/// Verifies recursive include traversal loads every reachable file.
#[test]
fn load_config_tree_recursively_loads_include_tree() {
    let root = temp_dir_path("load-tree");
    fs::create_dir_all(root.join("config").join("trading")).unwrap();
    fs::write(
        root.join("config.yaml"),
        "root\ninclude: config/trading.yaml\n",
    )
    .unwrap();
    fs::write(
        root.join("config").join("trading.yaml"),
        "trading\ninclude: trading/server.yaml\ninclude: trading/logging.yaml\n",
    )
    .unwrap();
    fs::write(
        root.join("config").join("trading").join("server.yaml"),
        "server\n",
    )
    .unwrap();
    fs::write(
        root.join("config").join("trading").join("logging.yaml"),
        "logging\n",
    )
    .unwrap();

    let tree = load_config_tree(root.join("config.yaml"), read_config).unwrap();

    let values = tree
        .nodes()
        .iter()
        .map(|node| node.value().lines().next().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(values, ["root", "trading", "server", "logging"]);

    let _ = fs::remove_dir_all(root);
}

/// Verifies sibling include traversal can be reversed.
#[test]
fn config_tree_options_can_reverse_include_order() {
    let root = temp_dir_path("reverse-tree");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("config.yaml"),
        "root\ninclude: first.yaml\ninclude: second.yaml\n",
    )
    .unwrap();
    fs::write(root.join("first.yaml"), "first\n").unwrap();
    fs::write(root.join("second.yaml"), "second\n").unwrap();

    let tree = ConfigTreeOptions::default()
        .include_order(IncludeOrder::Reverse)
        .load(root.join("config.yaml"), read_config)
        .unwrap();

    let values = tree.into_values();
    assert_eq!(
        values,
        [
            "root\ninclude: first.yaml\ninclude: second.yaml\n",
            "second\n",
            "first\n"
        ]
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies recursive include cycles are rejected.
#[test]
fn load_config_tree_rejects_recursive_include_cycle() {
    let root = temp_dir_path("cycle");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("config.yaml"), "include: child.yaml\n").unwrap();
    fs::write(root.join("child.yaml"), "include: config.yaml\n").unwrap();

    let err = load_config_tree(root.join("config.yaml"), read_config).unwrap_err();

    assert!(matches!(err, ConfigTreeError::IncludeCycle { .. }));
    assert!(err.to_string().contains("recursive config include cycle"));

    let _ = fs::remove_dir_all(root);
}

/// Verifies repeated include targets are loaded only once.
#[test]
fn load_config_tree_skips_previously_loaded_files() {
    let root = temp_dir_path("dedupe");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("config.yaml"),
        "root\ninclude: shared.yaml\ninclude: nested.yaml\n",
    )
    .unwrap();
    fs::write(root.join("nested.yaml"), "nested\ninclude: shared.yaml\n").unwrap();
    fs::write(root.join("shared.yaml"), "shared\n").unwrap();

    let tree = load_config_tree(root.join("config.yaml"), read_config).unwrap();

    assert_eq!(tree.nodes().len(), 3);

    let _ = fs::remove_dir_all(root);
}

/// Verifies loader errors are wrapped with the failing path.
#[test]
fn load_config_tree_wraps_loader_errors_with_path() {
    let path = PathBuf::from("/tmp/missing-config-tree-test.yaml");

    let err = load_config_tree(path.clone(), read_config).unwrap_err();

    assert!(matches!(err, ConfigTreeError::Load { .. }));
    assert!(err.to_string().contains(&path.display().to_string()));
}

/// Verifies `ConfigSource` accessors and ownership decomposition.
#[test]
fn config_source_exposes_parts() {
    let source = ConfigSource::new("value", vec![PathBuf::from("child.yaml")]);

    assert_eq!(source.value(), &"value");
    assert_eq!(source.includes(), &[PathBuf::from("child.yaml")]);
    assert_eq!(
        source.into_parts(),
        ("value", vec![PathBuf::from("child.yaml")])
    );
}

/// Verifies `ConfigNode` accessors and ownership decomposition.
#[test]
fn config_node_exposes_fields() {
    let tree = ConfigTree {
        nodes: vec![ConfigNode {
            path: PathBuf::from("/tmp/config.yaml"),
            value: "value",
            includes: vec![PathBuf::from("child.yaml")],
        }],
    };

    let node = &tree.nodes()[0];
    assert_eq!(node.path(), Path::new("/tmp/config.yaml"));
    assert_eq!(node.value(), &"value");
    assert_eq!(node.includes(), &[PathBuf::from("child.yaml")]);
    assert_eq!(tree.into_nodes()[0].clone().into_value(), "value");
}

/// Loads the line-based fixture config format used by tree tests.
fn read_config(path: &Path) -> io::Result<ConfigSource<String>> {
    let content = fs::read_to_string(path)?;
    let includes = parse_includes(&content);
    Ok(ConfigSource::new(content, includes))
}

/// Extracts include lines from the tree test fixture format.
fn parse_includes(content: &str) -> Vec<PathBuf> {
    content
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect()
}

/// Builds a unique temporary directory path for tree tests.
fn temp_dir_path(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rust-config-tree-{name}-{}-{now}",
        std::process::id()
    ))
}
