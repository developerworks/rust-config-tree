# 示例

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Português](README.pt.md) | [Svenska](README.sv.md) | [Suomi](README.fi.md) | [Nederlands](README.nl.md)

这些示例是小型可运行程序，会创建自己的临时配置文件。

在仓库根目录运行：

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

示例覆盖：

- `basic_loading.rs`：从递归 config tree 加载 `confique` schema。
- `cli_overrides.rs`：将应用 CLI 参数作为最高优先级 Figment provider 合并。
- `config_commands.rs`：把 `ConfigCommand` flatten 到应用 clap CLI。
- `generate_templates.rs`：从 schema 写入 root/section JSON Schema，以及绑定
  schema 的 TOML/YAML 模板。
- `tree_api.rs`：使用低层、格式无关的 include tree API。
