use std::path::PathBuf;

/// Generated JSON Schema content for one output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigSchemaTarget {
    /// Path that should receive the generated schema.
    pub path: PathBuf,
    /// Complete JSON Schema content to write to `path`.
    pub content: String,
}

