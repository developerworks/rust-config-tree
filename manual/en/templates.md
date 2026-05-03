# Template Generation

[English](templates.html) | [中文](../zh/templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

Templates are generated from the same `confique` schema used at runtime.
`confique` renders the actual template content, including doc comments,
defaults, required fields, and declared environment variable names.

Use `write_config_templates`:

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Generate Draft 7 JSON Schemas for the root config and split nested sections:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Mark a nested field with `#[schemars(extend("x-tree-split" = true))]` when it
should be generated as its own `config/*.yaml` template and
`schemas/*.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Generated schemas omit `required` constraints. IDEs can still offer completion,
but partial files such as `config/log.yaml` do not report missing root fields.
The root schema only completes fields that belong in the root file; split
section fields are omitted there and completed by their own section schemas.
Present fields are still schema-checked by the IDE. Required fields and final
merged config validation are handled by `load_config` or `config-validate`.

Bind those schemas from generated TOML and YAML templates:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Root TOML/YAML templates bind the root schema and do not complete split child
section fields. Split section YAML templates bind their section schema. JSON and JSON5
templates are left unchanged so the runtime config does not contain a
`$schema` field. Bind JSON files with editor settings such as VS Code
`json.schemas`.

The output format is inferred from the output path:

- `.yaml` and `.yml` generate YAML.
- `.toml` generates TOML.
- `.json` and `.json5` generate JSON5-compatible templates.
- unknown or missing extensions generate YAML.

## Schema Bindings

With a schema path of `schemas/myapp.schema.json`, generated root templates use:

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

Generated section templates bind section schemas:

```yaml
# config/log.yaml
# yaml-language-server: $schema=../schemas/log.schema.json
```

For JSON, keep the file free of `$schema` and bind it with editor settings:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

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

## Opt-in Section Splitting

When a source file has no includes, the crate can derive include targets from
nested schema sections marked with `x-tree-split`. For a schema with a marked
`server` section, an empty root template source can produce:

```text
config.example.yaml
config/server.yaml
```

The root template receives an include block, and `config/server.yaml` contains
only the `server` section. Unmarked nested sections stay inline in their parent
template. Nested sections are split recursively only when those fields also
carry `x-tree-split`.
