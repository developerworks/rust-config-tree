# IDE(集成开发环境) 补全

[English](../en/ide-completions.html) | [中文](ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

生成的 JSON Schema(JSON 结构定义) 可以给 TOML、YAML、JSON 和 JSON5 配置文件使用。
这个 schema(结构定义) 从 `confique` 使用的同一个 Rust(系统编程语言) 类型生成：

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

这会写入 root schema(根结构定义) 和 `schemas/server.schema.json` 这类
section schema(配置段结构定义)。生成的 schema(结构定义) 会移除 `required` 约束，
所以局部配置文件仍有补全，并且不会出现缺字段诊断。
root schema(根结构定义) 会省略被拆分的 nested section(嵌套配置段) 属性，所以
child section(子配置段) 的补全只会出现在绑定对应 section schema(配置段结构定义)
的文件里。没有标记的 nested section(嵌套配置段) 会保留在
root schema(根结构定义) 中。

带 `x-env-only` 标记的字段会从生成的 schema(结构定义) 中省略，因此
IDE(集成开发环境) 不会补全必须只来自环境变量的 secret(秘密值) 或其他值。

IDE schema(集成开发环境结构定义) 只用于补全和基础编辑期检查，例如生成的
schema(结构定义) 支持的类型、枚举和未知属性检查。它不负责判断具体字段值对应用
是否合法。字段值合法性应在代码中通过
`#[config(validate = Self::validate)]` 实现，并由 `load_config` 或
`config-validate` 触发。必填字段和最终合并配置的校验也使用这些运行时路径。

## TOML

TOML 文件应在顶部使用 `#:schema` directive(指令) 绑定 schema(结构定义)：

```toml
#:schema ./schemas/myapp.schema.json

[server]
bind = "0.0.0.0"
port = 3000
```

不要使用根字段 `$schema = "..."`。它会成为真实配置数据，可能影响运行时
反序列化。`write_config_templates_with_schema` 会为 TOML 模板自动添加
`#:schema` directive(指令)。

## YAML

YAML 文件使用 YAML Language Server(YAML 语言服务器) modeline(模式声明行)：

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json

server:
  bind: 0.0.0.0
  port: 3000
```

`write_config_templates_with_schema` 会为 YAML 模板自动添加这个 modeline(模式声明行)。
拆分出的 YAML 模板会绑定对应 section schema(配置段结构定义)，例如 `log.yaml`
绑定 `./schemas/log.schema.json`。

## JSON

JSON 和 JSON5 文件可以用顶层 `$schema` 字段绑定 schema(结构定义)。
`write_config_templates_with_schema` 会为生成的 JSON 和 JSON5 模板自动加入这个字段：

```json
{
  "$schema": "./schemas/myapp.schema.json"
}
```

如果项目不想在文件内写绑定，也可以通过编辑器设置绑定：

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

YAML 也可以通过 VS Code settings(代码编辑器设置) 绑定：

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

最终布局如下：

```text
schemas/myapp.schema.json:
  只包含 root(根配置) 文件字段

schemas/server.schema.json:
  server(服务器) 配置段结构定义

config.toml:
  #:schema ./schemas/myapp.schema.json

config.yaml:
  # yaml-language-server: $schema=./schemas/myapp.schema.json

server.yaml:
  # yaml-language-server: $schema=./schemas/server.schema.json

config.json:
  "$schema": "./schemas/myapp.schema.json"
```

参考：

- [Tombi JSON Schema](https://tombi-toml.github.io/tombi/docs/json-schema/)
- [Taplo directives](https://taplo.tamasfe.dev/configuration/directives.html)
- [YAML Language Server](https://github.com/redhat-developer/yaml-language-server)
- [VS Code JSON](https://code.visualstudio.com/Docs/languages/json)
