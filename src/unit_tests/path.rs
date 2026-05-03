use std::path::{Path, PathBuf};

use super::*;

/// Verifies lexical normalization removes `.` and `..` path components.
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
fn normalize_lexical_removes_current_dir_and_parent_segments() {
    assert_eq!(
        normalize_lexical("/tmp/config/../config.yaml"),
        PathBuf::from("/tmp/config.yaml")
    );
    assert_eq!(
        normalize_lexical("config/./trading/../server.yaml"),
        PathBuf::from("config/server.yaml")
    );
}

/// Verifies relative includes resolve from their declaring file.
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
fn resolve_include_path_resolves_relative_paths_from_parent_file() {
    assert_eq!(
        resolve_include_path("/app/config/trading.yaml", "trading/server.yaml"),
        PathBuf::from("/app/config/trading/server.yaml")
    );
}

/// Verifies absolute includes stay absolute after normalization.
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
fn resolve_include_path_keeps_absolute_include_paths() {
    assert_eq!(
        resolve_include_path("/app/config.yaml", "/etc/app/logging.yaml"),
        PathBuf::from("/etc/app/logging.yaml")
    );
}

/// Verifies lexical absolutization returns an absolute normalized path.
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
fn absolutize_lexical_returns_absolute_paths() {
    let path = absolutize_lexical("config/../config.yaml").unwrap();

    assert!(path.is_absolute());
    assert!(path.ends_with(Path::new("config.yaml")));
}
