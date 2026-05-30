# IDE(集成开发环境) 补全

[English](../en/ide-completions.html) | [中文](ide-completions.html) | [日本語](../ja/ide-completions.html) | [한국어](../ko/ide-completions.html) | [Français](../fr/ide-completions.html) | [Deutsch](../de/ide-completions.html) | [Español](../es/ide-completions.html) | [Português](../pt/ide-completions.html) | [Svenska](../sv/ide-completions.html) | [Suomi](../fi/ide-completions.html) | [Nederlands](../nl/ide-completions.html)

生成的 JSON Schema(JSON 结构定义) 可以给 TOML、YAML、JSON 和 JSON5 配置文件使用。
这个 schema(结构定义) 从 `confique` 使用的同一个 Rust(系统编程语言) 类型生成：

```rust
use std::path::PathBuf;

use confique::Config;
use schemars::JsonSchema;

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(default = [])]
    #[serde(default)]
    include: Vec<PathBuf>,

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
`validate-config` 触发。必填字段和最终合并配置的校验也使用这些运行时路径。

## Include(包含) 字段的 `#[config(default = [])]` 与 `#[serde(default)]`

root config(根配置) 上的 `include` 字段通常应同时加上这两个注解：

```rust
#[config(default = [])]
#[serde(default)]
include: Vec<PathBuf>,
```

它们作用于不同层，缺一不可：

| 注解 | 作用层 | 必要性 |
|------|--------|--------|
| `#[config(default = [])]` | `confique` 运行时 | 配置文件省略 `include` 时，中间 layer(层) 仍得到空列表；加载器可继续调用 `ConfigSchema::include_paths` 发现子文件；模板生成也会把 `include` 写成空默认值 |
| `#[serde(default)]` | `serde` / `schemars` | 生成 JSON Schema(JSON 结构定义) 时，为 `include` 写入 `"default": []`，并避免在原始 schema 中把它标成必填 |

`#[config(default = [])]` **不会**进入 `schemars` 生成的 JSON Schema(JSON 结构定义)。
如果只有它、没有 `#[serde(default)]`，生成的 root schema(根结构定义) 里仍会出现 `include` 属性，但通常**没有** `"default": []`。

### 对 IDE(集成开发环境) 补全的影响

`rust-config-tree` 会在生成 schema(结构定义) 后移除所有 `required` 约束，因此
`server.yaml` 这类局部文件即使不写 `include`，也不会因为「缺少 root(根配置) 字段」
而报错。这与 `#[serde(default)]` 无关，是 crate(软件包) 对 schema(结构定义) 的后处理。

`#[serde(default)]` 主要影响 **root schema(根结构定义) 绑定到 root 配置文件** 时的编辑体验：

- **有 `#[serde(default)]`**：schema(结构定义) 中 `include` 带 `"default": []`。YAML / JSON
  language server(语言服务器) 会把 `include` 视为可省略字段；补全列表里仍会出现
  `include`，并更容易给出 `[]` 或数组项形状提示。
- **只有 `#[config(default = [])]`**：`include` 仍在 `properties` 中，字段名可被补全，但
  schema(结构定义) 缺少默认值提示。部分 language server(语言服务器) 在较严格的校验模式下，
  仍可能把缺少 `include` 的 root(根配置) 文件标成不完整，或在插入 snippet(代码片段) 时
  不给出空数组默认值。

被 `x-tree-split` 拆出去、并绑定 section schema(配置段结构定义) 的文件（例如
`server.yaml`）**本来就不会**在 schema(结构定义) 里包含 `include`，因此 child section(子配置段)
文件的补全不受这两个注解影响。它们主要服务于 **root config(根配置) 文件**（例如
`config.yaml`、`config.toml`、`config.json`）。

若 `include` 使用了非空默认值(不常见), `#[serde(default)]` 仍应与其 serde 默认值保持一致;
`#[config(default = ...)]` 则与 `confique` 运行时默认值保持一致.

## 透明数组 Section(配置段)

当 split section(拆分配置段) 在单文件里应写成 `section: [...]`, 而 split 文件里只有 body-only 数组 `[...]` 时, 使用 `x-tree-transparent-array` 扩展与 `transparent_array_section!` 宏. **完整说明见 [透明数组 Section(配置段)](transparent-sections.md).**

```rust
use rust_config_tree::transparent_array_section;

transparent_array_section! {
    /// Child declarations stored as a transparent array section.
    pub struct ChildrenSection {
        #[config(default = [{ "name": "worker" }])]
        pub items: Vec<ChildDeclaration>,
    }
}

#[derive(Debug, Config, JsonSchema)]
struct AppConfig {
    #[config(nested)]
    #[schemars(extend(
        "x-tree-split" = true,
        "x-tree-transparent-array" = true
    ))]
    children: ChildrenSection,
}
```

行为说明:

- 运行时 `load_config` 接受 `children: [...]`, `children:\n  items: [...]`, 以及 body-only `children.yaml` 三种形状.
- 生成的 `children.schema.json` 顶层类型为 `array`, 而不是 `{ items: [...] }` 对象.
- 模板生成只输出 block YAML(块状 YAML) 数组体, 不再写入 `children:` 或 flow 风格 `[{ ... }]`.

可选扩展 `x-tree-inner-field = "items"` 可覆盖 confique 内部字段名, 默认值为 `"items"`.

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
