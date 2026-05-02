# Template Generation

[English](templates.md) | [中文](../zh/templates.md)

Templates are generated from the same `confique` schema used at runtime.
`confique` renders the actual template content, including doc comments,
defaults, required fields, and declared environment variable names.

Use `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

The output format is inferred from the output path:

- `.yaml` and `.yml` generate YAML.
- `.toml` generates TOML.
- `.json` and `.json5` generate JSON5-compatible templates.
- unknown or missing extensions generate YAML.

## Template Source Selection

Template generation chooses its source tree in this order:

1. Existing config path.
2. Existing output template path.
3. Output path treated as a new empty template tree.

This lets a project update templates from current config files, update an
existing template set, or create a new template set from only the schema.

## Mirrored Include Trees

If the source file declares includes, generated templates mirror those include
paths under the output directory.

```yaml
# config.yaml
include:
  - config/server.yaml
```

Generating `config.example.yaml` writes:

```text
config.example.yaml
config/server.yaml
```

Relative include targets are mirrored under the output file's parent directory.
Absolute include targets remain absolute.

## Automatic Section Splitting

When a source file has no includes, the crate can derive include targets from
nested schema sections. For a schema with a `server` section, an empty root
template source can produce:

```text
config.example.yaml
config/server.yaml
```

The root template receives an include block, and `config/server.yaml` contains
only the `server` section. Nested sections are split recursively.
