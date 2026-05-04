# 模板生成

[English](../en/templates.html) | [中文](templates.html) | [日本語](../ja/templates.html) | [한국어](../ko/templates.html) | [Français](../fr/templates.html) | [Deutsch](../de/templates.html) | [Español](../es/templates.html) | [Português](../pt/templates.html) | [Svenska](../sv/templates.html) | [Suomi](../fi/templates.html) | [Nederlands](../nl/templates.html)

模板由运行时使用的同一个 `confique` schema(结构定义) 生成。`confique` 负责
渲染实际模板内容，包括文档注释、默认值、必填字段和声明的环境变量名。

使用 `write_config_templates`：

```rust
use rust_config_tree::write_config_templates;

write_config_templates::<AppConfig>("config.yaml", "config.example.yaml")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

下面的调用会为 root config(根配置) 和显式拆分的嵌套 section(配置段)
生成 Draft 7 JSON Schema(JSON 结构定义)：

```rust
use rust_config_tree::write_config_schemas;

write_config_schemas::<AppConfig>("schemas/myapp.schema.json")?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

如果 nested(嵌套) 字段需要独立生成 `*.yaml` 模板和
`<section>.schema.json` schema(结构定义)，就使用
`#[schemars(extend("x-tree-split" = true))]` 标记这个字段。没有这个标记的
nested(嵌套) 字段会留在父模板和父 schema(结构定义) 中。

当某个 leaf(叶子) 字段只能从环境变量提供时，可以添加
`#[schemars(extend("x-env-only" = true))]`。生成的模板和
JSON Schema(JSON 结构定义) 会省略 env-only(仅环境变量) 字段；如果父对象因此变空，
生成器也会删除这个父对象。

生成的 schema(结构定义) 会移除 `required` 约束。IDE(集成开发环境) 仍然可以补全，
但是 `log.yaml` 这类局部文件不会因为缺少 root(根配置) 字段而报错。
root schema(根结构定义) 只补全 root(根配置) 文件里应该写的字段；被拆分的
section(配置段) 字段会从 root schema(根结构定义) 中省略，并只由各自的
section schema(配置段结构定义) 补全。
已经出现的字段仍可由 IDE(集成开发环境) 做基础编辑期检查，例如生成的
schema(结构定义) 支持的类型、枚举和未知属性检查。生成的 `*.schema.json`
不负责判断具体字段值对应用是否合法。字段值合法性应在代码中通过
`#[config(validate = Self::validate)]` 实现；
`load_config` 和 `config-validate` 会执行这类运行时校验。

生成 TOML、YAML、JSON 和 JSON5 模板时，可以绑定这些 schema(结构定义)：

```rust
use rust_config_tree::write_config_templates_with_schema;

write_config_templates_with_schema::<AppConfig>(
    "config.toml",
    "config.example.toml",
    "schemas/myapp.schema.json",
)?;
# Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
```

root(根配置) 模板会绑定 root schema(根结构定义)，并且不会补全被拆分的
child section(子配置段) 字段。拆分出的 section(配置段) YAML 模板会绑定对应的
section schema(配置段结构定义)。JSON 和 JSON5 模板会写入顶层 `$schema` 字段。
VS Code(代码编辑器) `json.schemas` 等编辑器设置仍可作为替代绑定方式。

输出格式由输出路径推断：

- `.yaml` 和 `.yml` 生成 YAML。
- `.toml` 生成 TOML。
- `.json` 和 `.json5` 会生成 JSON5-compatible(JSON5 兼容) 模板。
- 未知或缺失扩展名生成 YAML。

模板 API 会严格写入调用方传入的 `output_path`。内置的 `config-template`
CLI(命令行接口) 命令会把生成的模板归档到 `config/<root_config_name>/`；
未传 `--output` 时，`AppConfig` 会写入
`config/app_config/app_config.example.yaml`，对应的默认 schema(结构定义)
写入 `config/app_config/app_config.schema.json`。

## Schema(结构定义) 绑定

当 schema path(结构定义路径) 是 `schemas/myapp.schema.json` 时，生成的
root(根配置) 模板会使用以下内容：

```toml
#:schema ./schemas/myapp.schema.json
```

```yaml
# yaml-language-server: $schema=./schemas/myapp.schema.json
```

生成的 section(配置段) 模板会绑定 section schema(配置段结构定义)：

```yaml
# log.yaml
# yaml-language-server: $schema=./schemas/log.schema.json
```

生成的 JSON 和 JSON5 模板会用顶层 `$schema` 字段绑定 schema(结构定义)：

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
        "/config.*.json"
      ],
      "url": "./schemas/myapp.schema.json"
    }
  ]
}
```

## 模板 Source(来源) 选择

模板生成会按以下顺序选择 source tree(来源树)：

1. 它会先使用已存在的 config path(配置路径)。
2. 它会再使用已存在的 output template path(输出模板路径)。
3. 它最后会把 output path(输出路径) 当作新的空 template tree(模板树)。

这样项目可以从当前配置更新模板，也可以更新已有模板集，还可以只从
schema(结构定义) 创建新的模板集。

## 镜像 Include Tree(包含树)

如果 source(来源) 文件声明了 include(包含文件)，生成的模板会在
output(输出) 目录下镜像这些 include path(包含路径)。

```yaml
# config.yaml
include:
  - server.yaml
```

生成 `config.example.yaml` 会写入：

```text
config.example.yaml
server.yaml
```

相对 include(包含) 目标会镜像到 output(输出) 文件父目录下。绝对
include(包含) 目标会保留绝对路径。

## 显式 Section(配置段) 拆分

当 source(来源) 文件没有 include(包含文件) 时，crate(软件包) 可以从带
`x-tree-split` 标记的嵌套 schema section(结构定义配置段) 推导 include(包含)
目标。对于包含已标记 `server` section(配置段) 的 schema(结构定义)，空
root template source(根模板来源) 可以生成以下文件：

```text
config.example.yaml
server.yaml
```

root template(根模板) 会得到 include block(包含块)，`server.yaml`
只包含 `server` section(配置段)。没有标记的 nested section(嵌套配置段) 会内联
保留在父模板中；更深层 section(配置段) 只有同样带 `x-tree-split` 时才会继续拆分。
