use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;

/// Verifies include recursion mirrors source paths into output paths.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn collect_template_targets_recursively_mirrors_output_tree() {
    let root = temp_dir_path("template-targets");
    let config_dir = root.join("config");
    let output_path = root.join("examples").join("config.example.yaml");
    fs::create_dir_all(config_dir.join("trading")).unwrap();
    fs::write(
        root.join("config.yaml"),
        "include: config/trading.yaml\ninclude: config/logging.toml\n",
    )
    .unwrap();
    fs::write(
        config_dir.join("trading.yaml"),
        "include: trading/server.yaml\n",
    )
    .unwrap();
    fs::write(config_dir.join("trading").join("server.yaml"), "").unwrap();

    let targets =
        collect_template_targets(root.join("config.yaml"), &output_path, read_includes).unwrap();

    assert_eq!(targets.len(), 4);
    assert_eq!(targets[0].target_path(), output_path);
    assert_eq!(
        targets[1].target_path(),
        root.join("examples").join("config").join("trading.yaml")
    );
    assert_eq!(
        targets[2].target_path(),
        root.join("examples")
            .join("config")
            .join("trading")
            .join("server.yaml")
    );
    assert_eq!(
        targets[3].target_path(),
        root.join("examples").join("config").join("logging.toml")
    );
    assert_eq!(
        targets[1].include_paths(),
        &[PathBuf::from("trading/server.yaml")]
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies template source selection preference order.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn select_template_source_prefers_config_then_existing_output_then_output() {
    let root = temp_dir_path("template-source");
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.yaml");
    let output_path = root.join("config.example.yaml");

    assert_eq!(
        select_template_source(&config_path, &output_path),
        output_path
    );

    fs::write(&output_path, "").unwrap();
    assert_eq!(
        select_template_source(&config_path, &output_path),
        output_path
    );

    fs::write(&config_path, "").unwrap();
    assert_eq!(
        select_template_source(&config_path, &output_path),
        config_path
    );

    let _ = fs::remove_dir_all(root);
}

/// Verifies template target ownership decomposition.
///
/// # Arguments
///
/// This test has no arguments.
///
/// # Returns
///
/// Returns no value; failed assertions panic.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
#[test]
fn template_target_into_parts_returns_owned_values() {
    let target = TemplateTarget {
        source_path: PathBuf::from("/tmp/config.yaml"),
        target_path: PathBuf::from("/tmp/config.example.yaml"),
        include_paths: vec![PathBuf::from("child.yaml")],
    };

    assert_eq!(
        target.into_parts(),
        (
            PathBuf::from("/tmp/config.yaml"),
            PathBuf::from("/tmp/config.example.yaml"),
            vec![PathBuf::from("child.yaml")]
        )
    );
}

/// Reads test include lines from the custom fixture format.
///
/// # Arguments
///
/// - `path`: Fixture path to read.
///
/// # Returns
///
/// Returns include paths declared by `path`, or an empty list when missing.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn read_includes(path: &Path) -> io::Result<Vec<PathBuf>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    Ok(fs::read_to_string(path)?
        .lines()
        .filter_map(|line| line.strip_prefix("include: "))
        .map(PathBuf::from)
        .collect())
}

/// Builds a unique temporary directory path for template tests.
///
/// # Arguments
///
/// - `name`: Stable test-specific name segment.
///
/// # Returns
///
/// Returns a temporary directory path that includes the process id and time.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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
