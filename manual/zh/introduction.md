# 简介

[English](../en/introduction.html) | [中文](introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` 为使用分层配置文件的 Rust(系统编程语言) 应用提供可复用的
配置树加载能力和 CLI(命令行接口) 辅助能力。

这个 crate(软件包) 按照清晰的职责边界组织各项功能：

- `confique` 负责 schema(结构定义)、代码默认值、校验和配置模板生成。
- `figment` 负责运行时加载和运行时来源元数据。
- `rust-config-tree` 负责 include(包含文件) 的递归遍历、include(包含文件)
  路径解析、`.env` 文件加载、模板目标发现，以及可复用的 clap(命令行解析库)
  命令。

它适合这种自然的配置文件布局：

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

每个被 include(包含) 的文件都可以使用相同的 schema(结构定义) 形状。加载器会从
声明 include(包含) 的文件所在目录解析相对路径。最终得到的配置仍然是一个普通的
`confique` schema(结构定义) 值。

## 主要能力

- 它会递归遍历 include(包含文件)，并检测循环包含。
- 它会从声明 include(包含) 的文件解析相对路径。
- 它会在环境变量 provider(值提供器) 求值之前加载 `.env` 文件。
- 它会使用 schema(结构定义) 中声明的环境变量名，并且不会按分隔符拆分变量名。
- 它会通过 Figment(配置合并库) metadata(元数据) 追踪运行时来源。
- 它会通过 `tracing` 输出 TRACE(追踪级别) 的来源追踪事件。
- 它会生成 Draft 7 JSON Schema(JSON 结构定义)，供编辑器补全和基础
  schema(结构定义) 检查使用。
- 应用代码通过 `#[config(validate = Self::validate)]` 实现字段值合法性校验，
  `load_config` 或 `config-validate` 会执行这个校验。
- 它会生成 YAML、TOML、JSON 和 JSON5 配置模板。
- 它会为生成的 TOML 模板写入 `#:schema`，为 YAML 模板写入
  YAML Language Server(YAML 语言服务器) modeline(模式声明行)，并为
  JSON 和 JSON5 模板写入 `$schema` 字段。
- 它会按 `x-tree-split` 标记拆分嵌套 section(配置段) 的 YAML 模板。
- 它内置了用于 config template(配置模板)、JSON Schema(JSON 结构定义) 和
  shell completion(命令补全) 的 clap(命令行解析库) 子命令。
- 它为不使用 `confique` 的调用方提供低层 tree API(树形接口)。

## 主要入口

多数应用会使用这些 API(应用程序接口)：

- `load_config::<S>(path)` 会加载最终 schema(结构定义)。
- `load_config_with_figment::<S>(path)` 会加载 schema(结构定义)，并返回用于
  来源追踪的 Figment(配置合并库) graph(配置图)。
- `write_config_templates::<S>(config_path, output_path)` 会写入 root(根配置)
  模板和递归发现的子模板。
- `write_config_schemas::<S>(output_path)` 会写入 root(根配置) 和
  section(配置段) 的 Draft 7 JSON Schema(JSON 结构定义)。
- `handle_config_command::<Cli, S>(command, config_path)` 会处理内置的
  clap(命令行解析库) 配置命令。

不使用 `confique` 时，可以直接使用 `load_config_tree`。
