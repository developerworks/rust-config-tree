# IDE 補完

[English](../en/ide-completions.html) | [中文](../zh/ide-completions.html) | [日本語](ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

generated JSON Schema は TOML、YAML、JSON、JSON5 config file で使えます。
schema は `confique` が使う同じ Rust type から生成されます。

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

生成します。

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

これは root schema と `schemas/server.schema.json` のような section schema を
書きます。generated schemas は `required` constraint を省略するため、partial
config file に completion を出しながら missing-field diagnostic を出しません。
root schema は nested section property を省略するため、child section completion
は matching section schema を bind した file でだけ使えます。

`x-env-only` で mark した field は generated schema から省略されるため、環境変数だけで渡す secret などは IDE 補完に出ません。

IDE schema は補完と基本的な editor check のためのものです。type、enum、
unknown property check など、生成 schema が表せる範囲だけを扱います。
具体的な field value が application として合法かどうかは判断しません。
field value validation は code 側で `#[config(validate = Self::validate)]` として
実装し、`load_config` または `config-validate` で実行します。required field と
final merged config validation もこの runtime path で扱います。

## TOML

TOML file は top-of-file `#:schema` directive で schema を bind します。

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

TOML で root `$schema = "..."` field は使わないでください。real config data
になり、runtime deserialization に影響する可能性があります。
`write_config_templates_with_schema` は TOML template に `#:schema` directive を
自動追加します。

## YAML

YAML file は YAML Language Server modeline を使います。

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` は YAML template にこの modeline を自動追加
します。split YAML template は section schema を bind します。たとえば
`log.yaml` は `./schemas/log.schema.json` を bind します。

## JSON

JSON / JSON5 file は root `$schema` field で schema を bind できます。`write_config_templates_with_schema` は generated JSON / JSON5 template に自動で追加します。

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

file 内 binding を使わない project では editor settings も引き続き使えます。

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

YAML も VS Code settings で bind できます。

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

最終 layout:

```text
schemas/myapp.schema.json:
  Root file fields only

schemas/server.schema.json:
  Server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

References:

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
