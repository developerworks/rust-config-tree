//! Internal JSON Schema extension keys used during generation.

/// JSON Schema extension key that opts a nested section into template/schema splitting.
pub const TREE_SPLIT_SCHEMA_EXTENSION: &str = "x-tree-split";

/// JSON Schema extension key that opts a split section into transparent array serialization.
pub const TREE_TRANSPARENT_ARRAY_EXTENSION: &str = "x-tree-transparent-array";

/// JSON Schema extension key naming the confique inner field for transparent sections.
pub const TREE_INNER_FIELD_EXTENSION: &str = "x-tree-inner-field";

/// Default confique inner field name for transparent array sections.
pub const DEFAULT_TREE_INNER_FIELD: &str = "items";

/// JSON Schema extension key that opts a leaf field out of template and schema output.
pub const ENV_ONLY_SCHEMA_EXTENSION: &str = "x-env-only";
