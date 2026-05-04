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
should be generated as its own `*.yaml` template and
`<section>.schema.json` schema. Unmarked nested fields stay in the parent
template and parent schema.

Mark a leaf field with `#[schemars(extend("x-env-only" = true))]` when the value must come only from environment variables. Generated templates and JSON Schemas omit env-only fields, and empty parent objects left behind by those omissions are pruned.

Generated schemas omit `required` constraints. IDEs can still offer completion,
but partial files such as `log.yaml` do not report missing root fields.
The root schema only completes fields that belong in the root file; split
section fields are omitted there and completed by their own section schemas.
Present fields can still receive basic editor checks, such as type, enum, and
unknown property checks supported by the generated schema. Generated
`*.schema.json` files do not decide whether a concrete field value is legal for
the application. Implement field value validation in code with
`#[config(validate = Self::validate)]`; `load_config` and `config-validate`
execute that runtime validation.

Bind those schemas from generated TOML, YAML, JSON, and JSON5 templates:

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

Root templates bind the root schema and do not complete split child section
fields. Split section YAML templates bind their section schema. JSON and JSON5
templates receive a top-level `$schema` field. Editor settings such as VS Code
`json.schemas` can still be used as an alternative binding path.

The output format is inferred from the output path:

- `.yaml` and `.yml` generate YAML.
- `.toml` generates TOML.
- `.json` and `.json5` generate JSON5-compatible templates.
- unknown or missing extensions generate YAML.

The template APIs write exactly under the `output_path` you pass. The built-in
`config-template` CLI command normalizes generated templates under
`config/<root_config_name>/`; without `--output`, `AppConfig` writes
`config/app_config/app_config.example.yaml` and the matching default schema
`config/app_config/app_config.schema.json`.

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
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

Generated JSON and JSON5 templates bind the schema with a top-level `$schema`
field:

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

Editor settings are still useful when a project does not want an in-file
binding:

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
  - server.yaml
```

Generating `config.example.yaml` writes:

```text
config.example.yaml
server.yaml
```

Relative include targets are mirrored under the output file's parent directory.
Absolute include targets remain absolute.

## Opt-in Section Splitting

When a source file has no includes, the crate can derive include targets from
nested schema sections marked with `x-tree-split`. For a schema with a marked
`server` section, an empty root template source can produce:

```text
config.example.yaml
server.yaml
```

The root template receives an include block, and `server.yaml` contains
only the `server` section. Unmarked nested sections stay inline in their parent
template. Nested sections are split recursively only when those fields also
carry `x-tree-split`.
