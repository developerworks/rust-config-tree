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
