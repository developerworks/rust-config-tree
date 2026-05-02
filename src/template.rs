use std::path::{Path, PathBuf};

use crate::{
    BoxError, Result, absolutize_lexical, resolve_include_path,
    tree::{TraversalState, validate_include_paths},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateTarget {
    source_path: PathBuf,
    target_path: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl TemplateTarget {
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn target_path(&self) -> &Path {
        &self.target_path
    }

    pub fn include_paths(&self) -> &[PathBuf] {
        &self.include_paths
    }

    pub fn into_parts(self) -> (PathBuf, PathBuf, Vec<PathBuf>) {
        (self.source_path, self.target_path, self.include_paths)
    }
}

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

    let include_paths = if source_path.exists() {
        read_includes(&source_path)
            .map_err(|source| crate::ConfigTreeError::load(&source_path, source))?
    } else {
        Vec::new()
    };
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
