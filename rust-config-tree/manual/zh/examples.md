# 示例

[English](../en/examples.html) | [中文](examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

仓库包含可运行示例。这些示例覆盖 config tree(配置树) 加载、
CLI(命令行接口) 覆盖参数、内置配置命令、模板生成和低层 tree API(树形接口)。

可以阅读仓库 examples(示例目录) 索引：

- [examples/README.zh.md](https://github.com/developerworks/rust-config-tree/blob/main/rust-config-tree/examples/README.zh.md)

在仓库根目录运行示例：

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- generate-template
cargo run --example config_commands -- generate-schema
cargo run --example config_commands -- validate-config
cargo run --example generate_templates
cargo run --example tree_api
cargo run --example transparent_array_section
```

`config_commands` 的 template(模板) 和 schema(结构定义) 命令使用 CLI 默认路径,
因此 `AppConfig` 会把生成文件写到 `config/app_config/` 下.

透明数组 Section(配置段) 的完整说明见 [transparent-sections.md](transparent-sections.md).
