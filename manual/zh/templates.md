# 模板生成

[English](../en/templates.html) | [中文](templates.html)

模板由运行时使用的同一个 `confique` schema 生成。`confique` 负责渲染实际
模板内容，包括文档注释、默认值、必填字段和声明的环境变量名。

使用 `write_config_templates`：

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

生成一份 Draft 7 JSON Schema，供 TOML、YAML 和 JSON 编辑器支持共用：

```rust
use rust_config_tree::write_config_schema;

write_config_schema::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

生成 TOML 和 YAML 模板时绑定这份 schema：

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

TOML 模板会得到 `#:schema` directive。YAML 模板会得到 YAML Language
Server modeline。JSON 和 JSON5 模板保持不变，避免运行时配置里出现
`$schema` 字段。JSON 文件应通过 VS Code `json.schemas` 等编辑器设置绑定。

输出格式由输出路径推断：

- `.yaml` 和 `.yml` 生成 YAML。
- `.toml` 生成 TOML。
- `.json` 和 `.json5` 生成 JSON5-compatible 模板。
- 未知或缺失扩展名生成 YAML。

## Schema 绑定

当 schema path 是 `schemas/myapp.schema.json` 时，生成的模板会使用：

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
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

## 自动 Section 拆分

当 source 文件没有 include 时，crate 可以从嵌套 schema section 推导
include 目标。对于包含 `server` section 的 schema，空 root template source
可以生成：

```text
config.example.yaml
config/server.yaml
```

root template 会得到 include block，`config/server.yaml` 只包含 `server`
section。嵌套 section 会继续递归拆分。
