//! Error types shared by the tree loader and high-level config API.
//!
//! The lower-level API reports [`ConfigTreeError`]. The high-level `confique`
//! integration wraps those traversal failures together with dotenv loading,
//! Figment extraction, config parsing, schema serialization, and IO errors in
//! [`ConfigError`].

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

/// Convenience constructors for tree traversal errors.
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

/// Formats tree traversal errors for CLI and library callers.
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

/// Exposes underlying IO or loader causes for tree traversal failures.
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
    Tree(Box<ConfigTreeError>),
    /// Loading an existing `.env` file failed.
    Dotenv(Box<dotenvy::Error>),
    /// Figment failed to load or deserialize runtime config data.
    Figment(Box<figment::Error>),
    /// `confique` failed to load or merge config data.
    Config(Box<confique::Error>),
    /// JSON schema serialization failed.
    Json(Box<serde_json::Error>),
    /// File system or shell completion IO failed.
    Io(Box<io::Error>),
}

/// Formats high-level config errors by delegating to their underlying causes.
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tree(err) => err.fmt(f),
            Self::Dotenv(err) => err.fmt(f),
            Self::Figment(err) => err.fmt(f),
            Self::Config(err) => err.fmt(f),
            Self::Json(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

/// Exposes the wrapped source error for high-level config failures.
impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Tree(err) => Some(err.as_ref()),
            Self::Dotenv(err) => Some(err.as_ref()),
            Self::Figment(err) => Some(err.as_ref()),
            Self::Config(err) => Some(err.as_ref()),
            Self::Json(err) => Some(err.as_ref()),
            Self::Io(err) => Some(err.as_ref()),
        }
    }
}

/// Converts tree traversal failures into high-level config failures.
impl From<ConfigTreeError> for ConfigError {
    fn from(err: ConfigTreeError) -> Self {
        Self::Tree(Box::new(err))
    }
}

/// Converts dotenv loading failures into high-level config failures.
impl From<dotenvy::Error> for ConfigError {
    fn from(err: dotenvy::Error) -> Self {
        Self::Dotenv(Box::new(err))
    }
}

/// Converts Figment extraction failures into high-level config failures.
impl From<figment::Error> for ConfigError {
    fn from(err: figment::Error) -> Self {
        Self::Figment(Box::new(err))
    }
}

/// Converts `confique` merge failures into high-level config failures.
impl From<confique::Error> for ConfigError {
    fn from(err: confique::Error) -> Self {
        Self::Config(Box::new(err))
    }
}

/// Converts JSON serialization failures into high-level config failures.
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(Box::new(err))
    }
}

/// Converts IO failures into high-level config failures.
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(Box::new(err))
    }
}
