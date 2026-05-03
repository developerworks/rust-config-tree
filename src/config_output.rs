//! File output helpers shared by schema and template generation.
//!
//! Public generation APIs return target lists for inspection and use this
//! module only when they need to write those targets to disk.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::normalize_lexical;

use crate::config::ConfigResult;

/// Writes one generated template file, creating parent directories first.
///
/// # Arguments
///
/// - `path`: Destination file path.
/// - `content`: Complete template content to write.
///
/// # Returns
///
/// Returns `Ok(())` after the file has been written.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn write_template(path: &Path, content: &str) -> ConfigResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// Resolves the CLI template output path to a normalized absolute path.
///
/// # Arguments
///
/// - `output`: Optional user-provided output path. When omitted,
///   `config.example.yaml` is used.
///
/// # Returns
///
/// Returns a normalized absolute output path.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn resolve_config_template_output(output: Option<PathBuf>) -> ConfigResult<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let output = output.unwrap_or_else(|| PathBuf::from("config.example.yaml"));
    let output = if output.is_absolute() {
        output
    } else {
        current_dir.join(output)
    };

    Ok(normalize_lexical(output))
}
