//! Recursive include tree traversal primitives.
//!
//! This module provides the format-agnostic tree loader used by the high-level
//! `confique` API. Callers supply a loader that returns a source value and the
//! include paths declared by that source.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{BoxError, ConfigTreeError, Result, absolutize_lexical, resolve_include_path};

/// Controls the order in which sibling include paths are traversed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IncludeOrder {
    /// Visit include paths in the order they were declared.
    #[default]
    Declared,
    /// Visit sibling include paths in reverse declaration order.
    Reverse,
}

/// Options for loading a recursive config tree.
///
/// Use this type when the default traversal behavior is not enough, for example
/// when sibling includes should be visited in reverse declaration order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfigTreeOptions {
    include_order: IncludeOrder,
}

/// Builder-style configuration for include tree traversal.
impl ConfigTreeOptions {
    /// Sets the sibling include traversal order.
    ///
    /// # Arguments
    ///
    /// - `include_order`: Order used when visiting sibling include paths.
    ///
    /// # Returns
    ///
    /// Returns the updated options value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_config_tree::{ConfigTreeOptions, IncludeOrder};
    ///
    /// let options = ConfigTreeOptions::default().include_order(IncludeOrder::Reverse);
    /// # let _ = options;
    /// ```
    pub fn include_order(mut self, include_order: IncludeOrder) -> Self {
        self.include_order = include_order;
        self
    }

    /// Loads a config tree from `root_path` with a custom source loader.
    ///
    /// The loader returns both the source value and the include paths declared
    /// by that source. Relative include paths are resolved from the source path.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Loaded value type stored for each config source.
    /// - `E`: Error type returned by `load`.
    /// - `F`: Source loader callback type.
    ///
    /// # Arguments
    ///
    /// - `root_path`: Root config path to load first.
    /// - `load`: Callback that receives each normalized absolute source path
    ///   and returns the source value with its declared include paths.
    ///
    /// # Returns
    ///
    /// Returns a [`ConfigTree`] containing loaded nodes in traversal order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::{Path, PathBuf}};
    /// use rust_config_tree::{ConfigSource, ConfigTreeOptions};
    ///
    /// let tree = ConfigTreeOptions::default().load(
    ///     "root.yaml",
    ///     |path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         if path.ends_with("root.yaml") {
    ///             Ok(ConfigSource::new("root", vec![PathBuf::from("child.yaml")]))
    ///         } else {
    ///             Ok(ConfigSource::new("child", Vec::new()))
    ///         }
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.into_values(), vec!["root", "child"]);
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
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

    /// Recursively loads one source path and its declared includes.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Loaded value type stored for each config source.
    /// - `E`: Error type returned by `load`.
    /// - `F`: Source loader callback type.
    ///
    /// # Arguments
    ///
    /// - `self`: Traversal options controlling sibling include order.
    /// - `path`: Source path to load.
    /// - `load`: Source loader callback.
    /// - `state`: Traversal state used for cycle detection and deduplication.
    /// - `nodes`: Output list receiving loaded nodes.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` after this path and its includes are collected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
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

/// Value and includes returned by a config source loader.
///
/// # Type Parameters
///
/// - `T`: Loaded source value type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSource<T> {
    value: T,
    includes: Vec<PathBuf>,
}

/// Constructors and accessors for values returned by source loaders.
impl<T> ConfigSource<T> {
    /// Creates a source from a loaded value and its declared include paths.
    ///
    /// # Arguments
    ///
    /// - `value`: Loaded source value.
    /// - `includes`: Include paths declared by the source.
    ///
    /// # Returns
    ///
    /// Returns a new [`ConfigSource`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rust_config_tree::ConfigSource;
    ///
    /// let source = ConfigSource::new("value", vec![PathBuf::from("child.yaml")]);
    ///
    /// assert_eq!(source.value(), &"value");
    /// assert_eq!(source.includes(), &[PathBuf::from("child.yaml")]);
    /// ```
    pub fn new(value: T, includes: Vec<PathBuf>) -> Self {
        Self { value, includes }
    }

