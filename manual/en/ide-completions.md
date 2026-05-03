# IDE Completions

[English](ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

Generated JSON Schemas can be used by TOML, YAML, JSON, and JSON5 config files.
They are generated from the same Rust type used by `confique`:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend("x-tree-split" = true))]
    server: ServerConfig,
}
```

Generate them with:

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

This writes the root schema and section schemas such as
`schemas/server.schema.json`. Generated schemas omit `required` constraints so
completion works for partial config files without missing-field diagnostics.
The root schema omits split nested section properties, so split child section
completion is available only in files that bind the matching section schema.
Unmarked nested sections remain in the root schema.

IDE schemas still validate present fields, including type, enum, and unknown
property checks supported by the generated schema. Use `config-validate` for
required fields and final merged config validation.

## TOML

TOML files should bind the schema with a top-of-file `#:schema` directive:

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

Do not use a root `$schema = "..."` field in TOML. It becomes real config data
and can affect runtime deserialization. `write_config_templates_with_schema`
adds the `#:schema` directive automatically for TOML templates.

## YAML

YAML files should use the YAML Language Server modeline:

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` adds this modeline automatically for YAML
templates. Split YAML templates bind their section schema, for example
`config/log.yaml` binds `../schemas/log.schema.json`.

## JSON

JSON cannot carry comments, and `$schema` is a real JSON property. Keep runtime
config files clean and bind JSON files through editor settings:

```json
{
  "json.schemas": [
    {
      "fileMatch": [
        "/config.json",
        "/config.*.json",
        "/deploy/*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

YAML can also be bound through VS Code settings:

```json
{
  "yaml.schemas": {
    "./schemas/myapp.schema.json": [
      "config.yaml",
      "config.*.yaml",
      "deploy/*.yaml"
    ]
  }
}
```

The final layout is:

```text
schemas/myapp.schema.json:
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config/server.yaml:
  # yaml-language-server: $schema=../schemas/server.schema.json

config.json:
  No runtime $schema field; bind with editor settings
```

References:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
