//! Low-level template target discovery.
//!
//! This module maps source config files to output template files by following
//! include paths. It does not render template content; callers provide include
//! discovery and decide how each target should be rendered.

use std::path::{Path, PathBuf};

use crate::{
    BoxError, Result, absolutize_lexical, resolve_include_path,
    tree::{TraversalState, validate_include_paths},
};

/// A source-to-output mapping for one generated config template.
///
/// The source path is used to discover includes. The target path is the output
/// file that should receive the rendered template content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTarget {
    source_path: PathBuf,
    target_path: PathBuf,
    include_paths: Vec<PathBuf>,
}

/// Accessors for generated template target metadata.
impl TemplateTarget {
    /// Returns the config source path used to discover this target's includes.
    ///
    /// # Returns
    ///
    /// Returns the source path for this template target.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Returns the output path that should receive this target's template.
    ///
    /// # Returns
    ///
    /// Returns the output path for this template target.
    pub fn target_path(&self) -> &Path {
        &self.target_path
    }

    /// Returns include paths declared by this source target.
    ///
    /// # Returns
    ///
    /// Returns the include paths declared by the target source.
    pub fn include_paths(&self) -> &[PathBuf] {
        &self.include_paths
    }

    /// Decomposes the target into its source path, target path, and include paths.
    ///
    /// # Returns
    ///
    /// Returns `(source_path, target_path, include_paths)`.
    pub fn into_parts(self) -> (PathBuf, PathBuf, Vec<PathBuf>) {
        (self.source_path, self.target_path, self.include_paths)
    }
}

/// Chooses the source file used when generating templates.
///
/// Existing config files are preferred. If the config file does not exist, an
/// existing output template is used as the source. If neither exists, the output
/// path is returned so generation can start from an empty template tree.
///
/// # Arguments
///
/// - `config_path`: Preferred config source path.
/// - `output_path`: Output template path used as the fallback source.
///
/// # Returns
///
/// Returns the path that should be used as the root template source.
pub fn select_template_source(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> PathBuf {
    let config_path = config_path.as_ref();
    let output_path = output_path.as_ref();

    if config_path.exists() {
        return config_path.to_path_buf();
    }

    if output_path.exists() {
        return output_path.to_path_buf();
    }

    output_path.to_path_buf()
}

/// Collects template targets by recursively following include paths.
///
/// `read_includes` receives each absolute source path and returns the include
/// paths declared by that source. Relative include paths are resolved from the
/// source file and mirrored under the output file's parent directory. Absolute
/// include paths remain absolute targets. The callback is also called for source
/// paths that do not exist yet, so callers can treat missing template sources as
/// empty or synthesize default includes.
///
/// # Type Parameters
///
/// - `E`: Error type returned by `read_includes`.
/// - `F`: Include reader callback type.
///
/// # Arguments
///
/// - `config_path`: Preferred config source path.
/// - `output_path`: Root output template path.
/// - `read_includes`: Callback that receives each normalized source path and
///   returns include paths declared by that source.
///
/// # Returns
///
/// Returns all collected template targets in traversal order.
pub fn collect_template_targets<E, F>(
    config_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    mut read_includes: F,
) -> Result<Vec<TemplateTarget>>
where
    E: Into<BoxError>,
    F: FnMut(&Path) -> std::result::Result<Vec<PathBuf>, E>,
{
    let source_path = select_template_source(config_path, output_path.as_ref());
    let mut state = TraversalState::default();
    let mut targets = Vec::new();
    collect_template_target(
        &source_path,
        output_path.as_ref(),
        &mut read_includes,
        &mut state,
        &mut targets,
    )?;
    Ok(targets)
}

/// Recursively maps one source template path to one output template path.
fn collect_template_target<E, F>(
    source_path: &Path,
    target_path: &Path,
    read_includes: &mut F,
    state: &mut TraversalState,
    targets: &mut Vec<TemplateTarget>,
) -> Result<()>
where
    E: Into<BoxError>,
    F: FnMut(&Path) -> std::result::Result<Vec<PathBuf>, E>,
{
    let source_path = absolutize_lexical(source_path)?;
    if !state.enter(&source_path)? {
        return Ok(());
    }

    let include_paths = read_includes(&source_path)
        .map_err(|source| crate::ConfigTreeError::load(&source_path, source))?;
    validate_include_paths(&source_path, &include_paths)?;

    targets.push(TemplateTarget {
        source_path: source_path.clone(),
        target_path: target_path.to_path_buf(),
        include_paths: include_paths.clone(),
    });

    let target_base_dir = target_path.parent().unwrap_or_else(|| Path::new("."));
    for include_path in &include_paths {
        let source_child = resolve_include_path(&source_path, include_path);
        let target_child = if include_path.is_absolute() {
            include_path.clone()
        } else {
            target_base_dir.join(include_path)
        };
        collect_template_target(&source_child, &target_child, read_includes, state, targets)?;
    }

    state.leave();
    Ok(())
}

#[cfg(test)]
#[path = "unit_tests/template.rs"]
mod unit_tests;
