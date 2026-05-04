//! File output helpers shared by schema and template generation.
//!
//! Public generation APIs return target lists for inspection and use this
//! module only when they need to write those targets to disk.

use std::{
    any::type_name,
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
/// - `output`: Optional user-provided output path. When omitted, the root
///   config type name is converted to
///   `config/<root_config_name>/<root_config_name>.example.yaml`.
///   When provided, only the file name is used and it is written under the same
///   root config directory.
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
pub(crate) fn resolve_config_template_output<S>(output: Option<PathBuf>) -> ConfigResult<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let output = output
        .as_deref()
        .and_then(Path::file_name)
        .map(PathBuf::from)
        .map(|file_name| default_config_output_dir::<S>().join(file_name))
        .unwrap_or_else(default_config_template_output::<S>);
    let output = current_dir.join(output);

    Ok(normalize_lexical(output))
}

/// Returns the default config-template output path for a root config type.
///
/// The file stem is the root config structure name converted to snake_case.
///
/// # Type Parameters
///
/// - `S`: Root config type used to derive the generated target name.
///
/// # Returns
///
/// Returns `config/<root_config_name>/<root_config_name>.example.yaml`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn default_config_template_output<S>() -> PathBuf {
    let target_name = root_config_target_name::<S>();
    default_config_output_dir::<S>().join(format!("{target_name}.example.yaml"))
}

/// Returns the default root JSON Schema output path for a root config type.
///
/// The file stem is the root config structure name converted to snake_case.
///
/// # Type Parameters
///
/// - `S`: Root config type used to derive the generated target name.
///
/// # Returns
///
/// Returns `config/<root_config_name>/<root_config_name>.schema.json`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn default_config_schema_output<S>() -> PathBuf {
    let target_name = root_config_target_name::<S>();
    default_config_output_dir::<S>().join(format!("{target_name}.schema.json"))
}

/// Returns the default config output directory for a root config type.
///
/// # Type Parameters
///
/// - `S`: Root config type used to derive the generated directory name.
///
/// # Returns
///
/// Returns `config/<root_config_name>`.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn default_config_output_dir<S>() -> PathBuf {
    PathBuf::from("config").join(root_config_target_name::<S>())
}

/// Returns the default generated target stem for a root config type.
///
/// # Type Parameters
///
/// - `S`: Root config type.
///
/// # Returns
///
/// Returns the final type segment converted to snake_case.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
pub(crate) fn root_config_target_name<S>() -> String {
    type_segment_to_snake_case(last_type_segment(type_name::<S>()))
}

/// Returns the final Rust type-name segment without generic parameters.
///
/// # Arguments
///
/// - `name`: Fully qualified Rust type name.
///
/// # Returns
///
/// Returns the final `::` segment, stripped at the first `<` when present.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn last_type_segment(name: &str) -> &str {
    name.rsplit("::")
        .next()
        .unwrap_or(name)
        .split('<')
        .next()
        .unwrap_or("config")
}

/// Converts a Rust type-name segment to snake_case for generated file names.
///
/// # Arguments
///
/// - `name`: Rust type-name segment.
///
/// # Returns
///
/// Returns a lowercase snake_case string, or `config` when the name is empty.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
fn type_segment_to_snake_case(name: &str) -> String {
    let chars = name.chars().collect::<Vec<_>>();
    let mut output = String::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_uppercase() {
            let previous = index.checked_sub(1).and_then(|index| chars.get(index));
            let next = chars.get(index + 1);
            let starts_word = previous.is_some_and(|previous| {
                previous.is_ascii_lowercase()
                    || previous.is_ascii_digit()
                    || (previous.is_ascii_uppercase()
                        && next.is_some_and(|next| next.is_ascii_lowercase()))
            });

            if starts_word && !output.ends_with('_') {
                output.push('_');
            }

            output.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
        } else if !output.ends_with('_') {
            output.push('_');
        }
    }

    let output = output.trim_matches('_');
    if output.is_empty() {
        "config".to_owned()
    } else {
        output.to_owned()
    }
}