    /// Returns the loaded source value.
    ///
    /// # Arguments
    ///
    /// - `self`: Source value being inspected.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the loaded source value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_config_tree::ConfigSource;
    ///
    /// let source = ConfigSource::new("value", Vec::new());
    ///
    /// assert_eq!(source.value(), &"value");
    /// ```
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns include paths declared by the source.
    ///
    /// # Arguments
    ///
    /// - `self`: Source value being inspected.
    ///
    /// # Returns
    ///
    /// Returns the include paths declared by the source.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rust_config_tree::ConfigSource;
    ///
    /// let source = ConfigSource::new("value", vec![PathBuf::from("child.yaml")]);
    ///
    /// assert_eq!(source.includes(), &[PathBuf::from("child.yaml")]);
    /// ```
    pub fn includes(&self) -> &[PathBuf] {
        &self.includes
    }

    /// Decomposes the source into its value and include paths.
    ///
    /// # Arguments
    ///
    /// - `self`: Source value to decompose.
    ///
    /// # Returns
    ///
    /// Returns `(value, includes)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rust_config_tree::ConfigSource;
    ///
    /// let source = ConfigSource::new("value", vec![PathBuf::from("child.yaml")]);
    ///
    /// assert_eq!(source.into_parts(), ("value", vec![PathBuf::from("child.yaml")]));
    /// ```
    pub fn into_parts(self) -> (T, Vec<PathBuf>) {
        (self.value, self.includes)
    }
}

/// Converts the common `(value, includes)` loader shape into a source value.
impl<T> From<(T, Vec<PathBuf>)> for ConfigSource<T> {
    /// Builds a source value from a tuple.
    ///
    /// # Arguments
    ///
    /// - `(value, includes)`: Loaded value and declared include paths.
    ///
    /// # Returns
    ///
    /// Returns a [`ConfigSource`] containing the tuple parts.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    fn from((value, includes): (T, Vec<PathBuf>)) -> Self {
        Self::new(value, includes)
    }
}

/// A loaded config tree in traversal order.
///
/// # Type Parameters
///
/// - `T`: Loaded source value type stored by each node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTree<T> {
    nodes: Vec<ConfigNode<T>>,
}

/// Accessors for a loaded config tree.
impl<T> ConfigTree<T> {
    /// Returns loaded tree nodes in traversal order.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config tree being inspected.
    ///
    /// # Returns
    ///
    /// Returns loaded nodes in traversal order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.nodes().len(), 1);
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn nodes(&self) -> &[ConfigNode<T>] {
        &self.nodes
    }

    /// Decomposes the tree into its nodes.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config tree to decompose.
    ///
    /// # Returns
    ///
    /// Returns the loaded nodes, preserving traversal order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.into_nodes().len(), 1);
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn into_nodes(self) -> Vec<ConfigNode<T>> {
        self.nodes
    }

    /// Decomposes the tree into loaded values, discarding paths and includes.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config tree to decompose.
    ///
    /// # Returns
    ///
    /// Returns loaded source values in traversal order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.into_values(), vec!["root"]);
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn into_values(self) -> Vec<T> {
        self.nodes.into_iter().map(|node| node.value).collect()
    }
}

/// One loaded config source in a tree.
///
/// # Type Parameters
///
/// - `T`: Loaded source value type stored by this node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNode<T> {
    path: PathBuf,
    value: T,
    includes: Vec<PathBuf>,
}

