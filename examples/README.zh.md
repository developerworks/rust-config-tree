# 示例

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

这些示例是小型可运行程序，会创建自己的临时配置文件。

可以在仓库根目录运行这些示例：

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

`config_commands` 的 template(模板) 和 schema(结构定义) 命令使用 CLI 默认路径，
因此 `AppConfig` 会把生成文件写到 `config/app_config/` 下。

这些示例覆盖以下内容：

- `basic_loading.rs` 会从递归 config tree(配置树) 加载 `confique`
  schema(结构定义)。
- `cli_overrides.rs` 会将应用 CLI(命令行接口) 参数作为最高优先级
  Figment(配置合并库) provider(值提供器) 合并。
- `config_commands.rs` 会把 `ConfigCommand` flatten(展开) 到应用的
  clap(命令行解析库) CLI(命令行接口) 中。
- `generate_templates.rs` 会从 schema(结构定义) 写入 root(根配置) 和
  section(配置段) 的 JSON Schema(JSON 结构定义)，也会写入绑定
  schema(结构定义) 的 TOML、YAML、JSON 和 JSON5 模板。
- `tree_api.rs` 会使用低层、格式无关的 include tree API(包含树接口)。
