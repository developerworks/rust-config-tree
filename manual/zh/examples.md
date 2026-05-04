# 示例

[English](../en/examples.html) | [中文](examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

仓库包含可运行示例，覆盖 config tree 加载、CLI 覆盖参数、内置配置命令、模板
生成和低层 tree API。

阅读仓库 examples 索引：

- [examples/README.zh.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.zh.md)

在仓库根目录运行示例：

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output app_config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
