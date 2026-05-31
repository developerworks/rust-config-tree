use std::path::PathBuf;

/// Generated template content for one output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTemplateTarget {
    /// Path that should receive the generated content.
    pub path: PathBuf,
    /// Complete template content to write to `path`.
    pub content: String,
}
