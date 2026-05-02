# IDE Completions

[English](ide-completions.html) | [中文](../zh/ide-completions.html)

One generated JSON Schema can be shared by TOML, YAML, JSON, and JSON5 config
files. The schema is generated from the same Rust type used by `confique`:

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

Generate it with:

```rust
use rust_config_tree::write_config_schema;

write_config_schema::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

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
templates.

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
  One schema shared by TOML, YAML, JSON, and JSON5

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config.json:
  No runtime $schema field; bind with editor settings
```

References:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
