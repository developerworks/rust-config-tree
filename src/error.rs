use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, ConfigTreeError>;

#[derive(Debug)]
pub enum ConfigTreeError {
    CurrentDir { source: io::Error },
    Load { path: PathBuf, source: BoxError },
    EmptyIncludePath { path: PathBuf, index: usize },
    IncludeCycle { chain: Vec<PathBuf> },
}

impl ConfigTreeError {
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

#[derive(Debug)]
pub enum ConfigError {
    Tree(ConfigTreeError),
    Config(confique::Error),
    Io(io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tree(err) => err.fmt(f),
            Self::Config(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Tree(err) => Some(err),
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
