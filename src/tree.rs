use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{BoxError, ConfigTreeError, Result, absolutize_lexical, resolve_include_path};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IncludeOrder {
    #[default]
    Declared,
    Reverse,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigTreeOptions {
    include_order: IncludeOrder,
}

impl ConfigTreeOptions {
    pub fn include_order(mut self, include_order: IncludeOrder) -> Self {
        self.include_order = include_order;
        self
    }

    pub fn load<T, E, F>(&self, root_path: impl AsRef<Path>, mut load: F) -> Result<ConfigTree<T>>
    where
        E: Into<BoxError>,
        F: FnMut(&Path) -> std::result::Result<ConfigSource<T>, E>,
    {
        let mut state = TraversalState::default();
        let mut nodes = Vec::new();
        self.collect(root_path.as_ref(), &mut load, &mut state, &mut nodes)?;
        Ok(ConfigTree { nodes })
    }

    fn collect<T, E, F>(
        &self,
        path: &Path,
        load: &mut F,
        state: &mut TraversalState,
        nodes: &mut Vec<ConfigNode<T>>,
    ) -> Result<()>
    where
        E: Into<BoxError>,
        F: FnMut(&Path) -> std::result::Result<ConfigSource<T>, E>,
    {
        let path = absolutize_lexical(path)?;
        if !state.enter(&path)? {
            return Ok(());
        }

        let source = load(&path).map_err(|source| ConfigTreeError::load(&path, source))?;
        validate_include_paths(&path, &source.includes)?;

        let includes = source.includes;
        nodes.push(ConfigNode {
            path: path.clone(),
            value: source.value,
            includes: includes.clone(),
        });

        match self.include_order {
            IncludeOrder::Declared => {
                for include_path in &includes {
                    let include_path = resolve_include_path(&path, include_path);
                    self.collect(&include_path, load, state, nodes)?;
                }
            }
            IncludeOrder::Reverse => {
                for include_path in includes.iter().rev() {
                    let include_path = resolve_include_path(&path, include_path);
                    self.collect(&include_path, load, state, nodes)?;
                }
            }
        }

        state.leave();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSource<T> {
    value: T,
    includes: Vec<PathBuf>,
}

impl<T> ConfigSource<T> {
    pub fn new(value: T, includes: Vec<PathBuf>) -> Self {
        Self { value, includes }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn includes(&self) -> &[PathBuf] {
        &self.includes
    }

    pub fn into_parts(self) -> (T, Vec<PathBuf>) {
        (self.value, self.includes)
    }
}

impl<T> From<(T, Vec<PathBuf>)> for ConfigSource<T> {
    fn from((value, includes): (T, Vec<PathBuf>)) -> Self {
        Self::new(value, includes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTree<T> {
    nodes: Vec<ConfigNode<T>>,
}

impl<T> ConfigTree<T> {
    pub fn nodes(&self) -> &[ConfigNode<T>] {
        &self.nodes
    }

    pub fn into_nodes(self) -> Vec<ConfigNode<T>> {
        self.nodes
    }

    pub fn into_values(self) -> Vec<T> {
        self.nodes.into_iter().map(|node| node.value).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNode<T> {
    path: PathBuf,
    value: T,
    includes: Vec<PathBuf>,
}

impl<T> ConfigNode<T> {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn includes(&self) -> &[PathBuf] {
        &self.includes
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

pub fn load_config_tree<T, E, F>(root_path: impl AsRef<Path>, load: F) -> Result<ConfigTree<T>>
where
    E: Into<BoxError>,
    F: FnMut(&Path) -> std::result::Result<ConfigSource<T>, E>,
{
    ConfigTreeOptions::default().load(root_path, load)
}

#[derive(Default)]
pub(crate) struct TraversalState {
    visiting: Vec<PathBuf>,
    loaded: HashSet<PathBuf>,
}

impl TraversalState {
    pub(crate) fn enter(&mut self, path: &Path) -> Result<bool> {
        if let Some(pos) = self.visiting.iter().position(|existing| existing == path) {
            let mut chain = self.visiting[pos..].to_vec();
            chain.push(path.to_path_buf());
            return Err(ConfigTreeError::IncludeCycle { chain });
        }

        if !self.loaded.insert(path.to_path_buf()) {
            return Ok(false);
        }

        self.visiting.push(path.to_path_buf());
        Ok(true)
    }

    pub(crate) fn leave(&mut self) {
        self.visiting.pop();
    }
}

pub(crate) fn validate_include_paths(path: &Path, paths: &[PathBuf]) -> Result<()> {
    for (index, include_path) in paths.iter().enumerate() {
        if include_path.as_os_str().is_empty() {
            return Err(ConfigTreeError::EmptyIncludePath {
                path: path.to_path_buf(),
                index,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "unit_tests/tree.rs"]
mod unit_tests;