/// Accessors for one loaded config tree node.
impl<T> ConfigNode<T> {
    /// Returns the normalized absolute source path.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config node being inspected.
    ///
    /// # Returns
    ///
    /// Returns the normalized absolute source path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// assert!(tree.nodes()[0].path().ends_with("root.yaml"));
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the loaded source value.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config node being inspected.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the loaded source value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.nodes()[0].value(), &"root");
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns include paths declared by this source.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config node being inspected.
    ///
    /// # Returns
    ///
    /// Returns the include paths declared by this source.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::{Path, PathBuf}};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         if path.ends_with("root.yaml") {
    ///             Ok(ConfigSource::new("root", vec![PathBuf::from("child.yaml")]))
    ///         } else {
    ///             Ok(ConfigSource::new("child", Vec::new()))
    ///         }
    ///     },
    /// )?;
    ///
    /// assert_eq!(tree.nodes()[0].includes(), &[PathBuf::from("child.yaml")]);
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn includes(&self) -> &[PathBuf] {
        &self.includes
    }

    /// Decomposes the node into its loaded value.
    ///
    /// # Arguments
    ///
    /// - `self`: Loaded config node to decompose.
    ///
    /// # Returns
    ///
    /// Returns the loaded source value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{io, path::Path};
    /// use rust_config_tree::{ConfigSource, load_config_tree};
    ///
    /// let tree = load_config_tree(
    ///     "root.yaml",
    ///     |_path: &Path| -> io::Result<ConfigSource<&'static str>> {
    ///         Ok(ConfigSource::new("root", Vec::new()))
    ///     },
    /// )?;
    ///
    /// let mut nodes = tree.into_nodes();
    /// assert_eq!(nodes.remove(0).into_value(), "root");
    /// # Ok::<(), rust_config_tree::ConfigTreeError>(())
    /// ```
    pub fn into_value(self) -> T {
        self.value
    }
}

/// Loads a config tree with default traversal options.
///
/// # Type Parameters
///
/// - `T`: Loaded value type stored for each config source.
/// - `E`: Error type returned by `load`.
/// - `F`: Source loader callback type.
///
/// # Arguments
///
/// - `root_path`: Root config path to load first.
/// - `load`: Callback that receives each normalized absolute source path and
///   returns the source value with its declared include paths.
///
/// # Returns
///
/// Returns a [`ConfigTree`] containing loaded nodes in traversal order.
///
/// # Examples
///
/// ```
/// use std::{io, path::{Path, PathBuf}};
/// use rust_config_tree::{ConfigSource, load_config_tree};
///
/// let tree = load_config_tree(
///     "root.yaml",
///     |path: &Path| -> io::Result<ConfigSource<&'static str>> {
///         if path.ends_with("root.yaml") {
///             Ok(ConfigSource::new("root", vec![PathBuf::from("child.yaml")]))
///         } else {
///             Ok(ConfigSource::new("child", Vec::new()))
///         }
///     },
/// )?;
///
/// assert_eq!(tree.into_values(), vec!["root", "child"]);
/// # Ok::<(), rust_config_tree::ConfigTreeError>(())
/// ```
pub fn load_config_tree<T, E, F>(root_path: impl AsRef<Path>, load: F) -> Result<ConfigTree<T>>
where
    E: Into<BoxError>,
    F: FnMut(&Path) -> std::result::Result<ConfigSource<T>, E>,
{
    ConfigTreeOptions::default().load(root_path, load)
}

/// Tracks paths currently being visited and paths already loaded.
#[derive(Default)]
pub(crate) struct TraversalState {
    visiting: Vec<PathBuf>,
    loaded: HashSet<PathBuf>,
}

/// Include traversal state transitions.
impl TraversalState {
    /// Enters a normalized source path during traversal.
    ///
    /// # Arguments
    ///
    /// - `path`: Normalized absolute source path.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` when traversal should load the path, `Ok(false)` when
    /// it was already loaded, or an include-cycle error when the path is already
    /// in the active traversal stack.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
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

    /// Leaves the current traversal path.
    ///
    /// # Arguments
    ///
    /// - `self`: Traversal state whose current path should be popped.
    ///
    /// # Returns
    ///
    /// This function mutates the traversal stack and returns no value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let _ = ();
    /// ```
    pub(crate) fn leave(&mut self) {
        self.visiting.pop();
    }
}

/// Validates include paths declared by a source.
///
/// # Arguments
///
/// - `path`: Source path whose include list is being validated.
/// - `paths`: Include paths declared by `path`.
///
/// # Returns
///
/// Returns `Ok(())` when every include path is non-empty.
///
/// # Examples
///
/// ```no_run
/// let _ = ();
/// ```
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
