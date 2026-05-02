# IDE 补全

[English](../en/ide-completions.html) | [中文](ide-completions.html)

一份生成的 JSON Schema 可以同时给 TOML、YAML、JSON 和 JSON5 配置文件使用。
schema 从 `confique` 使用的同一个 Rust 类型生成：

```rust
use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    server: ServerConfig,
}
```

生成方式：

```rust
use rust_config_tree::write_config_schema;

write_config_schema::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

## TOML

TOML 文件应在顶部使用 `#:schema` directive 绑定 schema：

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

不要使用根字段 `$schema = "..."`。它会成为真实配置数据，可能影响运行时
反序列化。`write_config_templates_with_schema` 会为 TOML 模板自动添加
`#:schema` directive。

## YAML

YAML 文件使用 YAML Language Server modeline：

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` 会为 YAML 模板自动添加这个 modeline。

## JSON

JSON 不能写注释，`$schema` 也是一个真实 JSON 属性。生产配置文件应保持干净，
通过编辑器设置绑定：

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

YAML 也可以通过 VS Code settings 绑定：

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

最终布局：

```text
schemas/myapp.schema.json:
  TOML、YAML、JSON 和 JSON5 共用一份 schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config.json:
  不写运行时 $schema 字段，通过编辑器设置绑定
```

参考：

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
