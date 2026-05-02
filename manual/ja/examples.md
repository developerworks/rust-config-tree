# サンプル

[English](../en/examples.html) | [中文](../zh/examples.html) | [日本語](examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

repository には、config tree loading、CLI overrides、built-in config commands、
template generation、lower-level tree API を扱う実行可能なサンプルが含まれます。

repository examples index:

- [examples/README.ja.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.ja.md)

repository root から実行します。

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

