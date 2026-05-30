//! Internal JSON Schema extension keys used during generation.

/// JSON Schema extension key that opts a nested section into template/schema splitting.
pub const TREE_SPLIT_SCHEMA_EXTENSION: &str = "x-tree-split";

/// JSON Schema extension key that opts a leaf field out of template and schema output.
pub const ENV_ONLY_SCHEMA_EXTENSION: &str = "x-env-only";
