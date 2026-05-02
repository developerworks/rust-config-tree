# 示例

[English](../en/examples.html) | [中文](examples.html)

仓库包含可运行示例，覆盖 config tree 加载、CLI 覆盖参数、内置配置命令、模板
生成和低层 tree API。

阅读仓库 examples 索引：

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

在仓库根目录运行示例：

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
