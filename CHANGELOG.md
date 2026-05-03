# Changelog

All notable changes to `rust-config-tree` are documented in this file.

## 0.1.5 - 2026-05-03

### Fixed

- Fixed config-template generation for existing template trees. When a source
  config already declares `include`, missing default child includes inferred
  from nested schema sections are now appended instead of being skipped. This
  keeps newly added nested config sections split into their own generated files
  on regeneration.

### Changed

- Nested config sections are now split only when the field schema has
  `x-tree-split = true`, for example
  `#[schemars(extend("x-tree-split" = true))]`. Unmarked nested sections remain
  in their parent template and parent JSON Schema.
