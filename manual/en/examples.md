# Examples

[English](examples.html) | [中文](../zh/examples.html) | [日本語](../ja/examples.html) | [한국어](../ko/examples.html) | [Français](../fr/examples.html) | [Deutsch](../de/examples.html) | [Español](../es/examples.html) | [Português](../pt/examples.html) | [Svenska](../sv/examples.html) | [Suomi](../fi/examples.html) | [Nederlands](../nl/examples.html)

The repository includes runnable examples for loading config trees, CLI
overrides, built-in config commands, template generation, and the lower-level
tree API.

Read the repository examples index:

- [examples/README.md](https://github.com/developerworks/rust-config-tree/blob/main/examples/README.md)

Run examples from the repository root:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template
cargo run --example config_commands -- config-schema
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```

The `config_commands` template and schema commands use the CLI defaults, so
`AppConfig` writes generated files under `config/app_config/`.
