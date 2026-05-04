# 模板生成

[English](../en/templates.html) | [中文](templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

模板由运行时使用的同一个 `confique` schema 生成。`confique` 负责渲染实际
模板内容，包括文档注释、默认值、必填字段和声明的环境变量名。

使用 `write_config_templates`：

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

为 root config 和显式拆分的嵌套 section 生成 Draft 7 JSON Schema：

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

需要独立生成 `config/*.yaml` 模板和 `schemas/*.schema.json` schema 的
nested 字段使用 `#[schemars(extend("x-tree-split" = true))]` 标记。没有这个
标记的 nested 字段会留在父模板和父 schema 中。

当某个 leaf 字段只能从环境变量提供时，可以加
`#[schemars(extend("x-env-only" = true))]`。生成的模板和 JSON Schema 会省略
env-only 字段；如果父对象因此变空，也会一并裁剪。

生成的 schema 会移除 `required` 约束。IDE 仍然可以补全，但
`config/log.yaml` 这类局部文件不会因为缺少 root 字段而报错。
root schema 只补全 root 文件里应该写的字段；被拆分的 section 字段会从
root schema 中省略，只由各自的 section schema 补全。
已经出现的字段仍可由 IDE 做基础编辑期检查，例如生成 schema 支持的类型、枚举和
未知属性检查。生成的 `*.schema.json` 不负责判断具体字段值对应用是否合法。
字段值合法性应在代码中通过 `#[config(validate = Self::validate)]` 实现；
`load_config` 和 `config-validate` 会执行这类运行时校验。

生成 TOML 和 YAML 模板时绑定这些 schema：

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

root TOML/YAML 模板会绑定 root schema，并且不会补全被拆分的 child section
字段。拆分出的 section YAML 模板会绑定对应的 section schema。JSON 和 JSON5 模板
保持不变，避免运行时配置里出现 `$schema` 字段。JSON 文件应通过 VS Code
`json.schemas` 等编辑器设置绑定。

输出格式由输出路径推断：

- `.yaml` 和 `.yml` 生成 YAML。
- `.toml` 生成 TOML。
- `.json` 和 `.json5` 生成 JSON5-compatible 模板。
- 未知或缺失扩展名生成 YAML。

## Schema 绑定

当 schema path 是 `schemas/myapp.schema.json` 时，生成的 root 模板会使用：

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

生成的 section 模板会绑定 section schema：

```yaml
# config/log.yaml
# yaml-language-server: $schema=../schemas/log.schema.json
```

JSON 不写 `$schema`，通过编辑器设置绑定：

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

## 模板 Source 选择

模板生成按这个顺序选择 source tree：

1. 已存在的 config path。
2. 已存在的 output template path。
3. 将 output path 作为新的空 template tree。

这样项目可以从当前配置更新模板、更新已有模板集，或仅从 schema 创建新的
模板集。

## 镜像 Include Tree

如果 source 文件声明了 include，生成的模板会在 output 目录下镜像这些
include path。

```yaml
# config.yaml
include:
  - config/server.yaml
```

生成 `config.example.yaml` 会写入：

```text
config.example.yaml
config/server.yaml
```

相对 include 目标会镜像到 output 文件父目录下。绝对 include 目标保持
绝对路径。

## 显式 Section 拆分

当 source 文件没有 include 时，crate 可以从带 `x-tree-split` 标记的嵌套
schema section 推导 include 目标。对于包含已标记 `server` section 的
schema，空 root template source 可以生成：

```text
config.example.yaml
config/server.yaml
```

root template 会得到 include block，`config/server.yaml` 只包含 `server`
section。没有标记的 nested section 会内联保留在父模板中；更深层 section
只有同样带 `x-tree-split` 时才会继续拆分。
