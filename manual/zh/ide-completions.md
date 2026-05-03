# IDE 补全

[English](../en/ide-completions.html) | [中文](ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

生成的 JSON Schema 可以给 TOML、YAML、JSON 和 JSON5 配置文件使用。schema
从 `confique` 使用的同一个 Rust 类型生成：

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

生成方式：

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

这会写入 root schema 和 `schemas/server.schema.json` 这类 section schema。
生成的 schema 会移除 `required` 约束，局部配置文件仍有补全，但不会出现缺字段
诊断。
root schema 会省略被拆分的 nested section 属性，所以 child section 的补全只会出现在
绑定对应 section schema 的文件里。没有标记的 nested section 会保留在 root schema 中。

IDE schema 仍会校验已经出现的字段，包括生成 schema 支持的类型、枚举和未知
属性检查。必填字段和最终合并配置的校验使用 `config-validate`。

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
拆分出的 YAML 模板会绑定对应 section schema，例如 `config/log.yaml`
绑定 `../schemas/log.schema.json`。

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
  只包含 root 文件字段

schemas/server.schema.json:
  server section schema

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

config/server.yaml:
  # yaml-language-server: $schema=../schemas/server.schema.json

config.json:
  不写运行时 $schema 字段，通过编辑器设置绑定
```

参考：

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
