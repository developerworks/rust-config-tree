# Introduction

[English](introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` provides reusable configuration-tree loading and CLI helpers
for Rust applications that use layered config files.

The crate is designed around a small division of responsibilities:

- `confique` owns schema definitions, code defaults, validation, and config
  template generation.
- `figment` owns runtime loading and runtime source metadata.
- `rust-config-tree` owns recursive include traversal, include path resolution,
  `.env` loading, template target discovery, and reusable clap commands.

The crate is useful when an application wants a natural config file layout such
as this:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Each included file can use the same schema shape, and relative include paths are
resolved from the file that declared them. The final config is still a normal
`confique` schema value.

## Main Features

- Recursive include traversal with cycle detection.
- Relative include paths resolved from the declaring file.
- `.env` loading before environment providers are evaluated.
- Schema-declared environment variables without delimiter splitting.
- Figment metadata for runtime source tracking.
- TRACE-level source tracking events through `tracing`.
- Draft 7 JSON Schema generation for editor completion and validation.
- YAML, TOML, JSON, and JSON5 template generation.
- TOML `#:schema` and YAML Language Server schema modelines for generated templates.
- Automatic YAML template splitting for nested sections.
- Built-in clap subcommands for config templates, JSON Schema, and shell completions.
- A lower-level tree API for callers that do not use `confique`.

## Public Entry Points

Use these APIs for most applications:

- `load_config::<S>(path)` loads the final schema.
- `load_config_with_figment::<S>(path)` loads the schema and returns the
  Figment graph used for source tracking.
- `write_config_templates::<S>(config_path, output_path)` writes the root
  template and recursively discovered child templates.
- `write_config_schemas::<S>(output_path)` writes root and section Draft 7
  JSON Schemas.
- `handle_config_command::<Cli, S>(command, config_path)` handles built-in clap
  config commands.

Use `load_config_tree` when you need the traversal primitive without
`confique`.
