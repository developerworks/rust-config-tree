//! Error types shared by the tree loader and high-level config API.
//!
//! The lower-level API reports [`ConfigTreeError`]. The high-level `confique`
//! integration wraps those traversal failures together with dotenv loading,
//! config parsing, and IO errors in [`ConfigError`].

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

/// Boxed error type used by custom loaders.
///
/// Loader errors are boxed so tree traversal can accept different concrete
/// error types through a single public API.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Result type used by the lower-level tree API.
///
/// The error type is [`ConfigTreeError`].
pub type Result<T> = std::result::Result<T, ConfigTreeError>;

/// Errors produced while traversing a recursive config tree.
#[derive(Debug)]
pub enum ConfigTreeError {
    /// The current directory could not be resolved while absolutizing a path.
    CurrentDir {
        /// Underlying IO error returned while reading the current directory.
        source: io::Error,
    },
    /// A source loader failed for the given path.
    Load {
        /// Path that failed to load.
        path: PathBuf,
        /// Underlying loader error.
        source: BoxError,
    },
    /// An include list contained an empty path.
    EmptyIncludePath {
        /// Path whose include list contained the empty entry.
        path: PathBuf,
        /// Zero-based index of the empty include entry.
        index: usize,
    },
    /// Recursive includes formed a cycle.
    IncludeCycle {
        /// Normalized path chain that forms the include cycle.
        chain: Vec<PathBuf>,
    },
}

impl ConfigTreeError {
    /// Builds a loader failure for a source path.
    ///
    /// # Type Parameters
    ///
    /// - `E`: Concrete error type returned by the source loader.
    ///
    /// # Arguments
    ///
    /// - `path`: Source path that failed to load.
    /// - `source`: Underlying loader error.
    ///
    /// # Returns
    ///
    /// Returns a [`ConfigTreeError::Load`] value.
    pub(crate) fn load<E>(path: &Path, source: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self::Load {
            path: path.to_path_buf(),
            source: source.into(),
        }
    }
}

impl fmt::Display for ConfigTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDir { .. } => write!(f, "failed to resolve current directory"),
            Self::Load { path, source } => {
                write!(f, "failed to load config {}: {source}", path.display())
            }
            Self::EmptyIncludePath { path, index } => write!(
                f,
                "include path at index {index} in {} must not be empty",
                path.display()
            ),
            Self::IncludeCycle { chain } => {
                let chain = chain
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "recursive config include cycle: {chain}")
            }
        }
    }
}

impl Error for ConfigTreeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurrentDir { source } => Some(source),
            Self::Load { source, .. } => Some(source.as_ref()),
            Self::EmptyIncludePath { .. } | Self::IncludeCycle { .. } => None,
        }
    }
}

/// Errors produced by high-level config loading and template generation.
#[derive(Debug)]
pub enum ConfigError {
    /// Tree traversal failed.
    Tree(ConfigTreeError),
    /// Loading an existing `.env` file failed.
    Dotenv(dotenvy::Error),
    /// `confique` failed to load or merge config data.
    Config(confique::Error),
    /// File system or shell completion IO failed.
    Io(io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tree(err) => err.fmt(f),
            Self::Dotenv(err) => err.fmt(f),
            Self::Config(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Tree(err) => Some(err),
            Self::Dotenv(err) => Some(err),
            Self::Config(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

impl From<ConfigTreeError> for ConfigError {
    fn from(err: ConfigTreeError) -> Self {
        Self::Tree(err)
    }
}

impl From<dotenvy::Error> for ConfigError {
    fn from(err: dotenvy::Error) -> Self {
        Self::Dotenv(err)
    }
}

impl From<confique::Error> for ConfigError {
    fn from(err: confique::Error) -> Self {
        Self::Config(err)
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
