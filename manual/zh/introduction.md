# 简介

[English](../en/introduction.html) | [中文](introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](../fi/introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` 为使用分层配置文件的 Rust 应用提供可复用的配置树加载
能力和 CLI 辅助能力。

这个 crate 按照清晰的职责边界组织：

- `confique` 负责 schema 定义、代码默认值、校验和配置模板生成。
- `figment` 负责运行时加载和运行时来源元数据。
- `rust-config-tree` 负责递归 include 遍历、include 路径解析、`.env`
  加载、模板目标发现，以及可复用的 clap 命令。

它适合这种自然的配置文件布局：

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

每个被 include 的文件都可以使用相同的 schema 形状。相对 include 路径
会从声明它的文件所在目录解析。最终得到的配置仍然是一个普通的
`confique` schema 值。

## 主要能力

- 递归 include 遍历和循环检测。
- 相对 include 路径从声明文件解析。
- 在环境变量 provider 求值之前加载 `.env`。
- 基于 schema 声明的环境变量，不使用分隔符拆分。
- 通过 Figment metadata 追踪运行时来源。
- 通过 `tracing` 输出 TRACE 级别的来源追踪事件。
- 生成 Draft 7 JSON Schema，供编辑器补全和校验使用。
- 生成 YAML、TOML、JSON 和 JSON5 配置模板。
- 为生成的 TOML 模板写入 `#:schema`，为 YAML 模板写入 YAML Language Server modeline。
- 按 `x-tree-split` 标记拆分嵌套 section YAML 模板。
- 内置 config template、JSON Schema 和 shell completion 的 clap 子命令。
- 面向非 `confique` 调用方的低层 tree API。

## 主要入口

多数应用使用这些 API：

- `load_config::<S>(path)` 加载最终 schema。
- `load_config_with_figment::<S>(path)` 加载 schema，并返回用于来源追踪
  的 Figment graph。
- `write_config_templates::<S>(config_path, output_path)` 写入 root 模板和
  递归发现的子模板。
- `write_config_schemas::<S>(output_path)` 写入 root 和 section Draft 7
  JSON Schema。
- `handle_config_command::<Cli, S>(command, config_path)` 处理内置 clap 配置
  命令。

不使用 `confique` 时，可以直接使用 `load_config_tree`。
