# Examples

[English](examples.html) | [中文](../zh/examples.html)

The repository includes runnable examples for loading config trees, CLI
overrides, built-in config commands, template generation, and the lower-level
tree API.

Read the repository examples index:

- [examples/README.md](../../examples/README.md)

Run examples from the repository root:

```bash
cargo run --example basic_loading
cargo run --example cli_overrides -- --server-port 9000
cargo run --example config_commands -- config-template --output /tmp/config.example.yaml
cargo run --example config_commands -- config-schema --output /tmp/myapp.schema.json
cargo run --example config_commands -- config-validate
cargo run --example generate_templates
cargo run --example tree_api
```
